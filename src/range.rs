use serde::Serialize;
use crate::cache::client::CacheClient;
use crate::config::Properties;
use crate::etcd_client::{EtcdClient, EtcdErr};

#[derive(Clone, Debug, Serialize)]
pub struct Range {
    pub begin: u64,
    pub end: u64,
}

#[derive(Clone)]
pub struct RangeProvider {
    pub etcd_client: EtcdClient,
    pub cache: CacheClient,

    pub etcd_fetch_size: u64,
    pub max_client_range_size: u64,
}


impl RangeProvider {
    pub async fn get_next_range(&self, seq_id: String, range_size: u64) -> Result<Vec<Range>, RangeProviderErr> {
        // check if client requests a range of proper size
        if range_size > self.max_client_range_size {
            return Err(
                RangeProviderErr::Validation(
                    format!("Client requested too large range (requested {}, max {})",
                            &range_size, self.max_client_range_size)
                ))
        }

        // first, try to get requested range from cache
        let (mut from_cache, needed) = self.cache.get(seq_id.clone(), range_size).await;
        if needed == 0 {
            return Ok(from_cache)
        }

        // if there wasn't enough ranges in cache, get new range from etcd
        let new_range = self.etcd_client.next_range(seq_id.clone(), self.etcd_fetch_size).await?;
        let (left, rest) = split_range(new_range, needed).unwrap();

        // one part of new range is returned alongside with cached ones, rest is pushed to cache
        self.cache.put(seq_id, rest).await;
        from_cache.push(left);

        Ok(from_cache)
    }

    pub async fn create_sequence(&self, seq_id: String) -> Result<(), EtcdErr> {
        self.etcd_client.create_seq(seq_id).await
    }
}


pub fn get_range_size(r: &Range) -> u64 {
    r.end - r.begin
}

// left's size is size, right is the rest
// returns none if size is bigger than one of given Range
pub fn split_range(r: Range, size: u64) -> Option<(Range, Range)> {
    println!("Must split range {:?}. Size is {}", &r, size);
    println!("The range size is {}", get_range_size(&r));

    if get_range_size(&r) <= size {
        return None;
    }

    let left = Range {
        begin: r.begin,
        end: r.begin + size,
    };

    let right = Range {
        begin: r.begin + size + 1,
        end: r.end
    };

    return Some((left, right));
}






#[derive(Debug)]
pub enum RangeProviderErr{
    Etcd(EtcdErr),
    Validation(String)
}

impl From<EtcdErr> for RangeProviderErr {
    fn from(value: EtcdErr) -> Self {
        Self::Etcd(value)
    }
}