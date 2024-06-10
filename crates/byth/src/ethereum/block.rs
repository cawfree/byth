use ethers::prelude::*;

use futures::stream::FuturesUnordered;

#[derive(Clone)]
pub struct ContractDeployment {

    // The transaction the contract was deployed in.
    pub transaction_hash: H256,

    // Deployment address of a created contract.
    pub contract_address: H160,

    // Creation bytecode.
    pub creation_bytecode: String,

}

async fn get_contract_deployment(
    provider: &Provider<Http>,
    receipt: &TransactionReceipt
) -> Result<ContractDeployment, ProviderError> {

    // A panic is acceptable here. This is indicative of a
    // developer error, since we should not attempt to fetch
    // the creation_code from a contract which did not create one.
    let contract_address = receipt.contract_address.expect("Expected creation contract_address.");


    match provider.get_transaction(receipt.transaction_hash).await {

        Ok(transaction) => match transaction {

            Some(transaction) => {

                let creation_code = format!("{:x}", transaction.input);

                Ok(ContractDeployment{

                    transaction_hash: transaction.hash,

                    contract_address: contract_address,

                    // TODO: This is not completely correct because
                    //       we can't guarantee the format of transaction
                    //       data. Here we are making the simplest assumption
                    //       that the transaction `data` was the creation code.
                    creation_bytecode: format!("{}", creation_code.strip_prefix("0x").unwrap_or(&creation_code)),

                })
            },

            None => Err(ProviderError::CustomError(format!("Failed to fetch transaction."))),

        },

        Err(e) => Err(e),

    }

}


pub async fn get_contract_deployments(
    provider: &Provider<Http>,
    block: &Block<TxHash>
) -> Result<Vec<ContractDeployment>, ProviderError> {

    // Fetch the transactions which deployed contracts.
    match provider.get_block_receipts(block.number.unwrap()).await {

        Ok(result) => {

            let contracts: Vec<_> = result
                .iter()
                // If `contract_ddress` is truthy, a contract was created as part of the transaction:
                // https://docs.rs/ethers-core/2.0.11/ethers_core/types/transaction/response/struct.TransactionReceipt.html#structfield.contract_address
                .filter(|r| r.contract_address.is_some())
                .collect();

            let futures: FuturesUnordered<_> = contracts
                .iter()
                .map(|r| get_contract_deployment(provider, r))
                .collect();

            let results = futures.collect::<Vec<Result<ContractDeployment, ProviderError>>>().await;

            let contract_deployments = results
                .iter()
                .filter_map(|result| result.as_ref().ok())
                .map(|result| result.clone())
                .collect::<Vec<ContractDeployment>>();

            let expected = &contracts.len();
            let actual = &contract_deployments.len();

            if expected != actual {
                return Err(ProviderError::CustomError(format!("Expected {} deployments, received {}.", expected, actual)));
            }

            Ok(contract_deployments)
        },

        Err(e) => Err(e),

    }

}
