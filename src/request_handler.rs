use std::time::{SystemTime, UNIX_EPOCH};
use aws_config::SdkConfig;
use fizzy_commons::shared_structs::MessageRequest;
use log::Level::Info;
use redis::RedisError;
use regex::Regex;
use serde::de::Unexpected::Str;
use crate::redis::{create_new_step, create_new_tracker, get_last_tracker, get_last_tracker_step, get_user_message, publish_message};
use crate::structs::{Event, MessageLog, ModifiedReference, StandardResponse, TrackerStep};
use uuid::Uuid;
use crate::constants::{FlowStatus, MessageType, ResponseStatus};
use crate::step_functions::execute_function;
use crate::tools::{find_message_type, get_message_content, send_message, upload_image};


pub async fn outgoing_message(log: MessageLog) -> Result<StandardResponse, StandardResponse> {
    let mut response: StandardResponse = StandardResponse::new();
    let mut errors = vec![];
    let mut references = vec![];
    let first_step = FlowStatus::FlowStarted as u16;

    info!("log: {:?}", serde_json::to_string_pretty(&log).unwrap());

    if log.origin_system == "1" {
        info!("Message from whatsapp-manager");
        // If message comes from whatsapp manager mode selection

        // Create new workflow
        // Removes hyphen for limitation on query syntax
        let uuid_tracker = Uuid::new_v4().to_string().replace("-", "");

        let created = create_new_tracker(&uuid_tracker, &log.phone_number);

        if created.is_err() {
            error!("Error creating tracker for request");
            errors.push("Error creating tracker for request".to_string());

            response.errors = Some(errors);
            return Err(response)
        }
        references.push(ModifiedReference {
            system: "REDIS".to_string(),
            reference: created.unwrap().to_string(),
        });

        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_millis().to_string(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        // Create initial tracker step
        let uuid_step = Uuid::new_v4().to_string().replace("-", "");
        let initial_step = TrackerStep{
            tracker_id: uuid_tracker,
            timestamp: timestamp.clone(), // It's set on creation function
            id: uuid_step,
            status: first_step.to_string(),
            value: "".to_string(),
            attached_files: "".to_string(),
            message_reference: String::from(&log.register_id),
        };

        // Create new step register
        let step_res = create_new_step(&initial_step);


        if step_res.is_err() {
            error!("Error creating initial request for tracker");
            errors.push("Error creating initial request for tracker".to_string());

            response.errors = Some(errors);
            return Err(response)
        }
        references.push(ModifiedReference {
            system: "REDIS".to_string(),
            reference: step_res.unwrap().to_string(),
        });


        // Send message from step
        if FlowStatus::get_from_value(&first_step.to_string()).value().successful_response.is_none() {

            // Execute message sending
            let message = FlowStatus::get_from_value(&first_step.to_string()).value().successful_response.unwrap();
            let message_res: Result<StandardResponse, String> = send_message(message);

            // If error sending message
            if message_res.is_err() {
                let err: String = message_res.unwrap_err();
                error!("Error sending message {}", err);
                errors.push(format!("Error sending message {}", err));

                response.errors = Some(errors);
                return Err(response)
            }

            // Publish message

            let new_log = MessageLog{
                timestamp: timestamp.clone(),
                destination_systems: vec!["3".to_string()],
                origin_system: "3".to_string(),
                phone_number: log.phone_number.to_string(),
                origin: "OUTGOING".to_string(),
                register_id: String::from(&log.register_id),
            };


            let publish_res = publish_message(&new_log, &log.phone_number);

            // If error sending message
            if publish_res.is_err() {
                error!("Error publishing message {}", publish_res.as_ref().unwrap_err());
                errors.push(format!("Error publishing message {}", publish_res.as_ref().unwrap_err()));

                response.errors = Some(errors);
                return Err(response)
            }

        }


    } else if &log.origin_system == "3" {

        info!("Message from own system");
        // If message comes from same system

        let tracker = get_last_tracker(&log.phone_number);

        if tracker.is_err() {
            error!("Error obtaining last tracker {}", tracker.as_ref().unwrap_err().as_str());
            errors.push(format!("Error obtaining last tracker {}", tracker.as_ref().unwrap_err().as_str()));

            response.errors = Some(errors);
            return Err(response)
        }

        let step = get_last_tracker_step(&tracker.as_ref().unwrap().id);

        if step.is_err() {
            error!("Error obtaining last step {}", step.as_ref().unwrap_err().as_str());
            errors.push(format!("Error obtaining last step {}", step.as_ref().unwrap_err().as_str()));

            response.errors = Some(errors);
            return Err(response)
        }

        let status = FlowStatus::get_from_value(&step.unwrap().status);


        // If request is completed generate a log to channel so the request classification system can be notified
        if status == FlowStatus::RequestAccepted {

            let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => n.as_millis().to_string(),
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            };

            let accepted_log = MessageLog {
                timestamp: timestamp,
                destination_systems: vec![5.to_string()],
                origin_system: "3".to_string(),
                phone_number: log.phone_number.clone(),
                origin: "OUTGOING".to_string(),
                register_id: log.register_id.clone(),
            };

            publish_message(&accepted_log, &log.phone_number);

            response.errors = None;
            response.references = references;
            return Ok(response)

        }

        info!("Current flow status: {:?}", status);
        let next_step_id = if status.value().next_step.is_some() {
             (status.value().next_step.unwrap() as u16).to_string()
        }else{
            (status as u16).to_string()
        };

        let next_step = FlowStatus::get_from_value(&next_step_id.to_string());

        info!("Next step is {next_step:?}");
        if next_step.value().required_response.is_none() {

            info!("Step {next_step:?} requires response");


            let uuid_step = Uuid::new_v4().to_string().replace("-", "");

            let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(n) => n.as_millis().to_string(),
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            };

            let mut new_step = TrackerStep{
                tracker_id: String::from(&tracker.as_ref().unwrap().id),
                timestamp: timestamp.clone(),
                id: uuid_step,
                status: (next_step as u16).to_string(),
                value: "".to_string(),
                attached_files: "".to_string(),
                message_reference: String::from(&log.register_id.clone()),
            };

            info!("Executing {next_step:?} handler function");
            let parsed_message: Result<MessageRequest, String> = execute_function(&mut new_step, next_step, &log, "").await;

            if parsed_message.is_err() {
                // errors.push(parsed_message.unwrap_err());

                response.errors = Some(errors);
                return Err(response)
            }

            // SEND MESSAGE
            let res = parsed_message.unwrap();

            debug!("{:?}", serde_json::to_string(&res));
            let res = send_message(res);

            if res.is_err() {
                errors.push(String::from("Error sending message"));

                response.errors = Some(errors);
                return Err(response)
            }

            // PUBLISH MESSAGE TO CHANNEL

            let new_log = MessageLog{
                timestamp: timestamp.clone(),
                destination_systems: vec!["3".to_string()],
                origin_system: "3".to_string(),
                phone_number: log.phone_number.to_string(),
                origin: "OUTGOING".to_string(),
                register_id: String::from(&res.as_ref().unwrap().references[0].reference),
            };


            publish_message(&new_log, &log.phone_number);


            // Updated request status

            let step_res = create_new_step(&new_step);

            if step_res.is_err() {
                errors.push(format!("Unable to create new step: {}", step_res.as_ref().unwrap_err()));

                response.errors = Some(errors);
                return Err(response)
            }


            references.push(ModifiedReference{ system: "REDIS".to_string(), reference: step_res.as_ref().unwrap().clone() });
        }


    }else{
        error!("Origin system not supported: {}", &log.origin_system);
        errors.push(format!("Origin system not supported: {}", &log.origin_system));

        response.errors = Some(errors);
        return Err(response)
    }




    response.errors = None;
    response.references = references;
    Ok(response)
}


