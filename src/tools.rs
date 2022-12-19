use redis::streams::{
    StreamId, StreamKey, StreamMaxlen, StreamRangeReply, StreamReadOptions, StreamReadReply,
};
use std::collections::HashMap;
extern crate redis;
use crate::meta_requests::get_media_url;
use crate::structs::{ListChoice, MediaData};
use crate::{meta_requests, s3_tools};
use actix_web::cookie::time;
use actix_web::cookie::time::macros::offset;
use actix_web::cookie::time::{OffsetDateTime, UtcOffset};
use aws_config::SdkConfig;
use redis::{Commands, Value};
use std::error::Error;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub enum FlowStatus {
    FlowStarted = 1,
    ServiceModalSent = 2,
    ServiceSelected = 3,
    BrandModalSent = 4,
    BrandSelected = 5,
    ModelModalSent = 6,
    ModelSelected = 7,
    VinRequestSent = 8,
    VinProvided = 9,
    PartDescriptionRequested = 10,
    PartDescriptionProvided = 11,
    RequestAccepted = 12,
    PartFound = 13,
    RequestCanceled = 14,
}

impl FlowStatus {
    pub fn get_from_value(i: &String) -> FlowStatus {
        let status_id = i.parse().unwrap();
        match status_id {
            1 => FlowStatus::FlowStarted,
            2 => FlowStatus::ServiceModalSent,
            3 => FlowStatus::ServiceSelected,
            4 => FlowStatus::BrandModalSent,
            5 => FlowStatus::BrandSelected,
            6 => FlowStatus::ModelModalSent,
            7 => FlowStatus::ModelSelected,
            8 => FlowStatus::VinRequestSent,
            9 => FlowStatus::VinProvided,
            10 => FlowStatus::PartDescriptionRequested,
            11 => FlowStatus::PartDescriptionProvided,
            12 => FlowStatus::RequestAccepted,
            13 => FlowStatus::PartFound,
            14 => FlowStatus::RequestCanceled,
            _ => panic!("Value not found"),
        }
    }
}

fn create_client() -> Result<redis::Connection, Box<dyn Error>> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;

    Ok(con)
}

