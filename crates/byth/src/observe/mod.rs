use ethers::prelude::*;

use futures::future::try_join_all;

use crate::db::BythDatabase;
use crate::cli;
use crate::ethereum::block::{get_contract_deployments, ContractDeployment};
use crate::ethereum::rpc::ModalRpc;
use crate::foundry::FoundryProject;

use std::cmp::{min, max};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

mod handler;

fn main_simple_handle_results_processor(
    handle_results: &Vec<Result<handler::HandledContractDeployment, cli::BythError>>
) -> Option<cli:: BythError> {

    for result in handle_results {

        match result {

            Ok(handled_contract_deployment) => {

                let contract_address = &handled_contract_deployment.contract_deployment.contract_address;
                let results = &handled_contract_deployment.results;

                for result in results {

                    match result.success {

                        Some(success) => {

                            if success {
                                cli::debug(format!("🔥 Found a successful hit: {} (0x{:x})", result.signature, &contract_address));
                            }

                        }

                        None => {

                            cli::error(format!("Inconclusive: {} (0x{:x})", result.signature, &contract_address));

                        }

                    }

                }

            },

            Err(e) => {

                cli::error(format!("Error processing contract: {}", e));

            }

        };

    }

    None

}

fn blocking_send(
    sender: &std::sync::mpsc::Sender<ContractDeployment>,
    job_counter: &Arc<std::sync::Mutex<u64>>,
    parallelism: u64,
    data: ContractDeployment,
) {

    // We'll prevent the queue from growing too great
    // in length by delaying the parent thread until
    // the queue empties out.
    while *job_counter.lock().unwrap() >= parallelism * 4 {
        thread::sleep(std::time::Duration::from_secs(1));
    }

    // Forward the `contract_deployment` for processing
    // on the receiver.
    sender.send(data).unwrap();
    // Increment the job counter.
    *job_counter.lock().unwrap() += 1;
}

async fn fetch_block_deployments(
    https: &Provider<Http>,
    block_number: u64,
) -> Result<Vec<ContractDeployment>, cli::BythError> {
    match https.get_block(block_number).await {

        Ok(block) => match block {

            Some(block) => match get_contract_deployments(&https, &block).await {

                Ok(res) => Ok(res),

                Err(e) => Err(cli::BythError::HandlerError(format!("{}", e)))

            },

            None => Err(cli::BythError::HandlerError(format!("Failed to get block."))),

        }

        Err(e) => Err(cli::BythError::HandlerError(format!("{}", e)))

    }
}

