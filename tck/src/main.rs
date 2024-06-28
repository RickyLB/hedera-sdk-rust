use std::collections::HashMap;
use std::f32::consts::E;
use std::hash::Hash;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use hedera::{AccountId, Client, PrivateKey};
use hyper::body::Bytes;
use jsonrpsee::core::{async_trait, client::ClientT};
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use jsonrpsee::server::{RpcModule, Server};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::{ErrorCode, ErrorObject, ErrorObjectOwned};
use jsonrpsee::ConnectionDetails;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing_subscriber::util::SubscriberInitExt;


use once_cell::sync::Lazy;

static GLOBAL_SDK_CLIENT: Lazy<Arc<Mutex<Option<Client>>>> = Lazy::new(|| { Arc::new(Mutex::new(None))});

#[rpc(server, client)]
pub trait Rpc {
    /// Raw method with connection ID.
    #[method(name = "connectionIdMethod", raw_method)]
    async fn raw_method(&self, first_param: usize, second_param: u16) -> Result<usize, ErrorObjectOwned>;

    #[method(name = "createAccount")]
    fn create_account(
        &self,
        public_key: Option<String>,
        initial_balance: Option<i64>,
        receiver_signature_required: Option<bool>,
        max_automatic_token_associations: Option<u32>,
        staked_account_id: Option<String>,
        staked_node_id: Option<u64>,
        decline_staking_reward: Option<bool>,
        account_memo: Option<String>,
        // privateKey: Option<String>,
        // autoRenewPeriod: Option<String>
    ) -> Result<usize, ErrorObjectOwned>;

    #[method(name = "generatePublicKey")]
    fn generate_public_key(&self, private_key: String) -> Result<String, ErrorObjectOwned>;

    #[method(name = "generatePrivateKey")]
    fn generate_private_key(&self) -> Result<String, ErrorObjectOwned>;

    #[method(name = "setup")]
    fn setup(
        &self,
        operator_account_id: Option<String>,
        operator_private_key: Option<String>,
        node_ip: Option<String>,
        node_account_id: Option<String>,
        mirror_network_ip: Option<String>,
    ) -> Result<String, ErrorObjectOwned>;

    // generatePublicKey: ({privateKey}) => {
    // return PrivateKey.fromString(privateKey).publicKey.toString();
    // },
    // generatePrivateKey: () => {
    // return PrivateKey.generateED25519().toString();
    // }
}

pub struct RpcServerImpl;

pub struct SdkClient{ 
    client: Arc<Mutex<Client>>
}

#[async_trait]
impl RpcServer for RpcServerImpl {
    fn setup(&self, 
        operator_account_id: Option<String>,
        operator_private_key: Option<String>,
        node_ip: Option<String>,
        node_account_id: Option<String>,
        mirror_network_ip: Option<String>,
    ) -> Result<String, ErrorObjectOwned> {
        // println!("operator_account_id: {}",&operator_account_id.unwrap());
        // println!("operator_private_key: {}", operator_private_key.unwrap());
        // println!("node_ip: {}", node_ip.unwrap());
        // println!("node_account_id: {}", node_account_id.unwrap());
        // println!("mirror_network_ip: {}", mirror_network_ip.unwrap());

        let mut network: HashMap<String, AccountId> = HashMap::new();

        let client = match (node_ip, node_account_id, mirror_network_ip) {
            (Some(node_ip), Some(node_account_id), Some(mirror_network_ip)) => {
                let account_id = AccountId::from_str(node_account_id.as_str()).unwrap();
                network.insert(node_ip, account_id);

                let client = Client::for_network(network).unwrap();
                client.set_mirror_network([mirror_network_ip]);
                client
            },
            (None, None, None) => {
                Client::for_testnet()
            },
            _ => {
                return Err(ErrorObject::borrowed(
                    -32603,
                    "Failed to setup client",
                    None,
                ))
            }
        };


        let operator_id = if let Some(operator_account_id) = operator_account_id {
            AccountId::from_str(operator_account_id.as_str()).unwrap()
        } else {
            return Err(ErrorObject::borrowed(
                -32603,
                "Invalid operator account id",
                None,
            ))
        };

        let operator_key = if let Some(operator_private_key) = operator_private_key {
            PrivateKey::from_str(operator_private_key.as_str()).unwrap()
        } else {
            return Err(ErrorObject::borrowed(
                -32603,
                "Invalid operator private key",
                None,
            ))
        };

        client.set_operator(operator_id, operator_key);

        let mut global_client = GLOBAL_SDK_CLIENT.lock().unwrap();
        *global_client = Some(client);

        Ok("Success".to_string())
    }



    async fn raw_method(
        &self,
        connection_details: ConnectionDetails,
        _first_param: usize,
        _second_param: u16,
    ) -> Result<usize, ErrorObjectOwned> {
        // Return the connection ID from which this method was called.
        Ok(connection_details.id())
    }

    fn create_account(
        &self,
        public_key: Option<String>,
        _initial_balance: Option<i64>,
        _receiver_signature_required: Option<bool>,
        _max_automatic_token_associations: Option<u32>,
        _staked_account_id: Option<String>,
        _staked_node_id: Option<u64>,
        _decline_stakin_reward: Option<bool>,
        _account_memo: Option<String>,
        // _privateKey: Option<String>,
        // _autoRenewPeriod: Option<String>
    ) -> Result<usize, ErrorObjectOwned> {
        // The normal method does not have access to the connection ID.
        let mut client_guard = GLOBAL_SDK_CLIENT.lock().unwrap();
        let client = 
        println!("client_guard: {:?}", client_guard);


        Ok(usize::MAX)
    }

    fn generate_public_key(&self, private_key: String) -> Result<String, ErrorObjectOwned> {
        let private_key = private_key.trim_end();
        let key_type = PrivateKey::from_str(&private_key).unwrap();

        let public_key = if key_type.is_ed25519() {
            PrivateKey::from_str_ed25519(&private_key).unwrap().public_key().to_string()
        } else if key_type.is_ecdsa() {
            PrivateKey::from_str_ecdsa(&private_key).unwrap().public_key().to_string()
        } else {
            return Err(ErrorObject::owned(
                -1,
                "Unsupported key type".to_string(),
                Some(private_key)
            ));
        };

        Ok(public_key)
    }

    fn generate_private_key(&self) -> Result<String, ErrorObjectOwned> {
        let private_key = PrivateKey::generate_ed25519().to_string();

        Ok(private_key)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("jsonrpsee[method_call{name = \"createAccount\"}]=trace".parse()?);
    tracing_subscriber::FmtSubscriber::builder().with_env_filter(filter).finish().try_init()?;

    let server_addr = run_server().await?;
    let url = format!("http://{}", server_addr);

    let middleware = tower::ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .on_request(
                    |request: &hyper::Request<hyper::Body>, _span: &tracing::Span| tracing::info!(request = ?request, "on_request"),
                )
                .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                    tracing::info!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
                })
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
        );

    // let params = rpc_params![1_u64, 2, 3];
    // let response: Result<String, _> = client.request("createAccount", params).await;
    // tracing::info!("r: {:?}", response);

    println!("Server is running at {}", url);

    loop {}

    Ok(())
}

async fn run_server() -> anyhow::Result<SocketAddr> {
    let server = Server::builder().build("127.0.0.1:80").await?;

    let addr = server.local_addr()?;
    let handle = server.start(RpcServerImpl.into_rpc());

    // In this example we don't care about doing shutdown so let's it run forever.
    // You may use the `ServerHandle` to shut it down or manage it yourself.
    tokio::spawn(handle.stopped());

    Ok(addr)
}