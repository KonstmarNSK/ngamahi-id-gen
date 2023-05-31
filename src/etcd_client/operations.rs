use std::io::Read;
use awc::error::{JsonPayloadError, SendRequestError};
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Error;
use crate::etcd_client::req_types::{CompareResult, CompareTarget, Comparison, OperationRequest, RequestPut, RequestRange, Target, Transaction};
use crate::etcd_client::resp_types::{RangeResponse, TxCreateSeqResp, TxEnlargeSeqResp};



/// Get current value of given sequence
pub async fn get_range(seq_id: String, client: &awc::Client, host: String) -> Result<u64, GetRangeErr> {

    let url = host + "/v3/kv/range";
    let encoded_seq_name = general_purpose::STANDARD.encode(seq_id.clone());
    let body = RequestRange { key: encoded_seq_name };
    let body = serde_json::to_string(&body).unwrap();

    // construct request
    let req = client.post(url).insert_header(("User-Agent", "id-gen/1.0"));

    // send request and await response
    let mut res = req.send_body(body).await.unwrap();
    let payload = res.json::<RangeResponse>().limit(2000).await.unwrap();

    let next_range_start = payload.kvs
        .map(|value| { value.first().cloned() })
        .flatten()
        .map(|value| { value.value.clone() })
        .flatten();


    match &next_range_start {
        Some(num) => {
            let next_range_start = general_purpose::STANDARD.decode(num).unwrap();
            return Ok(u64::from_be_bytes(next_range_start.try_into().unwrap()));
        }

        None => Err(GetRangeErr::NoSuchSeq(seq_id))
    }
}




// ===========| Transactions |=============

pub struct EnlargeSeqTx{
    tx: Transaction
}

pub struct CreateSeqTx{
    tx: Transaction
}




impl EnlargeSeqTx {

    pub async fn exec(self, host: String, client: &awc::Client) -> Result<(), EnlargeTxErr> {

        let tx = serde_json::to_string(&self.tx)?;

        let req = client.post(format!("{}/v3/kv/txn", host))
            .insert_header(("User-Agent", "id-gen/1.0"));

        let mut res = req.send_body(tx).await?;
        let res = res.json::<TxEnlargeSeqResp>().limit(2000).await?;


        return if let Some(true) = res.succeeded {
            Ok(())
        } else {
            Err(
                EnlargeTxErr::StaleSequenceNum { new_num: old_value(res).unwrap() }
            )
        };


        fn old_value(mut res: TxEnlargeSeqResp) -> Option<u64> {
            return res.responses.first_mut()
                .map(| res| res.response_range.take())
                .flatten()
                .map(|range| range.kvs.first().cloned())
                .flatten()
                .map(|range| range.value)// base64-encoded
                .flatten()
                .map(|str_value| decode(str_value));


            fn decode(str_val: String) -> u64 {
                let bytes = general_purpose::STANDARD.decode(str_val).unwrap();

                u64::from_be_bytes(bytes.try_into().unwrap())
            }
        }
    }
}



impl CreateSeqTx {

    pub async fn exec(self, host: String, client: &awc::Client) -> Result<(), CreateSeqTxErr> {
        let tx = serde_json::to_string(&self.tx)?;

        let req = client.post(format!("{}/v3/kv/txn", host))
            .insert_header(("User-Agent", "id-gen/1.0"));

        let mut res = req.send_body(tx).await?;
        let mut res = res.json::<TxCreateSeqResp>().limit(2000).await?;


        return if let Some(true) = res.succeeded {
            Ok(())
        } else {
            Err(
                CreateSeqTxErr::SeqAlreadyExists { seq_value: old_value(res).unwrap()}
            )
        };


        fn old_value(mut res: TxCreateSeqResp) -> Option<u64> {
            return res.responses.first_mut()
                .map(| res| res.response_range.take())
                .flatten()
                .map(|range| range.kvs.first().cloned())
                .flatten()
                .map(|range| range.value)// base64-encoded
                .flatten()
                .map(|str_value| decode(str_value));


            fn decode(str_val: String) -> u64 {
                let bytes = general_purpose::STANDARD.decode(str_val).unwrap();

                u64::from_be_bytes(bytes.try_into().unwrap())
            }
        }
    }
}

//=========================



// Transactions creation:

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





// =========| ERRORS |=========

#[derive(Debug)]
pub enum EnlargeTxErr{
    StaleSequenceNum{ new_num: u64 }, // todo: rename
    SendReqErr(SendRequestError),
    SerializationErr(Error),
    DeserializationErr(JsonPayloadError),
}

#[derive(Debug)]
pub enum CreateSeqTxErr{
    SeqAlreadyExists{ seq_value: u64 },
    SendReqErr(SendRequestError),
    SerializationErr(Error),
    DeserializationErr(JsonPayloadError),
}

#[derive(Debug)]
pub enum GetRangeErr{
    NoSuchSeq(String),
}

// =========| Errors conversions |==========


// Enlarge tx:

impl From<Error> for EnlargeTxErr {
    fn from(value: Error) -> Self {
        Self::SerializationErr(value)
    }
}

impl From<SendRequestError> for EnlargeTxErr {
    fn from(value: SendRequestError) -> Self {
        Self::SendReqErr(value)
    }
}

impl From<JsonPayloadError> for EnlargeTxErr {
    fn from(value: JsonPayloadError) -> Self {
        Self::DeserializationErr(value)
    }
}


// CreateSeq tx:

impl From<Error> for CreateSeqTxErr {
    fn from(value: Error) -> Self {
        Self::SerializationErr(value)
    }
}

impl From<SendRequestError> for CreateSeqTxErr {
    fn from(value: SendRequestError) -> Self {
        Self::SendReqErr(value)
    }
}

impl From<JsonPayloadError> for CreateSeqTxErr {
    fn from(value: JsonPayloadError) -> Self {
        Self::DeserializationErr(value)
    }
}






