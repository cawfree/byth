use ethers::prelude::*;

use std::time::SystemTime;
use bonsaidb::core::schema::{Collection, SerializedCollection};
use bonsaidb::local::config::{Builder, StorageConfiguration};
use bonsaidb::core::connection::StorageConnection;
use bonsaidb::local::Storage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Collection, Eq, PartialEq)]
#[collection(
    name = "contracts",
    primary_key = String,
    natural_id = |contract: &Contract| Some(
        format!("{:x}", contract.address)
    )
)]
pub struct Contract {
    pub address: H160,
    pub timestamp: SystemTime,
}

#[derive(Debug, Serialize, Deserialize, Collection, Eq, PartialEq)]
#[collection(
    name = "approvals",
    primary_key = [u8; 32],
    natural_id = |approval: &Approval| Some(
        H256(ethers::utils::keccak256(
            abi::encode_packed(&[
                ethers::abi::Token::Address(approval.token),
                ethers::abi::Token::Address(approval.from),
                ethers::abi::Token::Address(approval.to),
            ]).unwrap()
        ))
            .to_fixed_bytes()
    )
)]
pub struct Approval {
    pub token: H160,
    // TODO: Can we make this a contract? We only care about contract approvals.
    pub from: H160,
    pub to: H160,
}

pub struct BythDatabase {
    contracts: bonsaidb::local::Database,
    approvals: bonsaidb::local::Database,
}

impl BythDatabase {

    pub fn new(storage_configuration: StorageConfiguration) -> Result<Self, bonsaidb::core::Error> {

        let storage = Storage::open(
            storage_configuration 
                .with_schema::<Contract>()?
                .with_schema::<Approval>()?
        )?;

        let contracts = storage.create_database::<Contract>("contracts", true)?;
        let approvals = storage.create_database::<Approval>("approvals", true)?;

        Ok(BythDatabase { contracts, approvals })
    }

    #[allow(dead_code)]
    pub fn insert_contract(&self, address: H160) -> Result<(), bonsaidb::core::Error> {

        Contract {
            address,
            timestamp: SystemTime::now(),
        }
        .push_into(&self.contracts)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn insert_approval(&self, token: H160, from: H160, to: H160) -> Result<(), bonsaidb::core::Error> {

        Approval {
            token,
            from,
            to,
        }
        .push_into(&self.approvals)?;

        Ok(())
    }
    
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_insert() {

        let db = BythDatabase::new(StorageConfiguration::new(".$.test.bonsaidb").memory_only()).expect("failed to init db");

        let contract_address = H160::random();

        db.insert_contract(contract_address).expect("failed to insert contract");

        // Attempting to re-insert the contract should fail.
        assert_eq!(db.insert_contract(contract_address).is_ok(), false);

        let token = H160::random();
        let from = H160::random();
        let to = H160::random();

        db.insert_approval(token, from, to).expect("failed to insert approval");

        // Attempting to re-insert the approval should fail.
        assert_eq!(db.insert_approval(token, from, to).is_ok(), false);

    }

}
