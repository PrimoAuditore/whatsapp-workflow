extern crate redis;

use std::collections::HashMap;
use std::env::Args;
use std::error::Error;
use std::process::Command;

use aws_config::SdkConfig;
use redis::Commands;
use uuid::Uuid;
use crate::constants::MessageType;
use crate::s3_tools;

use crate::structs::{Event, ListChoice, MediaData, MessageRequest, StandardResponse};

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


pub(crate) fn validate_vin(vin: String) -> bool {
    let translitering_chart: HashMap<&str, u32> = HashMap::from([
        ("A", 1),
        ("B", 2),
        ("C", 3),
        ("D", 4),
        ("E", 5),
        ("F", 6),
        ("G", 7),
        ("H", 8),
        ("J", 1),
        ("K", 2),
        ("L", 3),
        ("M", 4),
        ("N", 5),
        ("P", 7),
        ("R", 9),
        ("S", 2),
        ("T", 3),
        ("U", 4),
        ("V", 5),
        ("W", 6),
        ("X", 7),
        ("Y", 8),
        ("Z", 9),
    ]);

    let valid_size: bool = usize::try_from(17)
        .map(|size| &vin.chars().count() == &size)
        .unwrap_or(false);

    if valid_size {
        let mut transliterated_vin: Vec<u32> = vec![];

        for char in vin.chars() {
            if char.is_alphabetic() {
                let digit = translitering_chart.get(char.to_string().as_str());

                match digit {
                    Some(x) => transliterated_vin.push(x.clone()),
                    None => panic!("Digit not found in transliteration table"),
                }
            } else {
                transliterated_vin.push(char.to_string().parse::<u32>().unwrap());
            }
        }

        let mut weight = 8;
        let mut result = 0;
        for digit in transliterated_vin {
            result += digit * weight;

            if weight == 2 {
                weight = 10;
            } else if weight == 10 {
                weight = 0;
            } else if weight == 0 {
                weight = 9;
            } else {
                weight -= 1;
            }
        }

        let result_digit = result % 11;

        if result_digit == 10 && vin.chars().nth(8).unwrap().to_string() == "X" {
            return true;
        } else if vin
            .chars()
            .nth(9)
            .unwrap()
            .to_string()
            .parse::<u32>()
            .unwrap_or(10)
            == result_digit
        {
            return true;
        } else {
            return false;
        }
    } else {
        return false;
    }
}

pub fn get_brand_models(
    brand: &str,
    page: i32,
    brand_selected: &str,
) -> Result<Vec<ListChoice>, Box<dyn Error>> {
    let client = redis::Client::open(std::env::var("REDIS_URL").unwrap())?;

    let mut con = client.get_connection().expect("conn");

    let start: isize = if page == 1 {
        0
    } else {
        9 * (page - 1) as isize
    };

    let stop: isize = if page == 1 { 8 } else { (9 * page) - 1 } as isize;

    let model_input: Vec<String> = con
        .lrange(format!("models:{}", brand), start, stop)
        .expect("Error");

    let mut models_list = vec![];

    for model in model_input {
        let mut model_object = ListChoice::new();

        model_object.id = format!("{}-{}", brand, model);
        model_object.title = model[0..1].to_uppercase() + &model[1..];

        models_list.push(model_object);
    }

    if models_list.len() == 9 {
        models_list.push(ListChoice {
            title: "Mas modelos".to_string(),
            id: format!("page-{}-{}", page + 1, brand_selected).to_string(),
        })
    }

    Ok(models_list)
}

pub fn get_makes(page: i32) -> Result<Vec<ListChoice>, Box<dyn Error>> {
    let client = redis::Client::open(std::env::var("REDIS_URL").unwrap())?;

    let mut con = client.get_connection().expect("conn");

    let start: isize = if page == 1 {
        0
    } else {
        9 * (page - 1) as isize
    };

    let stop: isize = if page == 1 { 8 } else { (9 * page) - 1 } as isize;

    let makes_input: Vec<String> = con.lrange("makes", start, stop).expect("Error");

    let mut makes_list = vec![];

    for make in makes_input {
        let mut model_object = ListChoice::new();

        model_object.id = make.clone().to_ascii_lowercase();
        model_object.title = make.clone()[0..1].to_uppercase() + &make[1..];

        makes_list.push(model_object);
    }

    if makes_list.len() == 9 {
        makes_list.push(ListChoice {
            title: "Mas marcas".to_string(),
            id: format!("page-{}", page + 1).to_string(),
        })
    }

    Ok(makes_list)
}

