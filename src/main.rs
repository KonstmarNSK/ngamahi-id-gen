mod etcd_client;

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
    let client = EtcdClient{ client, host_addr: "http://localhost:2379".to_string() };

    let app_data = AppData { seq_provider: RangeProvider { etcd_client: client } };

    Ok(app_data.clone())
}



#[get("/sequence/{seq}")]
async fn get_next_range(data: web::Data<AppData>, path: web::Path<String>) -> impl Responder {
    let seq_id = path.into_inner();
    let next_range = data.seq_provider.get_next_range(seq_id);

    match next_range {
        Ok(seq) => HttpResponse::Ok().body(format!("{}:{}", seq.begin, seq.end)),
        Err(SeqReadErr::NoSeqFound(seq)) => HttpResponse::NotFound().body(format!("No such sequence '{}'", seq))
    }
}

#[post("/sequence/{seq}")]
async fn create_seq(data: web::Data<AppData>, path: web::Path<String>) -> impl Responder {
    let seq_id = path.into_inner();
    let result = data.seq_provider.create_sequence(seq_id);

    match result {
        Ok(()) => HttpResponse::Ok().body(format!("Sequence '{}' created successfully", seq_id)),
        Err(SeqCreationErr::SomeErr(seq)) =>
            HttpResponse::InternalServerError().body(format!("Something bad happened. Unable to create sequence '{}'", seq))
    }
}



#[derive(Clone)]
struct AppData {
    seq_provider: RangeProvider,
}

pub(crate) struct Range {
    begin: u64,
    end: u64,
}

#[derive(Clone)]
struct RangeProvider {
    etcd_client: EtcdClient
}


impl RangeProvider {
    async fn get_next_range(&self, seq_id: String) -> Result<Range, SeqReadErr> {
        self.etcd_client.next_range(seq_id, 500).await
            .map_err(|e| SeqReadErr::EtcdErr(e))
    }

    async fn create_sequence(&self, seq_id: String) -> () {
        self.etcd_client.create_seq(seq_id).await
    }
}


enum SeqReadErr {
    EtcdErr(EtcdErr)
}

enum SeqCreationErr {
    SomeErr(String)
}