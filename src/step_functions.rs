use std::collections::HashMap;
use std::env::Args;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::ser::Error;
use serde_json::to_string;
use uuid::Uuid;
use crate::constants::*;
use crate::constants::FlowStatusId::*;
use crate::constants::MessageType::*;
use crate::redis::{get_list, get_step_by_status, get_user_message, reset_user_mode};
use crate::structs::{MessageContent, MessageLog, MessageRequest, RequestTracker, StepDefinition, TrackerStep};
use crate::tools::{send_message, upload_image, validate_vin};

pub async fn execute_function(new_step: &mut TrackerStep, status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    info!("Executing function for status: {}", status as u16);

    let res: Result<MessageRequest, String> = match FlowStatus::get_from_value(&(status as u16).to_string()) {
        // STEPS DEFINITION

        FlowStatus::FlowStarted => {
            todo!()
        }
        FlowStatus::BrandModalSent => {
            brand_modal_sent(&new_step, status, log, message_content)
        }
        FlowStatus::BrandSelected => {
            brand_selected(&new_step, status, log, message_content)
        }
        FlowStatus::ModelModalSent => {
            model_modal_sent(&new_step, status, log, message_content)
        }
        FlowStatus::ModelSelected => {
            model_selected(&new_step, status, log, message_content)
        }
        FlowStatus::IdentificationRequestSent => {
            identification_request_sent(&new_step, status, log, message_content)
        }
        FlowStatus::IdentificationProvided => {
            identification_provided(&new_step, status, log, message_content)
        }
        FlowStatus::PartDescriptionRequested => {
            description_requested(&new_step, status, log, message_content)
        }
        FlowStatus::PartDescriptionProvided => {
            description_provided(new_step, status, log, message_content).await
        }
        FlowStatus::RequestAccepted => {
            request_accepted(&new_step, status, log, message_content)
        }
    };

    if res.is_err() {
        // IMPLEMENT ERROR message
        unimplemented!()
    }




    Ok(res.unwrap())

}


pub fn brand_modal_sent(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>  {
    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    // retrieve data is required from data origin

    let mut list: Option<Vec<String>> = None;
    if status.value().data_origin.is_some() {
        debug!("Found data origin for step");

        debug!("Obtaining list choices");
        let list_res = get_list(status.value().data_origin.unwrap());

        if list_res.is_err() {

            return Err(list_res.unwrap_err().to_string());

        }

        list = Some(list_res.unwrap());
    }

    // check what is the next step expected response type
    // let expected_response_id = step_definition.next_step.unwrap();
    // let expected_response = FlowStatus::get_from_value(&(expected_response_id as u16).to_string());

    // Add choices to message in case of list of buttons messages
    debug!("Adding choices to message response");
    if message_request.message_type == "list".to_string() && list.is_some()
    {
        for choice in list.unwrap(){
            message_request.content.list.as_mut().unwrap().choices.push(choice);
        }
    }

     // Adding to
    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}


pub fn model_modal_sent(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>  {
    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    // Get make response from user, 3 is the
    let step_id = (FlowStatusId::BrandSelectedId as u16).to_string();
    let make_step = get_step_by_status(&step.tracker_id, &step_id);

    if make_step.is_err() {
        error!("Error obtaining brand selection step: {}", make_step.as_ref().unwrap_err().to_string());
        return Err(format!("Error obtaining brand selection step: {}", make_step.as_ref().unwrap_err().to_string()));
    }

    // Parsed brand selection into data origin for list
    let brand_selection = String::from(make_step.unwrap().value.split("-").collect::<Vec<&str>>()[0]);
    let data_origin = status.value().data_origin.unwrap().replace("{}", &brand_selection);

    // retrieve data is required from data origin

    let mut list: Option<Vec<String>> = None;
    if status.value().data_origin.is_some() {
        debug!("Found data origin for step");

        debug!("Obtaining list choices");
        let list_res = get_list(data_origin);

        if list_res.is_err() {

            return Err(list_res.unwrap_err().to_string());

        }

        println!("{:?}", list_res.as_ref().unwrap());
        list = Some(list_res.unwrap());
    }

    // check what is the next step expected response type
    // let expected_response_id = step_definition.next_step.unwrap();
    // let expected_response = FlowStatus::get_from_value(&(expected_response_id as u16).to_string());

    // Add choices to message in case of list of buttons messages
    debug!("Adding choices to message response");
    if message_request.message_type == "list".to_string() && list.is_some()
    {
        for choice in list.unwrap(){
            message_request.content.list.as_mut().unwrap().choices.push(choice);
        }
    }



    info!("sending message {}", serde_json::to_string_pretty(&message_request).unwrap());

    // Adding to
    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}


fn brand_selected(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}

fn description_requested(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}


async fn description_provided(step: &mut TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    let mut message_request = status.value().successful_response.unwrap();
    message_request.to.push(log.clone().phone_number);


    let message = get_user_message(&log.register_id, &log.phone_number);

    if message.is_err() {
        error!("Error obtaining user message");
        return Err("Error obtaining user message".to_string())
    }

    // Check if message have attached files
    let event = message.unwrap();

    // If there's an image attached to message
    if event.entry[0].changes[0].value.messages.as_ref().unwrap()[0].image.is_some() {
        let image_id = String::from(&event.entry[0].changes[0].value.messages.as_ref().unwrap()[0].image.as_ref().unwrap().id);
        let upload_res = upload_image(image_id).await;

        if upload_res.is_err() {
            error!("Error uploading file to S3: {}", upload_res.as_ref().unwrap_err())
        };

        step.attached_files = upload_res.unwrap();
    }

    Ok(message_request)
}

fn identification_provided(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}

fn model_selected(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}


fn request_accepted(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    message_request.to.push(log.clone().phone_number);

    // Reset user mode selection
    let res = reset_user_mode(&log.phone_number);

    if res.is_err() {
        error!("Failed to reset user mode");
        return Err(res.unwrap_err())
    }

    Ok(message_request)
}

fn identification_request_sent(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    // Set phone number
    message_request.to.push(log.clone().phone_number);

    // determine if vin or patent was matched
    info!("validating vin: {}", message_content.to_string());
    let is_valid = validate_vin(message_content.to_string());

    if !is_valid {
        error!("Provided VIN is not valid");

        // Send error message
        let error_message = MessageRequest{
            system_id: 3,
            to: vec![log.phone_number.clone()],
            message_type: "text".to_string(),
            content: MessageContent {
                body: Some("VIN ingresado no es valido, verifique y reintente.".to_string()),
                list: None,
                buttons: None,
            },
        };

        let error_res = send_message(message_request);

        // If sending error message failed
        if error_res.is_err(){
            error!("Provided VIN is not valid, Error message wasn't sent: {}", error_res.as_ref().unwrap_err());
            return Err(format!("Error message wasn't sent: {}", error_res.as_ref().unwrap_err()))
        }

        // If vin is invalid
        return Err("Provided VIN is not valid".to_string())
    }

    Ok(message_request)
}