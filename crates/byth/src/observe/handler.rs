use ethers::prelude::*;

use crate::cli;
use crate::ethereum::block::{get_contract_deployments, ContractDeployment};
use crate::foundry::{FoundryProject, FoundryTestResult};

use std::vec::Vec;

pub struct HandledContractDeployment {

    // The contract the handler was executed on.
    pub contract_deployment: ContractDeployment,

    // The results.
    pub results: Vec<FoundryTestResult>,

}

fn handle_contract_deployment(
    project: &FoundryProject,
    relative_paths: &Vec<String>,
    function: &String,
    contract_deployment: &ContractDeployment,
) -> Result<HandledContractDeployment, cli::BythError> {

    let bytecode = &contract_deployment.creation_bytecode;

    if let Some(e) = &project.inject(&relative_paths, bytecode.to_string()) {

        return Err(cli::BythError::FoundryProjectError(format!("{}", e)));

    }

    match project.halmos(function.to_string()) {

        Ok(results) => Ok(HandledContractDeployment {

            contract_deployment: contract_deployment.clone(),

            results: results,

        }),

        Err(e) => Err(cli::BythError::HandlerError(format!("{}", e))),

    }

}

pub async fn handle_contract_deployments_parallel(
    contract_deployments: &Vec<ContractDeployment>,
    projects: &Vec<FoundryProject>,
    relative_paths: &Vec<String>,
    function: &String,
    _debug: bool,
) -> Vec<Result<HandledContractDeployment, cli::BythError>> {

    let mut results: Vec<Result<HandledContractDeployment, cli::BythError>> = vec![];

    // Split the provided bytecode into batches so we can
    // parallelize these across each project deployment.
    for contract_deployments_batch in contract_deployments.chunks(projects.len()).collect::<Vec<&[ContractDeployment]>>().iter() {

        let threads: Vec<_> = contract_deployments_batch
            .iter()
            .enumerate()
            .map(move |(index, &ref contract_deployment)| {
                let project = projects[index].clone();
                let relative_paths = relative_paths.clone();
                let function = function.clone();
                let contract_deployment = contract_deployment.clone();
                std::thread::spawn(move || handle_contract_deployment(&project, &relative_paths, &function, &contract_deployment))
            })
            .collect();

        results.extend(
            threads
                .into_iter()
                .filter_map(|handle| handle.join().ok())
                .collect::<Vec<Result<HandledContractDeployment, cli::BythError>>>()
        );

    }
    
   results

}

// TODO: Remove this function - it is no longer necessary.
pub async fn handle(
    provider: &Provider<Http>,
    block: &Block<TxHash>,
    projects: &Vec<FoundryProject>,
    relative_paths: &Vec<String>,
    function: &String,
    debug: bool,
) -> Result<Vec<Result<HandledContractDeployment, cli::BythError>>, cli::BythError> {

    match get_contract_deployments(provider, &block).await {

        Ok(contract_deployments) => {

            if contract_deployments.len() == 0 {
                return Ok(vec![]);
            }

            if debug {
                cli::debug(format!("Block {:?} (Deployments: {})", &block.number, contract_deployments.len()));
            }

            Ok(
                handle_contract_deployments_parallel(
                    &contract_deployments,
                    projects,
                    relative_paths,
                    function,
                    debug,
                )
                    .await
            )
        },

        // HACK: Here, we expect the caller to retry this call.
        Err(e) => Err(cli::BythError::RpcError(format!("Provider failed to get_contracts_created. {}", e))),

    }

}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::observe::ModalRpc;

    #[tokio::test]
    async fn test_susceptible_block() {

        dotenv::from_filename(".env.test").unwrap();

        let provider = ModalRpc::new(&dotenv::var("ETH_RPC_URL").unwrap())
            .new_https_provider()
            .await
            .unwrap();

        let susceptible_block = &provider
            .get_block(16001838u64)
            .await
            .expect("Failed to fetch block.")
            .expect("Block did not exist.");

        crate::internals::ensure_bindings_exists_in_tmpdir(
            std::env::current_dir().expect("Failed to get current directory").join("..").join("..").join("bindings")
        );

        let project = FoundryProject::new(
            std::env::current_dir().expect("Failed to get current directory").join("..").join("..").join("detectors_default")
        )
            .fork()
            .expect("Unable to fork fixture.");

        let paths = &project.get_relative_paths().expect("failed to get_relative_paths");

        let projects = vec![project];

        let function = format!("check");

        let handle_result = handle(
            &provider,
            susceptible_block,
            &projects,
            &paths,
            &function,
            false,
        )
            .await
            .expect("Call to handle failed.");

        assert_eq!(handle_result.len(), 1);

        let res = handle_result.get(0).unwrap().as_ref().expect("");

        assert_eq!(res.results.len(), 2);
        assert_eq!(format!("{:x}", res.contract_deployment.transaction_hash), format!("d1265bf6397044769c3a2614dc4a860de92d7ce2f1261d2b6bf00b0684182c87"));
        assert_eq!(format!("{:x}", res.contract_deployment.contract_address), format!("c3e7a7e8870b9a35dec7a993363f7036622723de"));

        assert_eq!(res.results.get(0).unwrap().success, Some(true));
        assert_eq!(res.results.get(0).unwrap().signature, "check_1_DangerouslyApproved_ArbitraryBasic(address,address,bytes4)");

    }

}