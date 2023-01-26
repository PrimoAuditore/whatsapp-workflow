use crate::structs::MessageLog;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use aws_config::meta::region::RegionProviderChain;
use aws_config::SdkConfig;
use crate::constants::FlowStatus;

#[macro_use]
extern crate log;
mod redis;
mod request_handler;
mod s3_tools;
mod structs;
mod tools;
mod constants;
mod step_functions;

static mut CONFIG: Option<SdkConfig> = None;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();



    HttpServer::new(|| {
        App::new()
            .service(health)
            .service(incoming)
            .service(outgoing)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/incoming")]
async fn incoming(message_log: web::Json<MessageLog>) -> impl Responder {
    let response = request_handler::incoming_message(message_log.0).await;

    match response {
        Ok(response) => HttpResponse::Ok().body(serde_json::to_string(&response).unwrap()),
        Err(response) => {
            HttpResponse::InternalServerError().body(serde_json::to_string(&response).unwrap())
        }
    }
}

#[post("/outgoing")]
async fn outgoing(message_log: web::Json<MessageLog>) -> impl Responder {
    let response = request_handler::outgoing_message(message_log.0).await;

    match response {
        Ok(response) => HttpResponse::Ok().body(serde_json::to_string(&response).unwrap()),
        Err(response) => {
            HttpResponse::InternalServerError().body(serde_json::to_string(&response).unwrap())
        }
    }
}
