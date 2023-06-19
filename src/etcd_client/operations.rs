use std::io::Read;
use awc::error::{JsonPayloadError, SendRequestError};
use base64::{Engine as _, engine::{self, general_purpose}, alphabet, DecodeError};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Error;
use crate::etcd_client::http_client::make_request;
use crate::etcd_client::HttpClient;
use crate::etcd_client::req_types::{CompareResult, CompareTarget, Comparison, OperationRequest, RequestPut, RequestRange, Target, Transaction};
use crate::etcd_client::resp_types::{RangeResponse, TxResp};


/// Get current value of given sequence
pub async fn get_range(seq_id: String, client: &HttpClient, host: String) -> Result<u64, GetRangeErr> {
    let url = host + "/v3/kv/range";
    let encoded_seq_name = general_purpose::STANDARD.encode(seq_id.as_str());
    let body = RequestRange { key: encoded_seq_name };
    let body = serde_json::to_string(&body).unwrap();

    let response = make_request::<RangeResponse>(body, url, client).await?;

    let mut range_op_result = response.kvs.ok_or_else(|| GetRangeErr::NoSuchSeq(seq_id))?;

    let next_range_start = match range_op_result.into_iter().next() {
        Some(range) => range.value,
        None => return Err(EtcdInteropErr::DeserializationErr(
                DeserializeErr::Common("Etcd didn't sent back value".to_string())).into())
    };

    let next_range_start = next_range_start
        .ok_or_else(|| EtcdInteropErr::DeserializationErr(
            DeserializeErr::Common("Etcd didn't sent back value".to_string())))?;

    Ok(num_from_base64(next_range_start.as_str())
        .map_err(|e| EtcdInteropErr::from(e))?)
}


// ===========| Transactions |=============

pub struct EnlargeSeqTx {
    tx: Transaction,
}

pub struct CreateSeqTx {
    tx: Transaction,
}


impl EnlargeSeqTx {
    pub async fn exec(self, mut host: String, client: &HttpClient) -> Result<(), EnlargeTxErr> {
        let response = execute_tx::<TxResp>(&self.tx, host, client).await?;

        return if let Some(true) = response.succeeded {
            Ok(())
        } else {
            Err(EnlargeTxErr::StaleSequenceNum { new_num: unwrap_seq_value(response)? })
        };
    }
}


impl CreateSeqTx {
    pub async fn exec(self, mut host: String, client: &HttpClient) -> Result<(), CreateSeqTxErr> {
        let response = execute_tx::<TxResp>(&self.tx, host, client).await?;

        return if let Some(true) = response.succeeded {
            Ok(())
        } else {
            Err(CreateSeqTxErr::SeqAlreadyExists { seq_value: unwrap_seq_value(response)? })
        };
    }
}


