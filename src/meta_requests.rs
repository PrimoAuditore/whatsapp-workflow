use std::error::Error;
use ureq::Response;
use crate::structs::ListChoice;
use crate::tools::{get_brand_models, get_makes};

pub(crate) fn service_message(phone_number: &str) -> Result<(), Box<dyn Error>>{


    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization",  format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!({
    "messaging_product": "whatsapp",
    "recipient_type": "individual",
    "to": phone_number,
    "type": "template",
    "template": {
        "name": "solicitud_vin_1",
    "language": {
            "code": "es"
        }
    }
    }))?.into_string().unwrap();
    Ok(())
}

pub(crate) fn help_request(phone_number: &str) -> Result<(), Box<dyn Error>>{


    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization",  format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!({
    "messaging_product": "whatsapp",
    "recipient_type": "individual",
    "to": phone_number,
    "type": "template",
    "template": {
        "name": "solicitud_ayuda",
    "language": {
            "code": "es"
        }
    }
    }))?.into_string().unwrap();
    Ok(())
}

pub(crate) fn request_vin(phone_number: &str) -> Result<(), Box<dyn Error>>{

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!({
    "messaging_product": "whatsapp",
    "recipient_type": "individual",
    "to": phone_number,
    "type": "template",
    "template": {
        "name": "solicitud_vin_3",
    "language": {
            "code": "es"
        }
    }
    }))?
        .into_string()?;

    Ok(())
}

pub(crate) fn model_list(phone_number: &str, brand_selected: String, page:i32) -> Result<(), Box<dyn Error>>{



    let models_list = get_brand_models(brand_selected.to_ascii_lowercase().as_str(),page, brand_selected.as_str());
    let modal_title = format!("Modelos {} - {}", brand_selected[0..1].to_uppercase() + &brand_selected[1..] ,page);

    if models_list.is_err() {
        panic!("Error obtaining model list");
    }

    let model_json = serde_json::to_string(&models_list.unwrap()).unwrap();

    println!("{}", model_json);
    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!({
    "recipient_type": "individual",
    "to": phone_number,
    "messaging_product": "whatsapp",
    "type": "interactive",
    "interactive": {
        "type": "list",
    "header": {
            "type": "text",
    "text": "Busqueda de repuesto"
        },
    "body": {
            "text": "Seleccione modelo de auto."
        },
    "action": {
            "button": "Modelos",
    "sections": [
            {
                "title": modal_title,
            "rows": model_json
            }
                    ]
        }
    }
    }))?
        .into_string()?;

    println!("response {}", resp);

    Ok(())
}
pub(crate) fn brand_list(phone_number: &str, page:i32) -> Result<(), Box<dyn Error>>{

    let make_list = get_makes(page);
    let modal_title = format!("Marcas - pagina {}", page);

    if make_list.is_err() {
        panic!("Error obtaining model list");
    }

    let make_json = serde_json::to_string(&make_list.unwrap()).unwrap();

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!({
    "recipient_type": "individual",
    "to": phone_number,
    "messaging_product": "whatsapp",
    "type": "interactive",
    "interactive": {
        "type": "list",
    "header": {
            "type": "text",
    "text": "Busqueda de repuesto"
        },
    "body": {
            "text": "Seleccione marca de auto."
        },
    "action": {
            "button": modal_title,
    "sections": [
    {
        "title": "Marca",
    "rows": make_json
    }
    ]
        }
    }
    }))?
        .into_string()?;

    Ok(())
}

fn execute_second_step(phone_number: &str) -> Result<(), Box<dyn Error>> {

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!({
    "messaging_product": "whatsapp",
    "recipient_type": "individual",
    "to": phone_number,
    "type": "template",
    "template": {
        "name": "solicitud_vin_2",
    "language": {
            "code": "es"
        }
    }
    }))?
        .into_string()?;

    println!("{}", &resp);

    Ok(())
}

pub(crate) fn successfull_request(phone_number: &str) -> Result<(), Box<dyn Error>> {

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!(
            {
                "preview_url": false,
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": phone_number,
                "type": "text",
                "text": {
                    "body": "Solicitud de repuesto realizada, le estaremos contactando a la brevedad si contamos con el repuesto, recuerde pedir los datos de contacto del cliente para poder contactarlo posteriormente.  Si desea iniciar una nueva solicitud escriba 'Hola' en el chat."
                }
            }))?
        .into_string()?;

    Ok(())
}


pub(crate) fn request_part_description(phone_number: &str) -> Result<(), Box<dyn Error>> {

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!(
            {
                "preview_url": false,
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": phone_number,
                "type": "text",
                "text": {
                    "body": "Describa el repuesto que necesita, provea la mayor cantidad de detalles que permitan distinguir el repuesto."
                }
            }))?
        .into_string()?;

    println!("{}", &resp);

    Ok(())
}

pub(crate) fn new_request(phone_number: &str) -> Result<(), Box<dyn Error>> {

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!(
            {
                "preview_url": false,
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": phone_number,
                "type": "text",
                "text": {
                    "body": "Si desea"
                }
            }))?
        .into_string()?;

    println!("{}", &resp);

    Ok(())
}

pub fn send_error_message(error_message: impl Into<String>, phone_number: &str) -> Result<(), Box<dyn Error>>{
    let resp: String = ureq::post("https://graph.facebook.com/v15.0/109252135350289/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!(
            {
                "preview_url": false,
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": phone_number,
                "type": "text",
                "text": {
                    "body": error_message.into()
                }
            }))?
        .into_string()?;

    println!("{}", &resp);

    Ok(())
}