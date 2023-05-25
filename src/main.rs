use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[get("/sequence/{seq}")]
async fn hello(data: web::Data<AppData>, path: web::Path<String>) -> impl Responder {
    let seq_id = path.into_inner();
    let next_range = data.seq_provider.get_sequence(seq_id);

    match next_range {
        Ok(seq) => HttpResponse::Ok().body(format!("{}:{}", seq.begin, seq.end)),
        Err(Err::NoSeqFound(seq)) => HttpResponse::NotFound().body(format!("No such sequence '{}'", seq))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = AppData { seq_provider: SequenceProvider {} };


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data))
            .service(hello)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(Copy, Clone)]
struct AppData {
    seq_provider: SequenceProvider,
}

struct Sequence {
    begin: u64,
    end: u64,
}

#[derive(Copy, Clone)]
struct SequenceProvider {}


enum Err {
    NoSeqFound(String)
}

impl SequenceProvider {
    fn get_sequence(&self, seq_id: String) -> Result<Sequence, Err> {
        if seq_id == "no_s".to_string() {
            return Err(Err::NoSeqFound(seq_id))
        }

        Ok(Sequence { begin: 0, end: 500 })
    }
}