use actix_web::{HttpResponse, Responder, web, get, post};
use crate::AppData;




#[get("/sequence/{seq}")]
pub async fn get_next_range(data: web::Data<AppData>, path: web::Path<String>) -> impl Responder {
    let seq_id = path.into_inner();
    let next_range = data.seq_provider.get_next_range(seq_id).await;

    match next_range {
        Ok(seq) => HttpResponse::Ok().body(format!("{}:{}", seq.begin, seq.end)),
        Err(err) => HttpResponse::NotFound().body(format!("Error: '{:?}'", err))
    }
}

#[post("/sequence/{seq}")]
pub async fn create_seq(data: web::Data<AppData>, path: web::Path<String>) -> impl Responder {
    let seq_id = path.into_inner();
    let result = data.seq_provider.create_sequence(seq_id.clone()).await;

    // HttpResponse::Ok().body(format!("Sequence '{}' created successfully", seq_id.clone()))

    match result {
        Ok(_) => HttpResponse::Ok().body(format!("Sequence '{}' created successfully", seq_id)),
        Err(err) =>
            HttpResponse::InternalServerError().body(format!("Something bad happened. Unable to create sequence '{:?}'", err))
    }
}
