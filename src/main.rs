use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use std::collections::HashMap;
use std::env;


//struct ValidationParams {
//    hub.mode: String,
//    hub.challenge: String,
//    hub.verify_token: String,
//}

//impl std::fmt::Display for ValidationParams {
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        write!(f, "(mode: {}, challenge: {}, verify_token: {})", self.hub.mode, self.hub.challenge, hub.verify_token)
//    }
//}

struct Event {
    object: String,
    entry: Vec<Entry>
}

struct Entry {
    id: String,
    time: String,
    changes: Vec<Change>
}

struct Change {
    field: String,
}

struct ChangeValue {

}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}


#[post("/webhook")]
async fn events(req_body: String) -> impl Responder {
    println!("{}", req_body);
    HttpResponse::Ok().body(req_body)
}


#[get("/webhook")]
async fn validate(validation_parameters : HttpRequest) -> impl Responder {


    let verify_token = match env::var("VERIFY_TOKEN") {
        Ok(x) => x,
        Err(err) => panic!("{}", err)
    };

    let mut param_map = HashMap::new();

    for param in validation_parameters.query_string().split("&"){
        let param_vec: Vec<&str> = param.split("=").collect();
        param_map.insert(param_vec[0], param_vec[1]);
    }

    if verify_token != param_map.get("hub.verify_token").unwrap().to_string(){
        panic!("Received verification token is not equals to defined one")
    }

    println!("{:?}", &param_map);


    HttpResponse::Ok().body(param_map.get("hub.challenge").unwrap().to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(validate)
            .service(events)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