// ===========| Transactions creation |=============

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
    pub fn new(sequence_name: String) -> Self {
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


// =========| Utils |==============

fn unwrap_seq_value(mut res: TxResp) -> Result<u64, RangeRespParsingErr> {
    let response = res.responses.into_iter().nth(0);
    let response = response.ok_or_else(||
        RangeRespParsingErr::Common("Couldn't get value of sequence. \
                    No operation responses from etcd.".to_string())
    )?;

    let response = response.response_range.ok_or_else(||
        RangeRespParsingErr::Common("Couldn't get value of sequence. \
                    No response of range operation from etcd.".to_string())
    )?;

    return unwrap_range_response(response);
}

fn unwrap_range_response(rang_resp: RangeResponse) -> Result<u64, RangeRespParsingErr> {
    let value = rang_resp.kvs.into_iter().nth(0);
    let value = value.ok_or_else(||
        RangeRespParsingErr::Common("Couldn't parse range op response: no such key".to_string()))?;

    let value = value.into_iter().nth(0).ok_or_else(||
        RangeRespParsingErr::Common("Couldn't parse range op response: no such key".to_string()))?;

    let value = value.value.ok_or_else(||
        RangeRespParsingErr::Common("Couldn't parse range result value".to_string()))?;

    let value = num_from_base64(value.as_str())?;

    return Ok(value);
}

fn num_from_base64(encoded: &str) -> Result<u64, Base64DecodeErr> {
    let next_range_start = general_purpose::STANDARD.decode(encoded)?;
    let bytes: [u8; std::mem::size_of::<u64>()] = next_range_start.try_into()
        .map_err(|e| Base64DecodeErr::NumFromBytesErr("Couldn't parse u64 from bytes".to_string()))?;

    Ok(u64::from_be_bytes(bytes))
}


async fn execute_tx<TResp>(tx: &Transaction, mut host: String, client: &HttpClient) -> Result<TResp, EtcdInteropErr>
    where
        TResp: DeserializeOwned
{
    let tx = serde_json::to_string(tx).map_err(|e| EtcdInteropErr::SerializationErr(e))?;
    host.push_str("/v3/kv/txn");

    make_request::<TResp>(tx, host, client).await
}


// =========| ERRORS |=========

#[derive(Debug)]
pub enum EnlargeTxErr {
    StaleSequenceNum { new_num: u64 },
    EtcdInteropError(EtcdInteropErr),
}

#[derive(Debug)]
pub enum CreateSeqTxErr {
    SeqAlreadyExists { seq_value: u64 },
    EtcdInteropError(EtcdInteropErr),
}

#[derive(Debug)]
pub enum GetRangeErr {
    NoSuchSeq(String),
    EtcdInteropError(EtcdInteropErr),
}

#[derive(Debug)]
pub enum RangeRespParsingErr {
    Common(String),
    Base64(Base64DecodeErr),
}

#[derive(Debug)]
pub enum EtcdInteropErr {
    SendReqErr(SendRequestError),
    SerializationErr(Error),
    DeserializationErr(DeserializeErr),
    Base64DecodeErr(Base64DecodeErr),
}

#[derive(Debug)]
pub enum DeserializeErr {
    Common(String),
    JsonPayload(JsonPayloadError),
}

#[derive(Debug)]
pub enum Base64DecodeErr {
    DecodeErr(DecodeError),
    NumFromBytesErr(String),
}

// =========| Errors conversions |==========

impl From<Base64DecodeErr> for EtcdInteropErr {
    fn from(value: Base64DecodeErr) -> Self {
        Self::Base64DecodeErr(value)
    }
}

impl From<Base64DecodeErr> for RangeRespParsingErr {
    fn from(value: Base64DecodeErr) -> Self {
        Self::Base64(value)
    }
}

impl From<DeserializeErr> for EtcdInteropErr {
    fn from(value: DeserializeErr) -> Self {
        Self::DeserializationErr(value)
    }
}

impl From<RangeRespParsingErr> for EtcdInteropErr {
    fn from(value: RangeRespParsingErr) -> Self {
        match value {
            RangeRespParsingErr::Common(s) =>
                Self::DeserializationErr(DeserializeErr::Common(s)),

            RangeRespParsingErr::Base64(err) =>
                Self::Base64DecodeErr(err)
        }
    }
}

impl From<RangeRespParsingErr> for EnlargeTxErr {
    fn from(value: RangeRespParsingErr) -> Self {
        Self::EtcdInteropError(value.into())
    }
}

impl From<RangeRespParsingErr> for CreateSeqTxErr {
    fn from(value: RangeRespParsingErr) -> Self {
        Self::EtcdInteropError(value.into())
    }
}

impl From<EtcdInteropErr> for EnlargeTxErr {
    fn from(value: EtcdInteropErr) -> Self {
        Self::EtcdInteropError(value)
    }
}

impl From<EtcdInteropErr> for CreateSeqTxErr {
    fn from(value: EtcdInteropErr) -> Self {
        Self::EtcdInteropError(value)
    }
}

impl From<EtcdInteropErr> for GetRangeErr {
    fn from(value: EtcdInteropErr) -> Self {
        Self::EtcdInteropError(value)
    }
}

impl From<SendRequestError> for EtcdInteropErr {
    fn from(value: SendRequestError) -> Self {
        Self::SendReqErr(value)
    }
}

impl From<DecodeError> for Base64DecodeErr {
    fn from(value: DecodeError) -> Self {
        Self::DecodeErr(value)
    }
}