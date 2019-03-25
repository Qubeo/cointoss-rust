#![allow(dead_code)]
use hdk::{
    self,
    entry_definition::{
        ValidatingEntryType
    },
    holochain_core_types::{
        cas::content::Address,
        dna::entry_types::Sharing,
        error::HolochainError,
        json::JsonString,
        hash::HashString
    }
};

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum TossOutcome {
    InitiatorWon,
    InitiatorLost,
    Draw
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct TossSchema {
    pub initiator: Address,
    pub initiator_seed_hash: HashString,
    pub responder: Address,
    pub responder_seed_hash: HashString,
    pub call: u8
    // pub required: ["initiator", "initiator_seed_hash", "responder", "responder_seed_hash"]; // Q: How to initialize the field?
}

#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct TossResultSchema {
    pub toss: TossSchema,       
	pub outcome: TossOutcome,         // Q: What format?
    pub time_stamp: String
    // pub required:  ["toss","result","time_stamp"] // Q: Validation rules?
}

// Q: Is this useful?
#[derive(Serialize, Deserialize, Debug, DefaultJson)]
pub struct ResultAndRevealedSchema {
    pub toss_result: TossResultSchema,
    pub initiator_seed: SeedSchema
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct SeedSchema {
    pub salt: String,    
    pub seed_value: u8
}

pub mod handlers;


pub fn seed_definition() -> ValidatingEntryType {
    entry!(
        name: "seed",
        description: "",
        sharing: Sharing::Private,
        native_type: SeedSchema, 
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |_seed: SeedSchema, _validation_data: hdk::ValidationData| { Ok(()) },
        links: [
            from!(
                "%agent_id",
                tag: "agent",
                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },
                validation: |_source: Address, _target: Address, _validation_data: hdk::ValidationData| {
                    Ok(())
                })
            ]
    )
}

pub fn toss_definition() -> ValidatingEntryType {
    entry!(
        name: "toss",    // Learning: Experimenting with "enum" instead of hardcoded string
        description: "",
        sharing: Sharing::Public,
        native_type: TossSchema, // Q: Or? Json? JsonString?
        validation_package: || { 
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |_toss: TossSchema, _validation_data: hdk::ValidationData| { Ok(()) }
    )
}

pub fn toss_result_definition() -> ValidatingEntryType {    
    entry!(
        name: "toss_result",
        description: "",
        sharing: Sharing::Public,
        native_type: TossResultSchema, // Q: Or?
        validation_package: || { 
            hdk::ValidationPackageDefinition::Entry
        },
        validation: |_toss_result: TossResultSchema, _validation_data: hdk::ValidationData| { Ok(()) }
    )
}


