//!
//! Use the [cosmos_sdk_proto](https://crates.io/crates/cosmos-sdk-proto) library to create clients for GRPC node requests.
//!
use cosmos_sdk_proto::cosmwasm::wasm::v1::msg_client::MsgClient;
use cosmos_sdk_proto::cosmwasm::wasm::v1::query_client::QueryClient;
use cosmrs::bip32;
use cosmrs::crypto::secp256k1::SigningKey;
use cosmrs::AccountId;
use cosmwasm_std::Addr;
use cw_croncat_core::msg::AgentTaskResponse;
use cw_croncat_core::msg::TaskResponse;
use cw_croncat_core::msg::{ExecuteMsg, GetConfigResponse, QueryMsg};
use cw_croncat_core::types::AgentResponse;
use serde::de::DeserializeOwned;
use tendermint_rpc::endpoint::broadcast::tx_commit::TxResult;
use tonic::transport::Channel;
use url::Url;

use crate::client::full_client::CosmosFullClient;
use crate::client::query_client::CosmosQueryClient;
use crate::config::ChainConfig;
use crate::errors::Report;
use crate::logging::info;

///
/// Create message and query clients for interacting with the chain.
///
#[no_coverage]
pub async fn connect(url: String) -> Result<(MsgClient<Channel>, QueryClient<Channel>), Report> {
    // Parse url
    let url = Url::parse(&url)?;

    info!("Connecting to GRPC services @ {}", url);

    // Setup our GRPC clients
    let msg_client = MsgClient::connect(url.to_string()).await?;
    let query_client = QueryClient::connect(url.to_string()).await?;

    info!("Connected to GRPC services @ {}", url);

    Ok((msg_client, query_client))
}

#[derive(Clone)]
pub struct GrpcSigner {
    client: CosmosFullClient,
    account_id: AccountId,
}

impl GrpcSigner {
    pub async fn new(cfg: ChainConfig, key: bip32::XPrv) -> Result<Self, Report> {
        let client = CosmosFullClient::new(cfg, key).await?;
        let account_id = client.key().public_key().account_id(&client.cfg.prefix)?;
        Ok(Self { client, account_id })
    }

    pub async fn query_croncat<T>(&self, msg: &QueryMsg) -> Result<T, Report>
    where
        T: DeserializeOwned,
    {
        let out = self
            .client
            .query_client
            .query_contract(&self.client.cfg.contract_address, msg)
            .await?;
        Ok(out)
    }

    pub async fn execute_croncat(&self, msg: &ExecuteMsg) -> Result<TxResult, Report> {
        let res = self
            .client
            .execute_wasm(msg, &self.client.cfg.contract_address)
            .await?;

        Ok(res.deliver_tx)
    }

    pub async fn register_agent(
        &self,
        payable_account_id: Option<String>,
    ) -> Result<TxResult, Report> {
        self.execute_croncat(&ExecuteMsg::RegisterAgent {
            payable_account_id: payable_account_id.map(Addr::unchecked),
        })
        .await
    }

    pub async fn unregister_agent(&self) -> Result<TxResult, Report> {
        self.execute_croncat(&ExecuteMsg::UnregisterAgent {}).await
    }

    pub async fn update_agent(&self, payable_account_id: String) -> Result<TxResult, Report> {
        let payable_account_id = Addr::unchecked(payable_account_id);
        self.execute_croncat(&ExecuteMsg::UpdateAgent { payable_account_id })
            .await
    }

    pub async fn withdraw_reward(&self) -> Result<TxResult, Report> {
        self.execute_croncat(&ExecuteMsg::WithdrawReward {}).await
    }

    pub async fn proxy_call(&self) -> Result<TxResult, Report> {
        self.execute_croncat(&ExecuteMsg::ProxyCall {}).await
    }

    pub async fn get_agent(&self, account_id: &str) -> Result<Option<AgentResponse>, Report> {
        let res = self
            .query_croncat(&QueryMsg::GetAgent {
                account_id: Addr::unchecked(account_id),
            })
            .await?;
        Ok(res)
    }

    pub async fn check_in_agent(&self) -> Result<TxResult, Report> {
        self.execute_croncat(&ExecuteMsg::CheckInAgent {}).await
    }

    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    pub async fn get_agent_tasks(
        &self,
        account_id: &str,
    ) -> Result<Option<AgentTaskResponse>, Report> {
        let res: Option<AgentTaskResponse> = self
            .query_croncat(&QueryMsg::GetAgentTasks {
                account_id: Addr::unchecked(account_id),
            })
            .await?;
        Ok(res)
    }

    pub fn key(&self) -> SigningKey {
        self.client.key()
    }

    pub fn wsrpc(&self) -> &str {
        &self.client.cfg.wsrpc_endpoint
    }

    pub fn grpc(&self) -> &str {
        &&self.client.cfg.grpc_endpoint
    }
}

pub struct GrpcQuerier {
    client: CosmosQueryClient,
    croncat_addr: String,
}
impl GrpcQuerier {
    pub async fn new(cfg: ChainConfig) -> Result<Self, Report> {
        Ok(Self {
            client: CosmosQueryClient::new(&cfg.grpc_endpoint, &cfg.denom).await?,
            croncat_addr: cfg.contract_address,
        })
    }

    pub async fn query_croncat<T>(&self, msg: &QueryMsg) -> Result<T, Report>
    where
        T: DeserializeOwned,
    {
        let out = self.client.query_contract(&self.croncat_addr, msg).await?;
        Ok(out)
    }

    pub async fn query_config(&self) -> Result<String, Report> {
        let config: GetConfigResponse = self.query_croncat(&QueryMsg::GetConfig {}).await?;
        let json = serde_json::to_string_pretty(&config)?;
        Ok(json)
    }

    pub async fn get_agent(&self, account_id: String) -> Result<String, Report> {
        let agent: Option<AgentResponse> = self
            .query_croncat(&QueryMsg::GetAgent {
                account_id: Addr::unchecked(account_id),
            })
            .await?;
        let json = serde_json::to_string_pretty(&agent)?;
        Ok(json)
    }

    pub async fn get_tasks(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Result<String, Report> {
        let response: Vec<TaskResponse> = self
            .query_croncat(&QueryMsg::GetTasks { from_index, limit })
            .await?;
        let json = serde_json::to_string_pretty(&response)?;
        Ok(json)
    }

    pub async fn get_agent_tasks(&self, account_id: String) -> Result<String, Report> {
        let response: Option<AgentTaskResponse> = self
            .query_croncat(&QueryMsg::GetAgentTasks {
                account_id: Addr::unchecked(account_id),
            })
            .await?;
        let json = serde_json::to_string_pretty(&response)?;
        Ok(json)
    }
}
