use ethers::prelude::*;

pub struct ModalRpc {

    /**
     * Defines the base `rpc_url` to reference. This could
     * be `https` or `wss`. Byth requires both of these at
     * runtime, and is therefore agnostic to the protocol.
     */
    rpc_url: String,

}

impl ModalRpc {

    fn is_wss_url(rpc_url: &str) -> bool {
        rpc_url.to_lowercase().starts_with("wss://")
    }

    fn is_https_url(rpc_url: &str) -> bool {
        rpc_url.to_lowercase().starts_with("https://")
    }
 
    fn get_wss_rpc_url(rpc_url: &str) -> Option<String> {

        if Self::is_wss_url(rpc_url) {
            return Some(rpc_url.to_string());
        }
    
        if Self::is_https_url(rpc_url) {
            return Some(rpc_url.replace("https://", "wss://"));
        }
    
        None
    }
 
    fn get_https_rpc_url(rpc_url: &str) -> Option<String> {
    
        if Self::is_https_url(rpc_url) {
            return Some(rpc_url.to_string());
        }
    
        if Self::is_wss_url(rpc_url) {
            return Some(rpc_url.replace("wss://", "https://"));
        }
    
        None
    }

    pub fn new(rpc_url: &str) -> ModalRpc {

        ModalRpc {
            rpc_url: rpc_url.to_string(),
        }

    }

    fn https(&self) -> Option<String> {
        Self::get_https_rpc_url(&self.rpc_url)
    }

    fn wss(&self) -> Option<String> {
        Self::get_wss_rpc_url(&self.rpc_url)
    }

    pub async fn new_https_provider(&self) -> Result<Provider<Http>, ProviderError> {
        Ok(Provider::<Http>::connect(&Self::https(self).expect("invalid https")).await)
    }

    pub async fn new_wss_provider(&self) -> Result<Provider<Ws>, ProviderError> {
        Provider::<Ws>::connect(&Self::wss(self).expect("invalid wss")).await
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_modal_rpc() {

        let rpc = ModalRpc::new("https://localhost:8545");

        assert_eq!(rpc.https().unwrap(), "https://localhost:8545");
        assert_eq!(rpc.wss().unwrap(), "wss://localhost:8545");

        let rpc = ModalRpc::new("wss://localhost:8545");

        assert_eq!(rpc.wss().unwrap(), "wss://localhost:8545");
        assert_eq!(rpc.https().unwrap(), "https://localhost:8545");

    }

    #[tokio::test]
    async fn test_https_provider() {

        dotenv::from_filename(".env.test").unwrap();

        assert_eq!(
            ModalRpc::new(&dotenv::var("ETH_RPC_URL").unwrap())
                .new_https_provider()
                .await
                .unwrap()
                .get_block(100u64)
                .await
                .is_ok(),
            true,
        );

    }

    #[tokio::test]
    async fn test_wss_provider() {

        dotenv::from_filename(".env.test").unwrap();

        let block = ModalRpc::new(&dotenv::var("ETH_RPC_URL").unwrap())
            .new_wss_provider()
            .await
            .unwrap()
            .subscribe_blocks()
            .await
            .unwrap()
            .take(1)
            .next()
            .await;

        assert_eq!(block.is_some(), true);

    }

}
