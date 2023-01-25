use std::string::ToString;
use enum_iterator::{all, cardinality, first, last, next, previous, reverse_all, Sequence};
use fizzy_commons::shared_structs::{ListMessage, MessageContent, MessageRequest};
use serde::de::Unexpected::Str;
use crate::constants::FlowStatus::{BrandModalSent, BrandSelected, IdentificationRequestSent, ModelModalSent, ModelSelected, PartDescriptionProvided, RequestAccepted};

use crate::constants::FlowStatusId::{BrandModalSentId, BrandSelectedId, FlowStartedId, IdentificationProvidedId, IdentificationRequestSentId, ModelModalSentId, ModelSelectedId, PartDescriptionProvidedId, PartDescriptionRequestedId, RequestAcceptedId};
use crate::constants::MessageType::{ListSelection, NoResponse, PlainText, PlainTextAndImage};
use crate::structs::{StepDefinition};

pub const SYSTEM_ID: u8 = 3;
pub const PART_CLASSIFICATION_SYSTEM_ID: u8 = 4;

#[derive(Debug)]
pub enum FlowStatusId {
    FlowStartedId = 1,
    BrandModalSentId = 2,
    BrandSelectedId = 3,
    ModelModalSentId = 4,
    ModelSelectedId = 5,
    IdentificationRequestSentId = 6,
    IdentificationProvidedId = 7,
    PartDescriptionRequestedId = 8,
    PartDescriptionProvidedId = 9,
    RequestAcceptedId = 10,
}

impl FlowStatusId {
    pub fn get_from_value(i: &String) -> FlowStatusId {
        let status_id = i.parse().unwrap();
        match status_id {
            1 => FlowStatusId::FlowStartedId,
            2 => FlowStatusId::BrandModalSentId,
            3 => FlowStatusId::BrandSelectedId,
            4 => FlowStatusId::ModelModalSentId,
            5 => FlowStatusId::ModelSelectedId,
            6 => FlowStatusId::IdentificationRequestSentId,
            7 => FlowStatusId::IdentificationProvidedId,
            8 => FlowStatusId::PartDescriptionRequestedId,
            9 => FlowStatusId::PartDescriptionProvidedId,
            10 => FlowStatusId::RequestAcceptedId,
            _ => panic!("Value not found"),
        }
    }
}

#[derive(Debug, PartialEq, Sequence, Clone, Copy)]
pub enum FlowStatus {
    FlowStarted = 1,
    BrandModalSent = 2,
    BrandSelected = 3,
    ModelModalSent = 4,
    ModelSelected = 5,
    IdentificationRequestSent = 6,
    IdentificationProvided = 7,
    PartDescriptionRequested = 8,
    PartDescriptionProvided = 9,
    RequestAccepted = 10,
}


impl FlowStatus{
    pub fn value(&self) -> StepDefinition {

        // SUCCESSFUL MESSAGE RESPONSES

        let FLOW_STARTED_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("text"),
            content: MessageContent {
                body: Some("Escribe 'hola' para iniciar la solicitud.".to_string()),
                list: None,
                buttons: None,
            },
        };

