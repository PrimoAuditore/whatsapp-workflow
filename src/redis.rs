use std::collections::HashMap;
use std::error::Error;
use crate::structs::{Event, MessageLog, RequestTracker, TrackerStep};
use redis::{Client, Commands, JsonCommands, RedisError, RedisResult};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_user_mode(phone_number: &str) -> Result<u16, RedisError> {
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    let mode: RedisResult<String> = con.hget(format!("selected-mode:{}", phone_number), "mode");

    if mode.is_err() {
        return Err(mode.unwrap_err())
    }

    let parsed_mode = mode.unwrap().parse::<u16>().unwrap();

    Ok(parsed_mode)
}

fn create_client() -> Result<Client, RedisError> {
    let url = std::env::var("REDIS_URL").unwrap();
    let client = redis::Client::open(url);

    return match client {
        Ok(client) => unsafe { Ok(client) },
        Err(err) => Err(err),
    };
}

pub fn get_user_message(message_id: &str, phone_number: &str) -> Result<Event, RedisError> {
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    info!("Searching message: {}", format!("incoming-messages:{}:{}", phone_number, message_id));

    let res: RedisResult<String> = con
        .json_get(
            format!("incoming-messages:{}:{}", phone_number, message_id),
            ".",
        );

    if res.is_err() {
        error!("Error getting user message: {}", res.as_ref().unwrap_err());

        if is_nil(&res.as_ref().unwrap_err()) {
            error!("Error getting user message(Response is NIL): {}", res.as_ref().unwrap_err());
        }

        return Err(res.unwrap_err());
    }

    let event: Event = serde_json::from_str(&res.unwrap()).unwrap();

    Ok(event)
}

