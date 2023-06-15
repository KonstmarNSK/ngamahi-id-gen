use std::sync::mpsc::Sender;
use crate::cache::common::msg::{Msg, MsgGet, MsgPut};
use crate::cache::common::waker::{Flag, GetResult};
use crate::range::Range;


#[derive(Clone)]
pub struct CacheClient {
    pub channel: Sender<Msg>,
}

impl CacheClient {
    pub async fn put(&self, key: String, value: Range) {
        println!("Putting value");

        let flag = Flag::new();

        let msg = MsgPut {
            key,
            value,
            result: flag.clone(),
        };


        self.channel.send(Msg::PutToCache(msg)).unwrap();

        flag.await;
    }

    pub async fn get(&self, key: String, range_size: u64) -> (Vec<Range>, u64) {
        println!("Getting value");

        let flag = GetResult::new();

        let msg = MsgGet {
            key,
            range_size,
            result: flag.clone(),
        };

        self.channel.send(Msg::GetFromCache(msg)).unwrap();

        return *flag.await;
    }

    pub async fn stop(&self) {
        self.channel.send(Msg::Stop).unwrap();
    }
}


