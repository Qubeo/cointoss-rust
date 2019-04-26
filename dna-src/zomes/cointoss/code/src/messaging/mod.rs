// use hdk::snowflake;
use hdk::{
    holochain_core_types::{
        cas::content::{ Address },
        error::HolochainError,
        hash::HashString,
        json::{ JsonString },
    }
};

// Q: Design: Is this coupling healthy?
use crate::toss;
// use crate::toss::{ TossSchema, SeedSchema, TossResultSchema };

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum MsgType {
    WannaPlay,
    RequestToss,
    TossResponse
}

// Represents the message 
// Q: Design patterns: Should those fields be public, or implement getters, or?
#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct GeneralMsg {
    pub agent_from: Address,
    pub message_type: MsgType,
    pub message: String
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct RequestMsg {
    pub agent_from: Address,
    pub seed_hash: HashString
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct TossResponseMsg {
    pub agent_from: Address,
    pub responder_seed: toss::SeedSchema,
    pub toss_hash: HashString,
    pub call: u8
}

// TODO: Add error handling.
// Q: How to make the deserialization safe?
// Q: Generalize the processing thru general types <T>?
pub fn process_request_msg(request: String) -> RequestMsg {
    let request_msg: RequestMsg = serde_json::from_str(&request).unwrap(); //RequestMsg::try_from(request);
    let _res_dbg = hdk::debug(format!("HCH/ process_request_msg(): request, request_msg: {}, {:?}", request.clone(), request_msg.clone()));

    request_msg
}

pub fn process_general_msg(payload: String) -> GeneralMsg {
    let general_msg: GeneralMsg = serde_json::from_str(&payload).unwrap();
    general_msg
}

pub fn process_toss_response_msg(message: String) -> TossResponseMsg {
    // Q: Beware some unhealthy coupling / tangling? (i.e. "process_" function shouldn't commit entries I'd say.)
    let _debug_res = hdk::debug("HCH/ process_toss_response_msg()");
    let toss_response_msg: TossResponseMsg = serde_json::from_str(&message).unwrap();
    toss_response_msg
}

/*
pub fn proccess_msg<T>(message: String) -> T {
    let response_msg: T = serde_json::from_str(&message).unwrap();
    response_msg
} */