pub fn create_new_tracker(tracker_id: &str, phone_number: &str) -> Result<String, RedisError> {
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    // Get timestamp
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis().to_string(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    // Create registry
    let res: Result<String, RedisError> = con.hset_multiple(
        format!("whatsapp-request:{}", tracker_id),
        &[("phone_number", phone_number), ("timestamp", &timestamp)],
    );

    if res.is_err() {
        return Err(res.unwrap_err());
    }

    return Ok(format!("whatsapp-request:{}", tracker_id));
}

pub fn is_nil(error: &RedisError) -> bool {
    error.to_string().contains("response was nil")
}

pub fn get_last_tracker(phone_number: &str) -> Result<RequestTracker, String>{
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    let res:RedisResult<Vec<(u32, String, Vec<String>)>> = redis::cmd("FT.SEARCH")
        .arg("userTrackers")
        .arg(phone_number)
        .arg("SORTBY")
        .arg("timestamp")
        .arg("DESC")
        .arg("LIMIT")
        .arg("0")
        .arg("1")
        .query(&mut con);


    if res.is_err() {

        if res.as_ref().unwrap_err().to_string().contains("response was [int(0)]") {
            // No record found
            return Err("No records found".to_string())
        }else{
            // Any other error
            return Err(res.unwrap_err().to_string())
        }
    }
    let register = res.unwrap();
   let mut requestTracker: RequestTracker = RequestTracker{
       phone_number: "".to_string(),
       timestamp: "".to_string(),
       id: String::from(&register[0].1.replace("whatsapp-request:", "")),
   };

    let mut params: HashMap<String, String> = HashMap::new();
    let mut param_name = "";
    for (index, elem) in register[0].2.iter().enumerate() {
        // It's a parameter name
        if index % 2 == 0 {
            param_name = elem;
        }else{
            params.insert(param_name.to_string(), elem.to_string());
        }
    }

    requestTracker.timestamp = params.get("timestamp").expect("timestamp param couldnt be found").clone();
    requestTracker.phone_number = params.get("phone_number").expect("phone_number param couldnt be found").clone();

    Ok(requestTracker)
}


pub fn get_last_tracker_step(tracker_id: &str) -> Result<TrackerStep, String>{
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    let res:RedisResult<Vec<(u32, String, Vec<String>)>> = redis::cmd("FT.SEARCH")
        .arg("trackerSteps")
        .arg(tracker_id)
        .arg("SORTBY")
        .arg("timestamp")
        .arg("DESC")
        .arg("LIMIT")
        .arg("0")
        .arg("1")
        .query(&mut con);


    if res.is_err() {

        if res.as_ref().unwrap_err().to_string().contains("response was [int(0)]") {
            // No record found
            return Err("No records found".to_string())
        }else{
            // Any other error
            return Err(res.unwrap_err().to_string())
        }
    }
    let register = res.unwrap();
    let mut trackerStep : TrackerStep = TrackerStep{
        tracker_id: "".to_string(),
        timestamp: "".to_string(),
        id: String::from(&register[0].1),
        status: "".to_string(),
        value: "".to_string(),
        attached_files: "".to_string(),
        message_reference: "".to_string(),
    };

    let mut params: HashMap<String, String> = HashMap::new();
    let mut param_name = "";
    for (index, elem) in register[0].2.iter().enumerate() {
        // It's a parameter name
        if index % 2 == 0 {
            param_name = elem;
        }else{
            params.insert(param_name.to_string(), elem.to_string());
        }
    }

    // Index is the second item(index 1) on tuple
    trackerStep.id = String::from(&register[0].1);

    trackerStep.timestamp = params.get("timestamp").expect("timestamp param couldnt be found").clone();
    trackerStep.tracker_id = params.get("tracker_id").expect("tracker_id param couldnt be found").clone();
    trackerStep.status = params.get("status").expect("status param couldnt be found").clone();
    trackerStep.value = params.get("value").expect("value param couldnt be found").clone();
    trackerStep.attached_files = params.get("attached_files").expect("attached_files param couldnt be found").clone();
    trackerStep.message_reference = params.get("message_reference").expect("message_reference param couldnt be found").clone();

    println!("{:?}", trackerStep);

    Ok(trackerStep)
}


pub fn create_new_step(step: &TrackerStep) -> Result<String, RedisError> {
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    // Get timestamp
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis().to_string(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    // Create registry
    let step_clone = step.clone();
    let res: Result<String, RedisError> = con.hset_multiple(
        format!("whatsapp-workflow:{}", &step.id),
        &[
            ("tracker_id", step_clone.clone().tracker_id),
            ("timestamp", timestamp),
            ("status", step_clone.status),
            ("value", step_clone.value),
            ("attached_files", step_clone.attached_files),
            ("message_reference", step_clone.message_reference),
        ],
    );

    if res.is_err() {
        return Err(res.unwrap_err());
    }

    return Ok(format!("whatsapp-workflow:{}", &step.id));
}

pub fn get_list(key: String)-> Result<Vec<String>, RedisError>{
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    let list_res:RedisResult<Vec<String>> = con.lrange(key, 0, 1000);

    if list_res.is_err() {
        return Err(list_res.unwrap_err())
    }

    Ok(list_res.unwrap())
}

pub fn publish_message(
    message: &MessageLog,
    phone_number: &String,
) -> Result<String, Box<dyn Error>> {
    let client = create_client()?;
    let mut con = client.get_connection()?;
    let _: () = con
        .publish(
            format!("whatsapp-notification:{}", phone_number),
            serde_json::to_string(message).unwrap(),
        )
        .expect("err");

    Ok("OK".to_string())
}


pub fn reset_user_mode(phone_number:&str) -> Result<(), String>{
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    let res: RedisResult<String> = con.hset(format!("selected-mode:{}", phone_number), "mode", "100");

    if res.is_err() {
        error!("Error reseting user mode: {}", res.as_ref().unwrap_err());
        return Err(format!("Error reseting user mode: {}", res.as_ref().unwrap_err()))
    }

    Ok(())
}
pub fn get_step_by_status(tracker_id: &str, status: &str) -> Result<TrackerStep, String>{
    let client = create_client().unwrap();
    let mut con = client.get_connection().unwrap();

    let res:RedisResult<Vec<(u32, String, Vec<String>)>> = redis::cmd("FT.SEARCH")
        .arg("trackerSteps")
        .arg(format!("@tracker_id:{} @status:{}", tracker_id, status))
        .query(&mut con);


    if res.is_err() {

        if res.as_ref().unwrap_err().to_string().contains("response was [int(0)]") {
            // No record found
            return Err("No records found".to_string())
        }else{
            // Any other error
            return Err(res.unwrap_err().to_string())
        }
    }
    let register = res.unwrap();
    let mut trackerStep : TrackerStep = TrackerStep{
        tracker_id: "".to_string(),
        timestamp: "".to_string(),
        id: String::from(&register[0].1),
        status: "".to_string(),
        value: "".to_string(),
        attached_files: "".to_string(),
        message_reference: "".to_string(),
    };

    let mut params: HashMap<String, String> = HashMap::new();
    let mut param_name = "";
    for (index, elem) in register[0].2.iter().enumerate() {
        // It's a parameter name
        if index % 2 == 0 {
            param_name = elem;
        }else{
            params.insert(param_name.to_string(), elem.to_string());
        }
    }

    // Index is the second item(index 1) on tuple
    trackerStep.id = String::from(&register[0].1);

    trackerStep.timestamp = params.get("timestamp").expect("timestamp param couldnt be found").clone();
    trackerStep.tracker_id = params.get("tracker_id").expect("tracker_id param couldnt be found").clone();
    trackerStep.status = params.get("status").expect("status param couldnt be found").clone();
    trackerStep.value = params.get("value").expect("value param couldnt be found").clone();
    trackerStep.attached_files = params.get("attached_files").expect("attached_files param couldnt be found").clone();
    trackerStep.message_reference = params.get("message_reference").expect("message_reference param couldnt be found").clone();

    println!("{:?}", trackerStep);

    Ok(trackerStep)
}