fn download_image(image_url: &str, image_name: &str) {
    let output = Command::new("bash")
        .arg("download-image.sh")
        .arg(image_url)
        .arg(image_name)
        .arg(std::env::var("META_TOKEN").unwrap())
        .output()
        .expect("Failed to execute command");

    debug!(
        "Command stdout: {}",
        String::from_utf8(output.stdout).unwrap()
    );
    error!(
        "Command stderr{}",
        String::from_utf8(output.stderr).unwrap()
    );
}

fn get_image_url(image_id: &str) -> MediaData {
    let request = get_media_url(image_id);

    match request {
        Ok(media) => media,
        Err(e) => panic!("Error obtaining image url: {}", e),
    }
}

pub async fn upload_image(
    image_id: String,
) -> Result<String, Box<dyn Error>> {
    let config = s3_tools::create_config().await;

    if config.is_none() {
        panic!("AWS config not available")
    }


    let image_name = format!("{}.jpeg", Uuid::new_v4().to_string());

    debug!("Obtaining image url");
    let media_data = get_image_url(image_id.as_str());

    debug!("Downloading image");
    download_image(media_data.url.as_str(), image_name.as_str());

    debug!("uploading image to S3");

    s3_tools::upload_image_to_s3(image_name.as_str(), config.unwrap()).await?;

    Ok(image_name)
}


pub fn get_message_content(event: &Event) -> String {
    let message = event.entry[0].changes[0].value.messages.as_ref().unwrap();
    match message[0].message_type.as_str() {
        "text" => {
            message[0]
                .clone()
                .text
                .unwrap()
                .body
        }
        "interactive" =>{
            if message[0].interactive.as_ref().unwrap().button_reply.as_ref().is_some(){
                &message[0].interactive.as_ref().unwrap().button_reply.as_ref().unwrap().id
            } else if message[0].interactive.as_ref().unwrap().list_reply.as_ref().is_some(){
                &message[0].interactive.as_ref().unwrap().list_reply.as_ref().unwrap().id
            }else{
                panic!("message type no supported")
            }.to_string()
        },
        "image" => {
            message[0]
                .clone()
                .image
                .unwrap()
                .caption
        }
        _ => {
            panic!("Message type not supported")
        }
    }.to_string()
}

pub fn find_message_type(event: &Event) -> MessageType {
    let message = event.entry[0].changes[0].value.messages.as_ref().unwrap();
    match message[0].message_type.as_str() {
        "text" => {
            MessageType::PlainText

        },
        "interactive" =>{
            if message[0].interactive.as_ref().unwrap().button_reply.is_some(){
                MessageType::ButtonSelection
            } else if message[0].interactive.as_ref().unwrap().list_reply.is_some(){
                MessageType::ListSelection
            }else{
                panic!("message type no supported")
            }
        },
        "image" => {
            MessageType::PlainTextAndImage
        }
        _ => {
            panic!("Message type not supported: {}", message[0].message_type.as_str())
        }
    }
}

pub fn send_message(message: MessageRequest) -> Result<StandardResponse, String> {

    debug!("Sending message with payload: \n {}", ureq::json!(message));
    let url = std::env::var("WHATSAPP_MANAGER_HOST").unwrap();

    let resp = ureq::post(format!("{}/message", url).as_str())
        .set(
            "Authorization",
            format!("Bearer {}", std::env::var("META_TOKEN").unwrap()).as_str(),
        )
        .send_json(ureq::json!(message)).unwrap()
        .into_string();

    if resp.is_err() {
        return Err(resp.unwrap_err().to_string())
    }

    let parsed_response: StandardResponse = serde_json::from_str(&resp.unwrap()).unwrap();

    Ok(parsed_response)
}

