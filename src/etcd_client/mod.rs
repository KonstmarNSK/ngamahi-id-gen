mod req_types;
mod resp_types;
mod operations;

use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
use operations::{execute_tx, get_range};
use crate::etcd_client::operations::{CreateSeqTx, EnlargeSeqTx};
use crate::Range;

// use std::cell::RefCell;
// use std::thread;
//
// thread_local! {
//     static CLIENT: RefCell<f32> = RefCell::new(awc::Client);
// }


#[derive(Clone)]
pub struct EtcdClient {
    pub client: awc::Client,
    pub host_addr: String
}

impl EtcdClient {

    pub async fn next_range(&self, seq_name: String, range_size: u32) -> Result<Range, EtcdErr> {

        let mut old_value = get_range(seq_name.clone(), &self.client, self.host_addr.clone()).await;


        for _ in 0..5 { //todo: make a constant
            let new_value = old_value + (range_size as u64);

            let tx = EnlargeSeqTx::new(seq_name.clone(), old_value, new_value);
            let tx_result = execute_tx(&tx, &self.client, self.host_addr.clone()).await;

            if tx_result.succeeded() {
                return Ok(Range{ begin: old_value, end: new_value })
            }

            old_value = tx_result.old_value().unwrap();
        };

        Err(EtcdErr::OptimisticTxFailed)
    }


    pub async fn create_seq(&self, seq_name: String) -> () {
        let tx = CreateSeqTx::new(seq_name);
        execute_tx(&tx, &self.client, self.host_addr.clone()).await;
    }
}



pub enum EtcdErr{
    OptimisticTxFailed
}





