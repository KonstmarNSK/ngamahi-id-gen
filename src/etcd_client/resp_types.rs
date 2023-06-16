use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub(in crate::etcd_client) struct Header {
    pub cluster_id: Option<String>,
    pub member_id: Option<String>,
    pub revision: Option<String>,
    pub raft_term: Option<String>,
}


//===========|  RANGE  |=============

#[derive(Serialize, Deserialize)]
pub(in crate::etcd_client) struct RangeResponse {
    pub header: Header,
    pub kvs: Option<Vec<RangeResult>>,
    pub count: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(in crate::etcd_client) struct RangeResult {
    pub key: Option<String>,
    pub create_revision: Option<String>,
    pub mod_revision: Option<String>,
    pub version: Option<String>,
    pub value: Option<String>,
}



//============|  PUT  |==================

#[derive(Serialize, Deserialize)]
pub(in crate::etcd_client) struct ResponsePut {
    pub header: Header,
}

//=======================================




#[derive(Serialize, Deserialize)]
pub(in crate::etcd_client) struct OperationResult {
    pub response_range: Option<RangeResponse>,
    pub response_put: Option<ResponsePut>,
}


#[derive(Serialize, Deserialize)]
pub(in crate::etcd_client) struct TxResp {
    pub header: Header,
    pub succeeded: Option<bool>,
    pub responses: Vec<OperationResult>,
}