async fn producer(
    https: Provider<Http>,
    wss: Provider<Ws>,
    sender: std::sync::mpsc::Sender<ContractDeployment>,
    job_counter: Arc<std::sync::Mutex<u64>>,
    start_at: Option<u64>,
    parallelism: u64,
    debug: bool,
) {

    loop {

        /*
         * If the caller has specified a block number,
         * we may need to wait until the current block
         * matches a future value or we'll need to collect
         * old blocks until we've caught up.
         */

        if let Some(start_at) = start_at {

            // Fetch the current block number.
            let current_block_number = https
                .get_block_number()
                .await
                .unwrap();

            let mut now = std::time::Instant::now();

            // If the `start_at` defined by the caller is less than or
            // equal to the current block, we should process these
            // optimistically.

            if current_block_number > start_at.into() {

                cli::debug(format!("Current block number is {}, but requested {}. Fetching historical blocks...", current_block_number, start_at));

                let mut block_to_fetch = start_at;

                // Here, we cache the `chain_tip` to determine
                // how many blocks we need to process before
                // attempting to quit the loop. By the time the
                // `chain_tip` has been caught up to, it will
                // likely have progressed and will therefore
                // need to be updated.
                let mut chain_tip = https.get_block_number().await.unwrap().as_u64();
                let mut backoff = 0;

                // Fetch historical blocks.
                while block_to_fetch <= chain_tip {

                    // TODO: use dedicated parallelism flag
                    let amount_to_fetch = max(min(chain_tip - block_to_fetch, parallelism * 2), 1);

                    let fetched_blocks = try_join_all(
                        (block_to_fetch..(block_to_fetch + amount_to_fetch))
                            .map(|block_number| fetch_block_deployments(&https, block_number))
                            .collect::<Vec<_>>()
                    )
                        .await;


                    let result = match fetched_blocks {

                        Ok(res) => {

                            let contract_deployments = res
                                .into_iter()
                                .flatten()
                                .collect::<Vec<ContractDeployment>>();

                            // Send the `contract_deployments` over to the consumer
                            // loop for parallel processing.
                            for contract_deployment in contract_deployments {
                                blocking_send(&sender, &job_counter, parallelism, contract_deployment);
                            }

                            None
                        },

                        Err(e) => Some(e),

                    };

                    match result {
                        Some(e) => {

                            println!("encountered block failure {}", e);

                            // HACK: Decrement the block so that we can attempt to retry.
                            block_to_fetch = block_to_fetch - amount_to_fetch;

                            cli::error(format!("{}", e));
                            cli::warn(format!("Repeating attempt to fetch #{}...", block_to_fetch));

                            // Increment the `backoff`. This has two implications.
                            // First, it allows the process to delay for a nonzero
                            // amount of time before repeating. Secondly, in the
                            // case of successive errors, it allows the `backoff`
                            // to grow linearly.
                            backoff += 200;

                            tokio::time::sleep(tokio::time::Duration::from_millis(backoff)).await;

                        },
                        None => {
                            // If no error has occurred, we can reset the `backoff`.
                            backoff = 0;
                        }
                    };

                    // Otherwise, let's increment the block and continue indexing the chain.
                    block_to_fetch = block_to_fetch + amount_to_fetch;

                    // TODO: We need a status flag which can be queried.
                    if now.elapsed() > std::time::Duration::from_secs(15 * 60) {
                        cli::debug(format!("Continuing to track chain tip. Up to block {}.", block_to_fetch));
                        now  = std::time::Instant::now();
                    }

                    // Once we have caught up to the chain_tip, refetch it
                    // to determine whether we need to fetch some more blocks.
                    if block_to_fetch == chain_tip {
                        chain_tip = https.get_block_number().await.unwrap().as_u64();
                    }

                }

                cli::debug(format!("Caught up to chain tip!"));
            } else {

                cli::debug(format!("Current block number is {}. Indexing will begin at {}.", current_block_number, start_at));

            }
        }

        let mut stream = wss
            .subscribe_blocks()
            .await
            .unwrap()
            .take(usize::MAX);

        /* infinite_loop */
        while let Some(block) = stream.next().await {

            // Handle the `start_at` declaration.
            if let Some(start_at) = start_at {

                // If the caller has defined a block to start_at which is greater
                // than the current block, then don't index any data yet.
                if start_at > block.number.expect("Unable to determine block number.").as_u64() {
                    continue;
                }

            }

            let maybe_err = match get_contract_deployments(&https, &block).await {

                Ok(contract_deployments) => {

                    // Send the `contract_deployments` over to the consumer
                    // loop for parallel processing.
                    for contract_deployment in contract_deployments {
                        // Forward the `contract_deployment` for processing
                        // on the receiver.
                        blocking_send(&sender, &job_counter, parallelism, contract_deployment);
                    }

                    None
                },

                Err(e) => Some(cli::BythError::HandlerError(format!("{}", e))),

            };

            // HACK: We might want this to retry in future - the
            // RPC *can* still fail nondeterministically.
            if let Some(e) = maybe_err {
                // TODO: This used to `return Some(e)`, but now we need thread co-ordination.
                // TODO: It would be useful to fall back to the synchronous handler?
                panic!("{}", e);
                //return Some(e);
            }
        }
    }
}

