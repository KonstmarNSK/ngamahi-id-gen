use std::io::Read;
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use serde::{Deserialize, Serialize};
use crate::etcd_client::req_types::{CompareResult, CompareTarget, Comparison, OperationRequest, RequestPut, RequestRange, Target, Transaction};
use crate::etcd_client::resp_types::{RangeResponse, TxCreateSeqResp, TxEnlargeSeqResp};



pub async fn get_range(seq_id: String, client: &awc::Client, host: String) -> u64 {

    let url = host + "/v3/kv/range";
    let encoded_seq_name = general_purpose::STANDARD.encode(seq_id);
    let body = RequestRange { key: encoded_seq_name };
    let body = serde_json::to_string(&body).unwrap();

    // construct request
    let req = client.post(url).insert_header(("User-Agent", "awc/3.0"));

    // send request and await response
    let mut res = req.send_body(body).await.unwrap();
    let payload = res.json::<RangeResponse>().limit(2000).await.unwrap();

    return u64::from_str(&payload.kvs.first().unwrap().unwrap().value).unwrap();
}





pub async fn execute_tx<TTx: TxWrapper>(tx: &TTx, client: &awc::Client, host: String) -> TTx::ResponseWrapper {
    let tx = serde_json::to_string(tx).unwrap();

    // construct request
    let req = client.post(format!("{}/v3/kv/txn", host))
        .insert_header(("User-Agent", "awc/3.0"));

    // send request and await response
    let mut res = req.send_body(tx).await.unwrap();
    let res = res.json::<TTx::Response>().limit(2000).await.unwrap();

    TTx::ResponseWrapper::new(res)
}






// transaction (request) wrappers:

pub trait TxWrapper {
    type Response: Deserialize;
    type ResponseWrapper: TxRespWrapper;

    fn get_tx(self) -> Transaction;
}


pub struct EnlargeSeqTx{
    tx: Transaction
}

pub struct CreateSeqTx{
    tx: Transaction
}


impl TxWrapper for EnlargeSeqTx {
    type Response = TxEnlargeSeqResp;
    type ResponseWrapper = EnlargeSeqTxRespWrapper;

    fn get_tx(self) -> Transaction {
        self.tx
    }
}

impl TxWrapper for CreateSeqTx {
    type Response = TxCreateSeqResp;
    type ResponseWrapper = CreateSeqTxRespWrapper;

    fn get_tx(self) -> Transaction {
        self.tx
    }
}

//=========================




// transaction responses wrappers:


pub trait TxRespWrapper {
    type TxResp;

    fn new(tx_resp: Self::TxResp) -> Self;
}


pub struct EnlargeSeqTxRespWrapper {
    resp: TxEnlargeSeqResp,
}

pub struct CreateSeqTxRespWrapper {
    resp: TxCreateSeqResp,
}

impl TxRespWrapper for EnlargeSeqTxRespWrapper {
    type TxResp = TxEnlargeSeqResp;

    fn new(tx_resp: Self::TxResp) -> Self {
        Self {
            resp: tx_resp
        }
    }
}

impl TxRespWrapper for CreateSeqTxRespWrapper {
    type TxResp = TxCreateSeqResp;

    fn new(tx_resp: Self::TxResp) -> Self {
        Self {
            resp: tx_resp
        }
    }
}


impl EnlargeSeqTxRespWrapper {
    pub fn succeeded(&self) -> bool {
        self.resp.succeeded.unwrap_or(false)
    }

    pub fn old_value(&self) -> Option<u64> {
        return self.resp.responses.first()
            .map(| res| res.response_range)
            .flatten()
            .map(|range| range.kvs.first())
            .flatten()
            .map(|range| range.value)// base64-encoded
            .flatten()
            .map(|str_value| decode(str_value));


        fn decode(str_val: String) -> u64 {
            let bytes = general_purpose::STANDARD.decode(str_value).unwrap();

            u64::from_be_bytes(bytes.try_into().unwrap())
        }
    }
}


//====================================







// transactions:

impl EnlargeSeqTx {
    pub fn new(sequence_name: String, old_value: u64, new_value: u64) -> Self {
        let key = general_purpose::STANDARD.encode(sequence_name.as_bytes());
        let old_value = general_purpose::STANDARD.encode(old_value.to_be_bytes());
        let new_value = general_purpose::STANDARD.encode(new_value.to_be_bytes());

        Self {
            tx: Transaction {
                compare: vec![
                    Comparison {
                        key: key.clone(),
                        target_value: Target::Value(old_value),
                        target: CompareTarget::VALUE,
                        result: CompareResult::EQUAL,
                    }],

                success: vec![
                    OperationRequest::Put(
                        RequestPut { key: key.clone(), value: new_value }
                    )
                ],

                failure: vec![
                    OperationRequest::Range(
                        RequestRange { key }
                    )
                ],
            }
        }
    }
}


impl CreateSeqTx {
    pub fn new (sequence_name: String) -> Self {
        let key = general_purpose::STANDARD.encode(sequence_name.as_bytes());
        let new_value = general_purpose::STANDARD.encode(0_u64.to_be_bytes());

        Self {
            tx: Transaction {
                compare: vec![
                    // check key version. If it does exist, its version is greater than 0
                    Comparison {
                        key: key.clone(),
                        target_value: Target::Version(0_u64),
                        target: CompareTarget::VERSION,
                        result: CompareResult::EQUAL,
                    }],

                success: vec![
                    OperationRequest::Put(
                        RequestPut { key: key.clone(), value: new_value }
                    )
                ],

                failure: vec![
                    OperationRequest::Range(
                        RequestRange { key }
                    )
                ],
            }
        }
    }
}








