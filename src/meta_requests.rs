use crate::structs::{ListChoice, MediaData};
use crate::tools::{get_brand_models, get_makes};
use image::{DynamicImage, ImageFormat, ImageResult};
use std::error::Error;
use std::process::Command;
use ureq::Response;

pub(crate) fn service_message(phone_number: &str) -> Result<(), Box<dyn Error>> {
    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
        .send_json(ureq::json!({
  "messaging_product": "whatsapp",
  "recipient_type": "individual",
  "to": phone_number,
  "type": "interactive",
  "interactive": {
    "type": "button",
    "body": {
      "text": "Selecciona el servicio requerido."
    },
    "action": {
      "buttons": [
        {
          "type": "reply",
          "reply": {
            "id": "part-search",
            "title": "Busqueda de repuesto"
          }
        },
        {
          "type": "reply",
          "reply": {
            "id": "help",
            "title": "Ayuda"
          }
        }
      ]
    }
  }
}))?
        .into_string()
        .unwrap();
    Ok(())
}

pub(crate) fn get_media_url(media_id: &str) -> Result<MediaData, Box<dyn Error>> {
    let resp: String = ureq::get(format!("https://graph.facebook.com/v15.0/{}", media_id).as_str())
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
        .call()?
        .into_string()
        .unwrap();

    let media_data: MediaData = serde_json::from_str(resp.as_str()).unwrap();
    Ok(media_data)
}

pub(crate) fn get_image(image_url: &str) -> Result<(), Box<dyn Error>> {

    let output = Command::new("curl")
        .arg("-X GET")
        .arg(format!("'{}'", image_url))
        .arg(format!("-H 'Authorization: Bearer {}'",std::env::var("META_TOKEN").unwrap()).as_str())
        .arg("> media_file.jpeg.jpeg")
        .output()
        .expect("Failed to execute command");

    Ok(())
}

pub(crate) fn help_request(phone_number: &str) -> Result<(), Box<dyn Error>> {

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
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
        }))?
        .into_string()
        .unwrap();
    Ok(())
}

pub(crate) fn request_vin(phone_number: &str) -> Result<(), Box<dyn Error>> {
    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
        .send_json(ureq::json!({
      "messaging_product": "whatsapp",
      "recipient_type": "individual",
      "to": phone_number,
      "type": "text",
      "text": {
        "preview_url": false,
        "body": "Ingrese *VIN* del vehículo."
        }
    }))?
        .into_string()?;

    Ok(())
}

pub(crate) fn model_list(
    phone_number: &str,
    brand_selected: String,
    page: i32,
) -> Result<(), Box<dyn Error>> {
    let models_list = get_brand_models(
        brand_selected.to_ascii_lowercase().as_str(),
        page,
        brand_selected.as_str(),
    );
    let modal_title = format!(
        "Modelos {} - {}",
        brand_selected[0..1].to_uppercase() + &brand_selected[1..],
        page
    );

    if models_list.is_err() {
        panic!("Error obtaining model list");
    }

    let model_json = serde_json::to_string(&models_list.unwrap()).unwrap();

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
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

    Ok(())
}
pub(crate) fn brand_list(phone_number: &str, page: i32) -> Result<(), Box<dyn Error>> {
    let make_list = get_makes(page);
    let modal_title = format!("Marcas - pagina {}", page);

    if make_list.is_err() {
        panic!("Error obtaining model list");
    }

    let make_json = serde_json::to_string(&make_list.unwrap()).unwrap();

    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
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


pub(crate) fn successfull_request(phone_number: &str) -> Result<(), Box<dyn Error>> {
    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
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
    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
        .set("Authorization", format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str())
        .send_json(ureq::json!(
            {
                "preview_url": false,
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": phone_number,
                "type": "text",
                "text": {
                    "body": "Describa el repuesto que necesita, provea la mayor cantidad de detalles que permitan distinguir el repuesto, puedes adjuntar una imagen en el mismo mensaje."
                }
            }))?
        .into_string()?;

    Ok(())
}

pub(crate) fn new_request(phone_number: &str) -> Result<(), Box<dyn Error>> {
    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
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

    Ok(())
}

pub fn send_error_message(
    error_message: impl Into<String>,
    phone_number: &str,
) -> Result<(), Box<dyn Error>> {
    let resp: String = ureq::post("https://graph.facebook.com/v15.0/110000391967238/messages")
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
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

    Ok(())
}
