use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct Header {
    pub cluster_id: Option<String>,
    pub member_id: Option<String>,
    pub revision: Option<String>,
    pub raft_term: Option<String>,
}


//===========|  RANGE  |=============

#[derive(Serialize, Deserialize)]
pub struct RangeResult {
    pub key: Option<String>,
    pub create_revision: Option<String>,
    pub mod_revision: Option<String>,
    pub version: Option<String>,
    pub value: Option<String>,
}


#[derive(Serialize, Deserialize)]
pub struct RangeResponse {
    pub header: Header,
    pub kvs: Vec<RangeResult>,
    pub count: String,
}

//===================================




//============|  PUT  |==================

#[derive(Serialize, Deserialize)]
pub struct ResponsePut {
    pub header: Header,
}

//=======================================




//=============| TX CREATE SEQ RESPONSE  |======================

#[derive(Serialize, Deserialize)]
pub struct PutResult {
    pub response_put: ResponsePut,
}

#[derive(Serialize, Deserialize)]
pub struct TxCreateSeqResp {
    pub header: Header,
    pub succeeded: Option<bool>,
    pub responses: Vec<PutResult>,
}

//===============================================================





//=============| TX ENLARGE SEQ RESPONSE  |======================

#[derive(Serialize, Deserialize)]
pub struct ResponseRange {
    pub header: Header,
    pub kvs: Vec<RangeResult>,
    pub count: String,
}

#[derive(Serialize, Deserialize)]
pub struct OperationResult {
    pub response_range: Option<ResponseRange>,
    pub response_put: Option<ResponsePut>,
}

#[derive(Serialize, Deserialize)]
pub struct TxEnlargeSeqResp {
    pub header: Header,
    pub succeeded: Option<bool>,
    pub responses: Vec<OperationResult>,
}


//===============================================================