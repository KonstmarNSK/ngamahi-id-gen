mod etcd_client;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::web::BufMut;
use awc::error::{PayloadError, SendRequestError};
use crate::etcd_client::EtcdClient;


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let client = awc::Client::default();
    let client = EtcdClient{ client, host_addr: "http://localhost:2379".to_string() };

    let app_data = AppData { seq_provider: RangeProvider { etcd_client: client } };


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data))
            .service(get_next_range)
            .service(create_seq)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
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
        Ok(seq) => HttpResponse::Ok().body(format!("Sequence '{}' created successfully", seq)),
        Err(SeqCreationErr::SomeErr(seq)) =>
            HttpResponse::InternalServerError().body(format!("Something bad happened. Unable to create sequence '{}'", seq))
    }
}



#[derive(Copy, Clone)]
struct AppData {
    seq_provider: RangeProvider,
}

struct Range {
    begin: u64,
    end: u64,
}

#[derive(Copy, Clone)]
struct RangeProvider {
    etcd_client: EtcdClient
}


impl RangeProvider {
    fn get_next_range(&self, seq_id: String) -> Result<Range, SeqReadErr> {
        if seq_id == "no_s".to_string() {
            return Err(SeqReadErr::NoSeqFound(seq_id))
        }

        Ok(Range { begin: 0, end: 500 })
    }

    fn create_sequence(&self, seq_id: String) -> Result<String, SeqCreationErr> {
        Ok(seq_id)
    }
}



enum SeqReadErr {
    NoSeqFound(String)
}

enum SeqCreationErr {
    SomeErr(String)
}

#[derive(Debug)]
pub enum ClientError {
    RequestError(SendRequestError),
    ResponseError(PayloadError)
}

impl From<PayloadError> for ClientError {
    fn from(value: PayloadError) -> Self {
        ClientError::ResponseError(value)
    }
}

impl From<SendRequestError> for ClientError {
    fn from(value: SendRequestError) -> Self {
        ClientError::RequestError(value)
    }
}


#[actix_web::test]
pub async fn do_test() -> Result<(), ClientError> {
    let mut client = awc::Client::default();

    // construct request
    let req = client.post("http://localhost:2379/v3/kv/range")
        .insert_header(("User-Agent", "awc/3.0"));

    // send request and await response
    let mut res = req.send_body("{\"key\": \"c29tZV9rZXlfaHR0cA==\"}").await?;
    let payload = res.body().limit(2000).await?;
    println!("Response: {:?}", payload);


    assert_eq!(4, 2 + 2);

    Ok(())
}