mod req_types;
mod resp_types;
mod operations;

use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use operations::get_range;
use crate::etcd_client::operations::{CreateSeqTx, CreateSeqTxErr, EnlargeSeqTx, EnlargeTxErr, GetRangeErr};
use crate::Range;


#[derive(Clone)]
pub struct EtcdClient {
    pub client: awc::Client,
    pub host_addr: String,
}

impl EtcdClient {

    pub async fn next_range(&self, seq_name: String, range_size: u32) -> Result<Range, EtcdErr> {

        let mut old_value = get_range(seq_name.clone(), &self.client, self.host_addr.clone()).await?;

        for _ in 0..5 { //todo: make a property
            let new_value = old_value + (range_size as u64);

            let tx = EnlargeSeqTx::new(seq_name.clone(), old_value, new_value);
            let tx_result = tx.exec(self.host_addr.clone(), &self.client).await;

            match tx_result {
                Err(EnlargeTxErr::StaleSequenceNum { new_num }) => old_value = new_num,
                Err(other) => return Err(EtcdErr::EnlargeTxErr(other)),

                Ok(_) => return Ok(Range { begin: old_value, end: new_value }),
            }
        };

        Err(EtcdErr::OptimisticTxFailed)
    }


    pub async fn create_seq(&self, seq_name: String) -> Result<(), EtcdErr> {
        let tx = CreateSeqTx::new(seq_name);
        Ok(tx.exec(self.host_addr.clone(), &self.client).await?)
    }
}


#[derive(Debug)]
pub enum EtcdErr {
    OptimisticTxFailed,
    EnlargeTxErr(EnlargeTxErr), // todo: rename
    CreateSeqErr(CreateSeqTxErr),
    NoSuchRangeErr(GetRangeErr),
}

impl From<CreateSeqTxErr> for EtcdErr {
    fn from(value: CreateSeqTxErr) -> Self {
        Self::CreateSeqErr(value)
    }
}

impl From<GetRangeErr> for EtcdErr {
    fn from(value: GetRangeErr) -> Self {
        Self::NoSuchRangeErr(value)
    }
}