async fn consumer(
    https: Provider<Http>,
    wss: Provider<Ws>,
    receiver: std::sync::mpsc::Receiver<ContractDeployment>,
    job_counter: Arc<std::sync::Mutex<u64>>,
    parallelism: u64,
    projects: Vec<FoundryProject>,
    relative_paths: Vec<String>,
    function: String,
    debug: bool,
) {
    let mut deployments_to_process = vec![];

    loop {

        match receiver.recv() {
            Ok(data) => {

                // Accumulate deployments in preparation for processing.
                deployments_to_process.push(data);

                // If we've buffered the required amount of contracts
                // to execute symbolic evaluation, then we can execute
                // with optimal utilization.
                // TODO: correct parallelism
                if deployments_to_process.len() == parallelism as usize {

                    // Execute symbolic evaluation for result on
                    // the newly discovered contracts.
                    let results = handler::handle_contract_deployments_parallel(
                        &deployments_to_process,
                        &projects,
                        &relative_paths,
                        &function,
                        debug,
                    ).await;

                    main_simple_handle_results_processor(&results);

                    // Clear the buffer to accumulate new contracts in
                    // preparation for evaluation.
                    deployments_to_process.clear();

                    // Decrement the job counter; these tasks are now
                    // completed, and we can pull in more work.
                    *job_counter.lock().unwrap() -= parallelism;
                }

            }
            Err(e) => {
                cli::error(format!("{}", e));
            }
        }
    }
}

pub async fn main(
    _db: BythDatabase,
    rpc_url: Option<String>,
    project: Option<String>,
    parallelism: u64,
    function: Option<String>,
    start_at: Option<u64>,
    debug: bool,
) -> Option<cli::BythError> {

    if let None = rpc_url {
        return Some(cli::BythError::RpcError(format!("You must specify an --rpc-url.")));
    }

    if let None = project {
        return Some(cli::BythError::FoundryProjectError(format!("You must specify a --project.")));
    }

    if parallelism < 1 {
        return Some(cli::BythError::ArgsError(format!("Concurrency must be at least 1.")));
    }

    let project = FoundryProject::new((&project.unwrap()).into()).fork();

    if let Err(e) = project {
        return Some(cli::BythError::FoundryProjectError(format!("{}", e)));
    }

    let project = project.unwrap();

    if let Err(e) = &project.clean() {
        return Some(cli::BythError::FoundryProjectError(format!("{}", e)));
    }

    if let Err(e) = &project.build() {
        return Some(cli::BythError::FoundryProjectError(format!("{}", e)));
    }

    let rpc = ModalRpc::new(&rpc_url.unwrap());

    let relative_paths = project.get_relative_paths();

    if let Err(e) = &relative_paths {
        return Some(cli::BythError::FoundryProjectError(format!("{}", e)));
    }

    let relative_paths = relative_paths.unwrap(); 
 
    let function = function.unwrap_or(format!("check"));

    let https = &rpc
        .new_https_provider()
        .await
        .unwrap();
 
    let wss = &rpc
        .new_wss_provider()
        .await
        .unwrap();

    // Clone multiple instances of each project to
    // permit concurrent symbolic execution.
    let projects: Vec<FoundryProject> = (0..parallelism)
        .map(|_| project.fork().unwrap())
        .collect();

    // Declares a job queue counter to ensure the `producer`
    // cannot overwhelm `receiver`s.
    let job_counter = Arc::new(Mutex::new(0));

    // Enables the request thread to be decoupled from
    // background processing. This way, we can continue
    // to maintain lively processing and exploit parallelized
    // symbolic evaluation whilst working through empty
    // or unfruitful blocks.
    let (sender, receiver) = mpsc::channel::<ContractDeployment>();

    /* producer */
    tokio::spawn(producer(
        https.clone(),
        wss.clone(),
        sender,
        job_counter.clone(),
        start_at, 
        parallelism,
        debug,
    ));

    // TODO: Individual consumer loops should possess their own
    //       `projects` to increase parallelism.
    tokio::spawn(consumer(
        https.clone(),
        wss.clone(), 
        receiver,
        job_counter.clone(),
        parallelism,
        projects,
        relative_paths,
        function,
        debug,
    ));

    thread::park();

    /* unreachable */
    None
    
}
