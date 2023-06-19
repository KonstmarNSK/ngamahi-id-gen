use crate::etcd_client::{EtcdErr, HttpClient};
use crate::etcd_client::operations::{CreateSeqTx, EnlargeSeqTx, EnlargeTxErr, get_range};
use crate::range::Range;


#[derive(Clone)]
pub struct EtcdClient {
    pub client: HttpClient,
    pub host_addr: String,
}


impl EtcdClient {
    pub async fn next_range(&self, seq_name: String, range_size: u64) -> Result<Range, EtcdErr> {

        let mut old_value = get_range(seq_name.clone(), &self.client, self.host_addr.clone()).await?;

        for _ in 0..5 { //todo: make a property
            let new_value = old_value + range_size;

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