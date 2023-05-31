use serde::{Deserialize, Serialize};
use crate::etcd_client;

//==========|  TRANSACTION  |============

#[derive(Serialize, Deserialize)]
pub(in etcd_client) struct Transaction {
    pub compare: Vec<Comparison>,
    pub success: Vec<OperationRequest>,
    pub failure: Vec<OperationRequest>,
}


#[derive(Serialize, Deserialize)]
pub(in etcd_client) struct Comparison {
    pub key: String,
    pub result: CompareResult,

    #[serde(flatten)]
    pub target_value: Target,
    pub target: CompareTarget,
}


#[derive(Serialize, Deserialize)]
pub(in etcd_client) enum CompareResult {
    EQUAL,
    GREATER,
    LESS,
    NotEqual,
}


#[derive(Serialize, Deserialize)]
pub(in etcd_client) enum CompareTarget {
    VERSION,
    CREATE,
    MOD,
    VALUE,
}


#[derive(Serialize, Deserialize)]
pub(in etcd_client) enum Target {
    #[serde(rename = "version")]
    Version(u64),

    #[serde(rename = "create_revision")]
    CreateRevision(u64),

    #[serde(rename = "mod_revision")]
    ModRevision(u64),

    // base64 encoded
    #[serde(rename = "value")]
    Value(String),
}


#[derive(Serialize, Deserialize)]
pub(in etcd_client) enum OperationRequest {
    #[serde(rename = "requestPut")]
    Put(RequestPut),

    #[serde(rename = "requestRange")]
    Range(RequestRange),
}


//=======================================



#[derive(Serialize, Deserialize)]
pub(in etcd_client) struct RequestRange {
    pub key: String,
}


#[derive(Serialize, Deserialize)]
pub(in etcd_client) struct RequestPut {
    pub key: String,
    pub value: String,
}
