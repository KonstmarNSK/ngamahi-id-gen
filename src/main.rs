mod etcd_client;
mod cache;
mod range;
mod api_endpoints;
mod config;

use std::convert::Infallible;
use std::future::Future;
use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use actix_web::web::{BufMut, Data};
use awc::error::{PayloadError, SendRequestError};
use crate::etcd_client::{EtcdClient, EtcdErr};
use actix_web::middleware::Logger;
use log4rs;
use log::{info, warn};
use crate::config::Properties;
use crate::range::{Range, RangeProvider};
use crate::api_endpoints::{get_next_range, create_seq};
use crate::cache::CacheClient;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configs = config::read_configs().unwrap();
    let logger_cfg = &configs.logs_cfg_path;

    log4rs::init_file(logger_cfg, Default::default()).unwrap();

    let cache = cache::new_thread_local();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(get_app_data(configs.props.clone(), cache.clone())))
            .service(get_next_range)
            .service(create_seq)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}


fn get_app_data(props: Properties, cache: CacheClient) -> AppData {
    let http_client = awc::Client::default();
    let client = EtcdClient { client: http_client, host_addr: (&props.etcd_addr).clone() };

    AppData {
        seq_provider: RangeProvider {
            etcd_client: client,
            cache,
            etcd_fetch_size: props.etcd_fetch_range_size,
            max_client_range_size: props.client_range_max_size
        },
    }
}



#[derive(Clone)]
pub struct AppData {
    seq_provider: RangeProvider,
}




#[actix_web::test]
pub async fn do_test() {
    assert_eq!(2 + 2, 4);

    println!("creating cache");

    let cache = cache::new_common();

    let rng = Range {
        begin: 0,
        end: 100
    };

    println!("putting");

    cache.put("some-seq".to_string(),rng ).await;

    let from_cache = cache.get("some-seq".to_string(), 100).await;

    cache.stop().await;

    assert_eq!(from_cache.0.len(), 1);

    let range_from_cache = from_cache.0.first().unwrap();
    let needed = from_cache.1;

    println!("Got range: {} to {}, needed {}", range_from_cache.begin, range_from_cache.end, needed);

    assert_eq!(0, range_from_cache.begin);
    assert_eq!(100, range_from_cache.end);
    assert_eq!(0, needed);
}
