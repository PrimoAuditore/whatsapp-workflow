use crate::meta_requests::{get_media_url, help_request, send_error_message};
use crate::tools::{create_request_tracker, get_last_event, update_flow_status, FlowStatus};
use actix_web::{get, post, App, HttpRequest, HttpResponse, HttpServer, Responder};
use aws_config::meta::region::RegionProviderChain;
use aws_config::SdkConfig;
use aws_sdk_s3::Client;
use std::collections::HashMap;
use std::env;
use std::os::unix::raw::off_t;
use std::process::Command;
use uuid::Uuid;
use crate::tools::FlowStatus::FlowStarted;

#[macro_use]
extern crate log;

mod meta_requests;
mod s3_tools;
mod structs;
mod tools;

static mut CONFIG: Option<SdkConfig> = None;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    trace!("region provider");
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-2");

    trace!("config creation");
    unsafe { CONFIG = Some(aws_config::from_env().region(region_provider).load().await) }

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(validate)
            .service(events)
            .service(test)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/test")]
async fn test(req_body: String) -> impl Responder {
    send_error_message("test error", "56936748406");
    HttpResponse::Ok().body("")
}

#[post("/webhook")]
async fn events(req_body: String) -> impl Responder {

    debug!("{}", req_body);



    // Check for required env variables
    if env::var("META_TOKEN").is_err() || env::var("REDIS_URL").is_err() {

        error!("One or many env variables are not present");
        panic!("One or many env variables are not present");
    }

    if req_body.contains("user_initiated") {
        return HttpResponse::Created().body("");
    }

    // Parse json body to Event struct
    let data: structs::Event = serde_json::from_str(req_body.as_str()).unwrap();

    if data.entry[0].changes[0].value.statuses.is_none() {
        debug!("{}", req_body);
    }

    let message = data.entry[0].changes[0]
        .value
        .messages
        .clone()
        .unwrap_or_default();

    // Get last registered event from phone number
    let mut client_last_event = get_last_event(&message[0].from);

    // Creates a new flow if client has never had a request on the system
    if client_last_event.is_err() {

        trace!("create tracker id");

        let uuid = Uuid::new_v4().to_string();
        create_request_tracker(&message[0].from, uuid.as_str());

        client_last_event = get_last_event(&message[0].from);
    }

    // Specific data from event for simpler usage
    let current_status: FlowStatus = FlowStatus::get_from_value(
        &client_last_event
            .as_ref()
            .unwrap()
            .status_id
            .as_ref()
            .unwrap(),
    );
    let tracker_id = &client_last_event
        .as_ref()
        .unwrap()
        .tracker_id
        .as_ref()
        .expect("register with not tracker id");
    let phone_number = &message[0].from;

    tools::check_registry_expiry(&current_status, &client_last_event);

    if !tools::check_registry_expiry(&current_status, &client_last_event).unwrap_or(false) {
        update_flow_status(
            &message[0].from,
            &tracker_id,
            FlowStatus::RequestCanceled,
            None,
            None,
        );
        send_error_message("Expiro el tiempo de la solicitud, inicia una solicitud nueva escribiendo 'Hola' en el chat", &message[0].from);
    }

    // Executes step based on current status of request
    match current_status {
        FlowStatus::FlowStarted => {
            //Sends service selection modal to client

            let request_response = meta_requests::service_message(phone_number);

            let update_response = update_flow_status(
                &message[0].from,
                &tracker_id,
                FlowStatus::ServiceModalSent,
                None,
                None,
            );
        }
        FlowStatus::ServiceModalSent => {
            // Sends modal for brand selection

            if !message[0].clone().interactive.is_some() {
                // Message is not received from service button selection
                send_error_message(
                    "Por favor, seleccione una opcion en el mensaje anterior.",
                    phone_number,
                );
            }


            // Check for constraints
            if message[0].clone().interactive.unwrap().button_reply.unwrap().id == "part-search" {
                let update_response = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::ServiceSelected,
                    Some(message[0].clone().interactive.unwrap().button_reply.unwrap().id),
                    None,
                );

                // Sends brand selection modal to client
                let request_response = meta_requests::brand_list(phone_number, 1);

                match request_response {
                    Ok(_) => {
                        let update_response = update_flow_status(
                            &message[0].from,
                            &tracker_id,
                            FlowStatus::BrandModalSent,
                            None,
                            None,
                        );
                    }
                    Err(_) => {
                        panic!("Brand modal couldnt be sent");
                    }
                }
            } else if message[0].clone().button.unwrap().text == "" {
                help_request(&message[0].from);
            }
        }
        FlowStatus::ServiceSelected => {}
        FlowStatus::BrandModalSent => {
            // Registers vehicle brand and send modal for model selection
            // More brands option id: page-1

            if !message[0].clone().interactive.is_some() {
                // Message is not received from service button selection
                meta_requests::send_error_message("Por favor, presiona el boton 'Marcas', y seleccione la marca del vehiculo a consultar.", phone_number);
            }

            debug!(
                "option selected: {}",
                message[0].clone().interactive.unwrap().list_reply.unwrap().id
            );
            let option_selected = message[0].clone().interactive.unwrap().list_reply.unwrap().id;

            if option_selected.contains("page-") {
                let page: i32 = option_selected
                    .split("-")
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .parse()
                    .unwrap();
                meta_requests::brand_list(phone_number, page);
            } else {
                debug!("Brand selected");
                let update_response_selection = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::BrandSelected,
                    Some(option_selected.clone()),
                    None,
                );

                let request_response =
                    meta_requests::model_list(phone_number, option_selected.clone(), 1);

                let update_response_modal = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::ModelModalSent,
                    None,
                    None,
                );
            }
        }
        FlowStatus::BrandSelected => {}
        FlowStatus::ModelModalSent => {
            debug!("model selected");

            // Registers selected model and sends message requesting VIN.
            // More models option id: page-1-chevrolet

            if !message[0].clone().interactive.is_some() {
                // Message is not received from service button selection
                meta_requests::send_error_message("Por favor, presiona el boton 'Modelos', y seleccione el modelo del vehiculo a consultar.",phone_number);
            }

            let option_selected = message[0].clone().interactive.unwrap().list_reply.unwrap().id;

            if option_selected.contains("page-") {
                let page: i32 = option_selected
                    .split("-")
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .parse()
                    .unwrap();
                let make: String = option_selected
                    .split("-")
                    .nth(2)
                    .unwrap()
                    .to_string()
                    .parse()
                    .unwrap();

                meta_requests::model_list(phone_number, make, page);
            } else {
                let update_response_selection = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::ModelSelected,
                    Some(option_selected),
                    None,
                );

                let request_response = meta_requests::request_vin(phone_number);

                let update_response_modal = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::VinRequestSent,
                    None,
                    None,
                );
            }
        }
        FlowStatus::ModelSelected => {}
        FlowStatus::VinRequestSent => {
            let valid_vin = tools::validate_vin(message[0].clone().text.unwrap().body);

            if valid_vin {
                let update_response = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::VinProvided,
                    Some(message[0].clone().text.unwrap().body),
                    None,
                );

                let request_response = meta_requests::request_part_description(phone_number);

                let update_response_part = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::PartDescriptionRequested,
                    None,
                    None,
                );
            }
        }
        FlowStatus::VinProvided => {}
        FlowStatus::PartDescriptionRequested => unsafe {
            let message_value = if message[0].message_type == "image" {
                let image_url = tools::upload_image(message[0].clone().image.unwrap().id, &CONFIG)
                    .await
                    .unwrap();
                image_url
            } else if message[0].message_type == "text" {
                message[0].clone().text.unwrap().body
            } else {
                "".to_string()
            };

            // tools::attach_image(message[0].image.jpeg.unwrap().id);

            let update_response = update_flow_status(
                &message[0].from,
                &tracker_id,
                FlowStatus::PartDescriptionProvided,
                Some(message_value),
                None,
            );

            let request_response = meta_requests::successfull_request(phone_number);

            let update_response = update_flow_status(
                &message[0].from,
                &tracker_id,
                FlowStatus::RequestAccepted,
                None,
                None,
            );
        },
        FlowStatus::PartDescriptionProvided => {}
        FlowStatus::RequestAccepted => {
            if message[0]
                .clone()
                .text
                .unwrap()
                .body
                .to_ascii_lowercase()
                .trim()
                == "hola"
            {
                let uuid = Uuid::new_v4().to_string();
                create_request_tracker(&message[0].from, uuid.as_str());

                let request_response = meta_requests::service_message(phone_number);

                let update_response = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::ServiceModalSent,
                    None,
                    None,
                );
            } else {
                send_error_message(
                    " Si desea iniciar una nueva solicitud escriba 'Hola' en el chat.",
                    &message[0].from,
                );
            }
        }
        FlowStatus::PartFound => {}
        FlowStatus::RequestCanceled => {
            if message[0]
                .clone()
                .text
                .unwrap()
                .body
                .to_ascii_lowercase()
                .trim()
                == "hola"
            {
                let uuid = Uuid::new_v4().to_string();
                create_request_tracker(&message[0].from, uuid.as_str());

                let request_response = meta_requests::service_message(phone_number);

                let update_response = update_flow_status(
                    &message[0].from,
                    &tracker_id,
                    FlowStatus::ServiceModalSent,
                    None,
                    None,
                );
            } else {
                send_error_message(
                    " Si desea iniciar una nueva solicitud escriba 'Hola' en el chat.",
                    &message[0].from,
                );
            }
        }
    }

    HttpResponse::Ok().body("")
}

#[get("/webhook")]
async fn validate(validation_parameters: HttpRequest) -> impl Responder {
    let verify_token = match env::var("VERIFY_TOKEN") {
        Ok(x) => x,
        Err(err) => panic!("{}", err),
    };

    let mut param_map = HashMap::new();

    for param in validation_parameters.query_string().split("&") {
        let param_vec: Vec<&str> = param.split("=").collect();
        param_map.insert(param_vec[0], param_vec[1]);
    }

    if verify_token != param_map.get("hub.verify_token").unwrap().to_string() {
        panic!("Received verification token is not equals to defined one")
    }

    debug!("{:?}", &param_map);

    HttpResponse::Ok().body(param_map.get("hub.challenge").unwrap().to_string())
}
