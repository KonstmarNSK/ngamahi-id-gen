mod req_types;
mod resp_types;
mod operations;
mod client;
mod http_client;

use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use operations::get_range;
pub use client::EtcdClient;
use crate::etcd_client::operations::{CreateSeqTx, CreateSeqTxErr, EnlargeSeqTx, EnlargeTxErr, GetRangeErr};
use crate::Range;


pub type HttpClient = http_client::HttpClient;
pub use http_client::new_http_client;

pub fn new_etcd_client(client: HttpClient, host: String) -> EtcdClient {
    EtcdClient{
        client,
        host_addr: host,
    }
}




#[derive(Debug)]
pub enum EtcdErr {
    OptimisticTxFailed,
    EnlargeTxErr(EnlargeTxErr),
    CreateSeqTxErr(CreateSeqTxErr),
    NoSuchRangeErr(GetRangeErr),
}

impl From<CreateSeqTxErr> for EtcdErr {
    fn from(value: CreateSeqTxErr) -> Self {
        Self::CreateSeqTxErr(value)
    }
}

impl From<GetRangeErr> for EtcdErr {
    fn from(value: GetRangeErr) -> Self {
        Self::NoSuchRangeErr(value)
    }
}





