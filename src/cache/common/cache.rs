use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use crate::cache::cache_map::{self, CacheMap, get_range, store_range};
use crate::cache::common::client::CacheClient;
use crate::cache::common::msg::Msg;
use crate::range::{get_range_size, Range, split_range};


pub struct Cache {
    values: CacheMap,
    channel: Receiver<Msg>,
}


impl Cache {
    pub fn new() -> CacheClient {
        let (s, r) = mpsc::channel::<Msg>();

        let client = CacheClient { channel: s };
        let mut cache = Cache { values: cache_map::new(), channel: r };

        thread::spawn(move || {
            let mut c = cache;

            for msg in c.channel {
                match msg {
                    Msg::GetFromCache(g) => {
                        println!("Getting value 2");

                        let result = get_range(g.key, g.range_size, &mut c.values);

                        println!("Got: {:?}", result);

                        g.result.signal(result);
                    }

                    Msg::PutToCache(p) => {
                        println!("Putting value 2");

                        store_range(p.key, p.value, &mut c.values);

                        println!("Now cache is {:?}", &c.values);
                        p.result.signal();
                    }

                    Msg::Stop => break
                }
            }
        });

        return client;
    }
}
