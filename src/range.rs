use crate::etcd_client::{EtcdClient, EtcdErr};

#[derive(Clone, Debug)]
pub struct Range {
    pub begin: u64,
    pub end: u64,
}

#[derive(Clone)]
pub struct RangeProvider {
    pub etcd_client: EtcdClient,
}


impl RangeProvider {
    pub async fn get_next_range(&self, seq_id: String) -> Result<Range, EtcdErr> {
        self.etcd_client.next_range(seq_id, 500).await
    }

    pub async fn create_sequence(&self, seq_id: String) -> Result<(), EtcdErr> {
        self.etcd_client.create_seq(seq_id).await
    }
}