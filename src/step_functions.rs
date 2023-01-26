use std::collections::HashMap;
use std::env::Args;
use std::time::{SystemTime, UNIX_EPOCH};
use fizzy_commons::shared_structs::{Choice, MessageContent, MessageRequest};
use serde::de::Unexpected::Str;
use serde::ser::Error;
use serde_json::to_string;
use uuid::Uuid;
use crate::constants::*;
use crate::constants::FlowStatusId::*;
use crate::constants::MessageType::*;
use crate::redis::{get_list, get_list_size, get_step_by_status, get_user_message, reset_user_mode};
use crate::structs::{MessageLog, RequestTracker, StepDefinition, TrackerStep};
use crate::tools::{send_message, upload_image, validate_vin};

pub async fn execute_function(new_step: &mut TrackerStep, status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    info!("Executing function for status: {}", status as u16);

    let res: Result<MessageRequest, String> = match FlowStatus::get_from_value(&(status as u16).to_string()) {
        // STEPS DEFINITION

        FlowStatus::FlowStarted => {
            todo!()
        }
        FlowStatus::BrandModalSent => {
            brand_modal_sent(new_step, status, log, message_content)
        }
        FlowStatus::BrandSelected => {
            brand_selected(new_step, status, log, message_content)
        }
        FlowStatus::ModelModalSent => {
            model_modal_sent(&new_step, status, log, message_content)
        }
        FlowStatus::ModelSelected => {
            model_selected(new_step, status, log, message_content)
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


pub fn brand_modal_sent(mut step: &mut TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>  {

    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    // Check if its paging option
    let mut page = 1 as usize;

    // retrieve data is required from data origin
    let mut final_list: Vec<Choice> = vec![];
    if status.value().data_origin.is_some() {
        info!("Found data origin for step");

        info!("Obtaining list size");
        let list_size = get_list_size(&status.value().data_origin.as_ref().unwrap().to_string());

        if list_size.is_err() {
            return Err(format!("Failed to obtain origin {} size", status.value().data_origin.as_ref().unwrap()))
        }


        if page*10 < list_size.unwrap().into() {
            final_list.push(Choice{ id: format!("page-{}", page+1), value: "Pagina Siguiente".to_string() });
        }

        // Determine amount of items to be included in list
        let to = (9 - final_list.len()) * page;
        let from = (9 - final_list.len()) * (page-1);

        // list_size = 15; if page*10 < list_size; add next page button only
        // If page == 1 > dont add previous page button; else add previous page button

        info!("Obtaining list choices for origin {} from index {} to index {}, list_len {}, page {}",status.value().data_origin.as_ref().unwrap().to_string(), from, to, final_list.len(), page);
        let list_res = get_list(status.value().data_origin.as_ref().unwrap().to_string(), from, to);

        if list_res.is_err() {

            return Err(list_res.unwrap_err().to_string());

        }

        for make in list_res.unwrap(){
            info!("{}", &make);
            final_list.push(Choice{ id: format!("{}-id", make), value: make });

        }
    }

    // check what is the next step expected response type
    // let expected_response_id = step_definition.next_step.unwrap();
    // let expected_response = FlowStatus::get_from_value(&(expected_response_id as u16).to_string());

    // Add choices to message in case of list of buttons messages
    debug!("Adding choices to message response");
    info!("{}", message_request.message_type);
    if message_request.message_type == "list".to_string()
    {
        info!("Adding choices to list possibilities");
        message_request.content.list.as_mut().unwrap().choices = final_list;
    }

     // Adding to
    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}


pub fn model_modal_sent(mut step: &TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>  {
    // No required params
    let mut message_request = status.value().successful_response.unwrap();
    info!("{message_content}");

    // Check if its paging option
    let mut page = 1 as usize;

    // retrieve data is required from data origin
    let mut final_list: Vec<Choice> = vec![];
    if status.value().data_origin.is_some() {
        info!("Found data origin for step");

        // Get make
        let make_step = get_step_by_status(&step.tracker_id, &format!("{}", FlowStatus::BrandSelected as u16));

        if make_step.is_err() {
            error!("{}", make_step.as_ref().unwrap_err());
            return Err(make_step.unwrap_err())
        }

        let make: String = String::from(make_step.unwrap().value.split("-").collect::<Vec<&str>>()[0]);
        info!("Found make: {make}");
        let data_origin = status.value().data_origin.as_ref().unwrap().to_string().replace("{}", &make);

        info!("Obtaining list size");
        let list_size = get_list_size(&data_origin);

        if list_size.is_err() {
            return Err(format!("Failed to obtain origin {} size", status.value().data_origin.as_ref().unwrap()))
        }
        if page*10 < list_size.unwrap().into() {
            final_list.push(Choice{ id: format!("page-{}", page+1), value: "Pagina Siguiente".to_string() });
        }

        // Determine amount of items to be included in list
        let to = (9 - final_list.len()) * page;
        let from = (9 - final_list.len()) * (page-1);

        // list_size = 15; if page*10 < list_size; add next page button only
        // If page == 1 > dont add previous page button; else add previous page button

        info!("Obtaining list choices for origin {} from index {} to index {}, list_len {}, page {}",data_origin, from, to, final_list.len(), page);
        let list_res = get_list(data_origin, from, to);

        if list_res.is_err() {

            return Err(list_res.unwrap_err().to_string());

        }

        for make in list_res.unwrap(){
            info!("{}", &make);
            final_list.push(Choice{ id: format!("{}-id", make), value: make });

        }
    }

    // check what is the next step expected response type
    // let expected_response_id = step_definition.next_step.unwrap();
    // let expected_response = FlowStatus::get_from_value(&(expected_response_id as u16).to_string());

    // Add choices to message in case of list of buttons messages
    debug!("Adding choices to message response");
    info!("{}", message_request.message_type);
    if message_request.message_type == "list".to_string()
    {
        info!("Adding choices to list possibilities");
        message_request.content.list.as_mut().unwrap().choices = final_list;
    }

    // Adding to
    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}


fn brand_selected(mut step: &mut TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    // No required params
    let mut message_request = status.value().successful_response.unwrap();
    info!("Initial status: {}", &step.status);

    // Check if its paging option
    if message_content.contains("page-") {
        // Parse from from choice id
        info!("Selection is a paging action");
        let page_parsed = message_content.split("-").collect::<Vec<&str>>()[1];
        let page = page_parsed.parse::<usize>().expect("Error parsing brand modal page");

        // Update step to not update to next one
        let previous_status = ((status as u16)-1).to_string();
        status = FlowStatus::get_from_value(&previous_status);
        step.status = String::from(&previous_status);
        message_request = status.value().successful_response.unwrap();

        // Set default size


        // retrieve data is required from data origin
        let mut final_list: Vec<Choice> = vec![];
        if status.value().data_origin.is_some() {
            info!("Found data origin for step");

            info!("Obtaining list size");
            let list_size = get_list_size(&status.value().data_origin.as_ref().unwrap().to_string());

            if list_size.is_err() {
                return Err(format!("Failed to obtain origin {} size", status.value().data_origin.as_ref().unwrap()))
            }

            if page != 1 {
                final_list.push(Choice{ id: format!("page-{}", page-1), value: "Pagina Anterior".to_string() });
            }

            if page*10 < list_size.unwrap().into() {
                final_list.push(Choice{ id: format!("page-{}", page+1), value: "Pagina Siguiente".to_string() });
            }

            // Determine amount of items to be included in list
            let to = (9 - final_list.len()) * page;
            let from = (9 - final_list.len()) * (page-1);

            // list_size = 15; if page*10 < list_size; add next page button only
            // If page == 1 > dont add previous page button; else add previous page button

            info!("Obtaining list choices for origin {} from index {} to index {}, list_len {}, page {}",status.value().data_origin.as_ref().unwrap().to_string(), from, to, final_list.len(), page);
            let list_res = get_list(status.value().data_origin.as_ref().unwrap().to_string(), from, to);

            if list_res.is_err() {

                return Err(list_res.unwrap_err().to_string());

            }

            for make in list_res.unwrap(){
                info!("{}", &make);
                final_list.push(Choice{ id: format!("{}-id", make), value: make });

            }
        }

        // check what is the next step expected response type
        // let expected_response_id = step_definition.next_step.unwrap();
        // let expected_response = FlowStatus::get_from_value(&(expected_response_id as u16).to_string());

        // Add choices to message in case of list of buttons messages
        debug!("Adding choices to message response");
        info!("{}", message_request.message_type);
        if message_request.message_type == "list".to_string()
        {
            info!("Adding choices to list possibilities");
            message_request.content.list.as_mut().unwrap().choices = final_list;
        }

    }


    // Adding to
    message_request.to.push(log.clone().phone_number);

    info!("Final status: {}", &step.status);
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

    info!("Sending VIN request to user");
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

fn model_selected(mut step: &mut TrackerStep, mut status: FlowStatus, log:&MessageLog, message_content: &str) -> Result<MessageRequest, String>{

    // No required params
    let mut message_request = status.value().successful_response.unwrap();
    info!("Initial status: {}", &step.status);
    info!("{message_content}");

    // Check if its paging option
    if message_content.contains("page-") {
        // Parse from from choice id
        info!("Selection is a paging action");
        let page_parsed = message_content.split("-").collect::<Vec<&str>>()[1];
        let page = page_parsed.parse::<usize>().expect("Error parsing model modal page");

        // Update step to not update to next one
        let previous_status = ((status as u16)-1).to_string();
        status = FlowStatus::get_from_value(&previous_status);
        step.status = String::from(&previous_status);
        message_request = status.value().successful_response.unwrap();

        // Set default size


        // retrieve data is required from data origin
        let mut final_list: Vec<Choice> = vec![];
        if status.value().data_origin.is_some() {
            info!("Found data origin for step");
            // Get make
            let make_step = get_step_by_status(&step.tracker_id, &format!("{}", FlowStatus::BrandSelected as u16));

            if make_step.is_err() {
                error!("{}", make_step.as_ref().unwrap_err());
                return Err(make_step.unwrap_err())
            }

            let make: String = String::from(make_step.unwrap().value.split("-").collect::<Vec<&str>>()[0]);
            info!("Found make: {make}");
            let data_origin = status.value().data_origin.as_ref().unwrap().to_string().replace("{}", &make);


            info!("Obtaining list size");
            let list_size = get_list_size(&data_origin);

            if list_size.is_err() {
                return Err(format!("Failed to obtain origin {} size", status.value().data_origin.as_ref().unwrap()))
            }

            if page != 1 {
                final_list.push(Choice{ id: format!("page-{}", page-1), value: "Pagina Anterior".to_string() });
            }

            if page*10 < list_size.unwrap().into() {
                final_list.push(Choice{ id: format!("page-{}", page+1), value: "Pagina Siguiente".to_string() });
            }

            // Determine amount of items to be included in list
            let to = (9 - final_list.len()) * page;
            let from = (9 - final_list.len()) * (page-1);

            // list_size = 15; if page*10 < list_size; add next page button only
            // If page == 1 > dont add previous page button; else add previous page button


            let list_res = get_list(data_origin, from, to);

            if list_res.is_err() {

                return Err(list_res.unwrap_err().to_string());

            }

            for make in list_res.unwrap(){
                info!("{}", &make);
                final_list.push(Choice{ id: format!("{}-id", make), value: make });

            }
        }

        // check what is the next step expected response type
        // let expected_response_id = step_definition.next_step.unwrap();
        // let expected_response = FlowStatus::get_from_value(&(expected_response_id as u16).to_string());

        // Add choices to message in case of list of buttons messages
        debug!("Adding choices to message response");
        info!("{}", message_request.message_type);
        if message_request.message_type == "list".to_string()
        {
            info!("Adding choices to list possibilities");
            message_request.content.list.as_mut().unwrap().choices = final_list;
        }

    }


    // Adding to
    message_request.to.push(log.clone().phone_number);

    info!("Final status: {}", &step.status);
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

    info!("Sending VIN request to user");
    // No required params
    let mut message_request = status.value().successful_response.unwrap();

    // Set phone number
    message_request.to.push(log.clone().phone_number);

    Ok(message_request)
}