pub async fn incoming_message(log: MessageLog) -> Result<StandardResponse, StandardResponse>{
    let mut response: StandardResponse = StandardResponse::new();
    let mut errors: Vec<String> = vec![];
    let mut references: Vec<ModifiedReference> = vec![];

    // Get last tracker
    info!("Obtaining last tracker for phone number");
    debug!("Obtaining last tracker for phone number {}", &log.phone_number);

    let tracker = get_last_tracker(&log.phone_number);

    if tracker.is_err() {

        if tracker.as_ref().unwrap_err().contains("No records found"){
            error!("No request tracker found for user {}", &log.phone_number);
            errors.push(format!("No request tracker found for user {}", &log.phone_number));


        }
        errors.push(tracker.unwrap_err().to_string());
        response.errors = Some(errors);
        return Err(response)
    }

    info!("Found tracker for phone number");
    debug!("Found tracker {} for phone number {}", tracker.as_ref().unwrap().id, &log.phone_number);


    info!("Obtaining tracker last step in workflow");
    debug!("Obtaining tracker {} last step in workflow", tracker.as_ref().unwrap().id);
    // Get tracker last step
    let step = get_last_tracker_step(&tracker.as_ref().unwrap().id);

    if step.is_err() {

        if step.as_ref().unwrap_err().contains("No records found"){
            error!("No tracker step found for user {}", &log.phone_number);
            errors.push(format!("No tracker step found for user {}", &log.phone_number));


        }
        errors.push(step.unwrap_err().to_string());
        response.errors = Some(errors);
        return Err(response)
    }

    info!("Found tracker last step");
    debug!("Found tracker last step with id {}", step.as_ref().unwrap().id);


    info!("Getting message content for step associated message reference");
    info!("Getting message content for step {} associated message reference {}", step.as_ref().unwrap().id, step.as_ref().unwrap().message_reference.replace("whatsapp-workflow:", ""));

    let mut register_id = log.register_id.clone();


    // Get message
    info!("register id: {}", register_id);
    let message: Result<Event, RedisError>= get_user_message(&register_id, &log.phone_number);
    if message.is_err() {

        // errors.push(message.unwrap_err().to_string());
        response.errors = Some(errors);
        return Err(response)
    }

    info!("Found message content for specified reference");
    debug!("Found message content for specified reference {}", step.as_ref().unwrap().message_reference);


    // INIT OF LOGIC OF STEP STATUS

    info!("Obtaining next step for current status");
    debug!("Obtaining next step for current status {}", &step.as_ref().unwrap().status);
    // Get possible next steps based on current status
    let status: FlowStatus = FlowStatus::get_from_value(&step.as_ref().unwrap().status);

    if status.value().next_step.is_none() {
        errors.push("No possible next step".to_string());
        // implement a solution that doesnt throws an error when request is finished in last status
        response.errors = Some(errors);
        return Err(response)
    }
    let next_step_status = status.value().next_step.unwrap() as u16;
    let next_step = FlowStatus::get_from_value(&next_step_status.to_string());
    info!("Next step found");
    debug!("Next step found: {:?}", &next_step);

    // Check if next step expects an user response
    if next_step.value().required_response.is_none() && log.origin_system == "1"{
        errors.push("Next step in workflow doesnt expect an user response".to_string());

        response.errors = Some(errors);
        return Ok(response)
        // TODO: Dont send error for this
    }


    info!("Next step requires user response");

    // Filter step based if message type fits required response type(plain text, plain text with image, list selection, button selection)
    let message_type = find_message_type(message.as_ref().unwrap());

    info!("Message type expected found");
    debug!("Message type expected found: {:?}", &message_type);

    if next_step.value().required_response.is_some() && next_step.value().required_response.unwrap() != message_type {
        errors.push(format!("Message type {:?} doesnt match with the next step required message type {:?}", message_type, next_step));

        // TODO: implement a solution that doesnt throws an error when request is finished in last status
        response.errors = Some(errors);
        return Err(response)
    }

    // Obtaining message content
    info!("Obtaining content from message reference");
    let message_content = get_message_content(message.as_ref().unwrap());

    // Filter based on regex(if defined)
    info!("Proceeding to evaluate regex");
    debug!("Proceeding to evaluate regex: {}", next_step.value().validation_regex.unwrap());


    if next_step.value().validation_regex.unwrap() != "" {
        println!("regex to evaluate: {}", next_step.value().validation_regex.unwrap());
        let re = Regex::new(next_step.value().validation_regex.unwrap().as_str()).unwrap();

        println!("message content: {}", message_content);
        let caps = re.captures(&message_content);

        if caps.is_none() {
            error!("Message content doesnt match required regex");
            errors.push("Message content doesnt match required regex".to_string());

            response.errors = Some(errors);
            return Err(response)
        }
    }

    // EXECUTE STEP HANDLER FUNCTION

    info!("Handling next status flow");
    let uuid_step = Uuid::new_v4().to_string().replace("-", "");

    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis().to_string(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };


    let mut new_step = TrackerStep{
        tracker_id: String::from(&tracker.as_ref().unwrap().id),
        timestamp: timestamp.clone(),
        id: uuid_step,
        status: (next_step as u16).to_string(),
        value: message_content.clone(),
        attached_files: "".to_string(),
        message_reference: String::from(&log.register_id.clone()),
    };



    // Execute handler function
    let parsed_message: Result<MessageRequest, String> = execute_function(&mut new_step, next_step, &log, message_content.as_str()).await;
    if parsed_message.is_err() {

        response.errors = Some(errors);
        return Err(response)
    }

    // SEND MESSAGE
    let res = parsed_message.unwrap();

    info!("{:?}", serde_json::to_string(&res));
    let res = send_message(res);

    if res.is_err() {
        errors.push(String::from("Error sending message"));

        response.errors = Some(errors);
        return Err(response)
    }

    // Updated request status
    let step_res = create_new_step(&new_step);

    if step_res.is_err() {
        errors.push(format!("Unable to create new step: {}", step_res.as_ref().unwrap_err()));

        response.errors = Some(errors);
        return Err(response)
    }

    references.push(ModifiedReference{ system: "REDIS".to_string(), reference: step_res.as_ref().unwrap().clone() });


    info!("Next step status: {}, step: {}", new_step.status , step.as_ref().unwrap().status);
    if new_step.status != step.as_ref().unwrap().status {

        // Update if current status is different to computed next status
        // PUBLISH MESSAGE TO CHANNEL
        let new_log = MessageLog{
            timestamp: timestamp.clone(),
            destination_systems: vec!["3".to_string()],
            origin_system: "3".to_string(),
            phone_number: log.phone_number.to_string(),
            origin: "OUTGOING".to_string(),
            register_id: String::from(&res.as_ref().unwrap().references[0].reference),
        };


        publish_message(&new_log, &log.phone_number);

    }


    response.errors = None;
    response.references = references;
    Ok(response)
}