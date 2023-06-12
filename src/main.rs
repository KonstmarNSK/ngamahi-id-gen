mod etcd_client;
mod cache;

use std::convert::Infallible;
use std::future::Future;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::web::BufMut;
use awc::error::{PayloadError, SendRequestError};
use crate::etcd_client::{EtcdClient, EtcdErr};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .data_factory(get_app_data)
            .service(get_next_range)
            .service(create_seq)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}


async fn get_app_data() -> Result<AppData, Infallible> {
    let client = awc::Client::default();
    let client = EtcdClient { client, host_addr: "http://localhost:2379".to_string() };

    let app_data = AppData { seq_provider: RangeProvider { etcd_client: client } };

    Ok(app_data.clone())
}


#[get("/sequence/{seq}")]
async fn get_next_range(data: web::Data<AppData>, path: web::Path<String>) -> impl Responder {
    let seq_id = path.into_inner();
    let next_range = data.seq_provider.get_next_range(seq_id).await;

    match next_range {
        Ok(seq) => HttpResponse::Ok().body(format!("{}:{}", seq.begin, seq.end)),
        Err(err) => HttpResponse::NotFound().body(format!("Error: '{:?}'", err))
    }
}

#[post("/sequence/{seq}")]
async fn create_seq(data: web::Data<AppData>, path: web::Path<String>) -> impl Responder {
    let seq_id = path.into_inner();
    let result = data.seq_provider.create_sequence(seq_id.clone()).await;

    // HttpResponse::Ok().body(format!("Sequence '{}' created successfully", seq_id.clone()))

    match result {
        Ok(_) => HttpResponse::Ok().body(format!("Sequence '{}' created successfully", seq_id)),
        Err(err) =>
            HttpResponse::InternalServerError().body(format!("Something bad happened. Unable to create sequence '{:?}'", err))
    }
}


#[derive(Clone)]
struct AppData {
    seq_provider: RangeProvider,
}

#[derive(Clone, Debug)]
pub struct Range {
    begin: u64,
    end: u64,
}

#[derive(Clone)]
struct RangeProvider {
    etcd_client: EtcdClient,
}


impl RangeProvider {
    async fn get_next_range(&self, seq_id: String) -> Result<Range, EtcdErr> {
        self.etcd_client.next_range(seq_id, 500).await
    }

    async fn create_sequence(&self, seq_id: String) -> Result<(), EtcdErr> {
        self.etcd_client.create_seq(seq_id).await
    }
}


#[actix_web::test]
pub async fn do_test() {
    assert_eq!(2 + 2, 4);

    println!("creating cache");

    let cache = cache::new_cache();

    let rng = Range {
        begin: 0,
        end: 100
    };

    println!("putting");

    cache.put("some-seq".to_string(),rng ).await;

    let from_cache = cache.get("some-seq".to_string(), 100).await;

    cache.stop();

    assert_eq!(from_cache.0.len(), 1);

    let range_from_cache = from_cache.0.first().unwrap();
    let needed = from_cache.1;

    println!("Got range: {} to {}, needed {}", range_from_cache.begin, range_from_cache.end, needed);

    assert_eq!(0, range_from_cache.begin);
    assert_eq!(100, range_from_cache.end);
    assert_eq!(0, needed);
}