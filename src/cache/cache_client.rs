use std::sync::Arc;
use std::sync::mpsc::Sender;
use crate::cache::cache_response::CacheResult;
use crate::cache::{Command, GetRange, PutRange};
use crate::Range;

#[derive(Clone)]
pub struct CacheClient{
    pub(in crate::cache) sender: Sender<Command>
}


impl CacheClient {
    pub async fn get_range(&self, seq_name: String, range_size: u64,) -> (Vec<Range>, u64) {
        let cache_response = CacheResult::new();

        let command = GetRange {
            seq_name,
            range_size,
            response: cache_response.clone()
        };


        self.sender.send(Command::GetRange(command)).unwrap();

        println!("sent command, awaiting...");

        *cache_response.await
    }

    pub fn put_range(&self, seq_name: String, range: Range) -> () {
        let cmd = PutRange {
            seq_name,
            range
        };

        self.sender.send(Command::PutRange(cmd)).unwrap();
    }

    pub fn stop(self) {
        self.sender.send(Command::Stop).unwrap();
    }
}