        let BRAND_MODAL_SENT_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("list"),
            content: MessageContent {
                body: Some("Selecciona la marca del vehiculo.".to_string()),
                list: Some(ListMessage{
                    title: "Marcas".to_string(),
                    choices: vec![],
                }),
                buttons: None,
            },
        };

        let BRAND_SELECTED_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("text"),
            content: MessageContent {
                body: Some("Has seleccionado {}.".to_string()),
                list: None,
                buttons: None,
            },
        };

        let MODEL_MODAL_SENT_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("list"),
            content: MessageContent {
                body: Some("Selecciona el modelo correspondiente al vehiculo.".to_string()),
                list: Some(ListMessage{
                    title: "Modelos".to_string(),
                    choices: vec![],
                }),
                buttons: None,
            },
        };


        let MODEL_SELECTED_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("text"),
            content: MessageContent {
                body: Some("Has seleccionado {}.".to_string()),
                list: None,
                buttons: None,
            },
        };


        let IDENTIFICATION_REQUEST_SENT_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("text"),
            content: MessageContent {
                body: Some("Ingresa la patente o VIN del vehiculo a consultar.".to_string()),
                list: Some(ListMessage{
                    title: "Modelos".to_string(),
                    choices: vec![],
                }),
                buttons: None,
            },
        };

        let IDENTIFICATION_PROVIDED_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("text"),
            content: MessageContent {
                body: Some("El identificador provisto es valido.".to_string()),
                list: None,
                buttons: None,
            },
        };

        let PART_DESCRIPTION_REQUESTED_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("text"),
            content: MessageContent {
                body: Some("Por favor, describa el repuesto que busca de la manera mas especifica posible, puede adjuntar una imagen en el mismo mensaje(solo una).".to_string()),
                list: None,
                buttons: None,
            },
        };


        let PART_DESCRIPTION_PROVIDED_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("text"),
            content: MessageContent {
                body: Some("Se recibio descripcion de repuesto.".to_string()),
                list: None,
                buttons: None,
            },
        };

        let REQUEST_ACCEPTED_MESSAGE: MessageRequest = MessageRequest{
            system_id: SYSTEM_ID,
            to: vec![],
            message_type: String::from("text"),
            content: MessageContent {
                body: Some("Se recibio la solicitud de repuesto exitosamente, lo estaremos contactando una vez encontremos el repuesto buscado.".to_string()),
                list: None,
                buttons: None,
            },
        };

        let flow_started_step:StepDefinition =  StepDefinition{
            required_response: None,
            validation_regex: Some(String::from("")),

            next_step: Some(BrandModalSentId),
            successful_response: Some(FLOW_STARTED_MESSAGE),
            data_origin: None,
        };


        let brand_modal_sent_step: StepDefinition =  StepDefinition{
            required_response: Some(PlainText),
            validation_regex: Some(String::from("hola")),

            next_step: Some(BrandSelectedId),
            successful_response: Some(BRAND_MODAL_SENT_MESSAGE),
            data_origin: Some("makes".to_string()),
        };

        let brand_selected_step:StepDefinition =  StepDefinition{
            required_response: Some(ListSelection),
            validation_regex: Some("".to_string()),

            next_step: Some(ModelModalSentId),
            successful_response: Some(BRAND_SELECTED_MESSAGE),
            data_origin: None,
        };

        let model_modal_sent_step: StepDefinition =  StepDefinition{
            required_response: None,
            validation_regex: Some("".to_string()),

            next_step: Some(ModelSelectedId),
            successful_response: Some(MODEL_MODAL_SENT_MESSAGE),
            data_origin: Some("models:{}".to_string()),
        };

        let model_selected_step:StepDefinition = StepDefinition{
            required_response: Some(ListSelection),
            validation_regex: Some("".to_string()),

            next_step: Some(IdentificationRequestSentId),
            successful_response: Some(MODEL_SELECTED_MESSAGE),
            data_origin: Some("models:{}".to_string()),
        };

        let identification_request_sent_step:StepDefinition =  StepDefinition{
            required_response: None,
            validation_regex: Some("".to_string()),

            next_step: Some(IdentificationProvidedId),
            successful_response: Some(IDENTIFICATION_REQUEST_SENT_MESSAGE),
            data_origin: None,
        };

        let identification_provided_step: StepDefinition =  StepDefinition{
            required_response: Some(PlainText),
            validation_regex: Some("([A-Za-z0-9]){17,19}|([A-Z0-9]{6})".to_string()),

            next_step: Some(PartDescriptionRequestedId),
            successful_response: Some(IDENTIFICATION_PROVIDED_MESSAGE),
            data_origin: None,
        };

        let part_description_requested_step:StepDefinition =  StepDefinition{
            required_response: None,
            validation_regex: Some("".to_string()),

            next_step: Some(PartDescriptionProvidedId),
            successful_response: Some(PART_DESCRIPTION_REQUESTED_MESSAGE),
            data_origin: None,
        };

        let part_description_provided_step:StepDefinition =  StepDefinition{
            required_response: Some(PlainTextAndImage),
            validation_regex: Some("".to_string()),

            next_step: Some(RequestAcceptedId),
            successful_response: Some(PART_DESCRIPTION_PROVIDED_MESSAGE),
            data_origin: None,
        };


        let request_accepted_step:StepDefinition =  StepDefinition{
            required_response: None,
            validation_regex: Some("".to_string()),

            next_step: None,
            successful_response: Some(REQUEST_ACCEPTED_MESSAGE),
            data_origin: None,
        };

        match self {
            FlowStatus::FlowStarted => flow_started_step,
            FlowStatus::BrandModalSent => brand_modal_sent_step,
            FlowStatus::BrandSelected => brand_selected_step,
            FlowStatus::ModelModalSent => model_modal_sent_step,
            FlowStatus::ModelSelected => model_selected_step,
            FlowStatus::IdentificationRequestSent => identification_request_sent_step,
            FlowStatus::IdentificationProvided => identification_provided_step,
            FlowStatus::PartDescriptionRequested => part_description_requested_step,
            FlowStatus::PartDescriptionProvided => part_description_provided_step,
            FlowStatus::RequestAccepted => request_accepted_step,
        }


    }

    pub fn get_from_value(i: &String) -> FlowStatus {
        debug!("ingest: {}", i);
        let status_id = i.parse().unwrap();
        debug!("Parsing FlowStatus from id {}", status_id);
        match status_id {
            1 => FlowStatus::FlowStarted,
            2 => FlowStatus::BrandModalSent,
            3 => FlowStatus::BrandSelected,
            4 => FlowStatus::ModelModalSent,
            5 => FlowStatus::ModelSelected,
            6 => FlowStatus::IdentificationRequestSent,
            7 => FlowStatus::IdentificationProvided,
            8 => FlowStatus::PartDescriptionRequested,
            9 => FlowStatus::PartDescriptionProvided,
            10 => FlowStatus::RequestAccepted,
            _ => panic!("Value not found"),
        }
    }

}

#[derive(PartialEq, Debug)]
pub enum MessageType {
    PlainText,
    PlainTextAndImage,
    ListSelection,
    ButtonSelection,
    NoResponse
}

#[derive(PartialEq)]
pub enum ResponseStatus{
    ExpectingResponse,
    SystemMessage
}