pub fn create_request_tracker(stream_name: &str, track_id: &str) -> Result<String, Box<dyn Error>> {
    let client = redis::Client::open(std::env::var("REDIS_URL").unwrap())?;
    let mut con = client.get_connection()?;

    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis().to_string(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let res = redis::cmd("XADD")
        .arg(stream_name)
        .arg("*")
        .arg("track-id")
        .arg(track_id)
        .arg("timestamp")
        .arg(timestamp)
        .arg("status-id")
        .arg(FlowStatus::FlowStarted as i32)
        .arg("value")
        .arg("")
        .query(&mut con)?;

    // TODO: Manage error from register creation
    Ok("ok".to_string())
}

pub fn check_registry_expiry(
    current_status: &FlowStatus,
    client_last_event: &Result<FlowRegister, Box<dyn Error>>,
) -> Result<bool, Box<dyn Error>> {
    let time_as_integer = &client_last_event
        .as_ref()
        .unwrap()
        .timestamp
        .as_ref()
        .unwrap()
        .parse::<i64>()
        .unwrap();
    let parsed_time =
        OffsetDateTime::from_unix_timestamp(*time_as_integer / 1000)?.to_offset(offset!(-3));
    let time_difference = SystemTime::now().duration_since(SystemTime::from(parsed_time));

    // If last register has more than 3 hours
    if time_difference.unwrap().as_secs() > 10800 {
        match current_status {
            FlowStatus::FlowStarted
            | FlowStatus::ServiceModalSent
            | FlowStatus::ServiceSelected
            | FlowStatus::BrandModalSent
            | FlowStatus::BrandSelected
            | FlowStatus::ModelModalSent
            | FlowStatus::ModelSelected
            | FlowStatus::VinRequestSent
            | FlowStatus::VinProvided
            | FlowStatus::PartDescriptionRequested
            | FlowStatus::PartDescriptionProvided => {
                return Ok(false);
            }
            _ => return Ok(true),
        }
    };

    Ok(true)
}

pub fn update_flow_status(
    stream_name: &str,
    last_register: &str,
    updated_status: FlowStatus,
    value: Option<String>,
    attached_file: Option<String>,
) -> Result<String, Box<dyn Error>> {
    let client = redis::Client::open(std::env::var("REDIS_URL").unwrap())?;
    let mut con = client.get_connection()?;

    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis().to_string(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let res = redis::cmd("XADD")
        .arg(stream_name)
        .arg("*")
        .arg("track-id")
        .arg(&last_register)
        .arg("timestamp")
        .arg(timestamp)
        .arg("status-id")
        .arg(updated_status as i32)
        .arg("value")
        .arg(value.unwrap_or("".to_string()))
        .arg("attached_file")
        .arg(attached_file.unwrap_or("".to_string()))
        .query(&mut con)?;

    Ok("ok".to_string())
}

pub fn get_last_event(stream_name: &str) -> Result<FlowRegister, Box<dyn Error>> {
    let client = redis::Client::open(std::env::var("REDIS_URL").unwrap())?;

    let mut con = client.get_connection().expect("conn");

    let srr: StreamRangeReply = con
        .xrevrange_count(&stream_name, "+", "-", 1)
        .expect("read");

    if srr.ids.len() > 1 {
        panic!("Expected 1 value, received more than one");
    } else if srr.ids.len() == 0 {
        return Err("Error".into());
    }

    let flow_register = FlowRegisterBuilder::new()
        .tracker_id(parse_value_bytes(srr.ids[0].map.get("track-id").unwrap().clone()).unwrap())
        .timestamp(parse_value_bytes(srr.ids[0].map.get("timestamp").unwrap().clone()).unwrap())
        .status_id(parse_value_bytes(srr.ids[0].map.get("status-id").unwrap().clone()).unwrap())
        .build();

    Ok(flow_register.unwrap())
}

fn parse_value_bytes(value: Value) -> Result<String, &'static str> {
    let val = if let Value::Data(bytes) = value {
        let parsed_string: String = String::from_utf8(bytes).expect("utf8");
        Ok(parsed_string)
    } else {
        Err("weird data")
    };

    val
}

pub struct FlowRegister {
    pub tracker_id: Option<String>,
    pub timestamp: Option<String>,
    pub status_id: Option<String>,
    pub value: Option<String>,
}

#[derive(Default)]
struct FlowRegisterBuilder {
    tracker_id: Option<String>,
    timestamp: Option<String>,
    status_id: Option<String>,
    value: Option<String>,
}

impl FlowRegisterBuilder {
    pub fn new() -> FlowRegisterBuilder {
        FlowRegisterBuilder::default()
    }

    pub fn tracker_id(&mut self, tracker_id: impl Into<String>) -> &mut Self {
        self.tracker_id = Some(tracker_id.into());
        self
    }

    pub fn timestamp(&mut self, timestamp: impl Into<String>) -> &mut Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    pub fn status_id(&mut self, status_id: impl Into<String>) -> &mut Self {
        self.status_id = Some(status_id.into());
        self
    }

    pub fn value(&mut self, value: impl Into<String>) -> &mut Self {
        self.value = Some(value.into());
        self
    }

    pub fn build(&self) -> Result<FlowRegister, Box<dyn Error>> {
        Ok(FlowRegister {
            tracker_id: self.tracker_id.clone(),
            timestamp: self.timestamp.clone(),
            status_id: self.status_id.clone(),
            value: self.value.clone(),
        })
    }
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

    println!("{}", String::from_utf8(output.stdout).unwrap());
    println!("{}", String::from_utf8(output.stderr).unwrap());
}

fn get_image_url(image_id: &str) -> MediaData {
    let request = meta_requests::get_media_url(image_id);

    match request {
        Ok(media) => media,
        Err(e) => panic!("Error obtaining image url: {}", e),
    }
}

pub async fn upload_image(
    image_id: String,
    config: &Option<SdkConfig>,
) -> Result<String, Box<dyn Error>> {
    let image_name = format!("{}.jpeg", Uuid::new_v4().to_string());

    println!("Obtaining image url");
    let media_data = get_image_url(image_id.as_str());

    println!("Downloading image");
    download_image(media_data.url.as_str(), image_name.as_str());

    println!("uploading image to S3");

    s3_tools::upload_image(image_name.as_str(), config).await?;

    Ok(image_name)
}
