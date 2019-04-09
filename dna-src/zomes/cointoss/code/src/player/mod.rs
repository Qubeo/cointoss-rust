#![allow(dead_code)]
use hdk::{
    self,
    entry_definition::{
        ValidatingEntryType
    },
    holochain_core_types::{
        cas::content::Address, dna::entry_types::Sharing, error::HolochainError, json::JsonString
    }
};

pub mod handlers;


#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct HandleSchema {
    pub handle: String
}

// Agent / player / member related ---------------------------------------------------------------------
//-----------------------------------------------------------------------------
//                            Entry definitions
//-----------------------------------------------------------------------------

pub fn handle_definition() -> ValidatingEntryType {
        
    // Entry: "handle" for __________? The player? 
    entry!(
        name: "handle",
        description: "",
        sharing: Sharing::Public,
        native_type: HandleSchema,                                // Q: Why does String, or even JsonString not work any more?
        
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        
        validation: |_handle: HandleSchema, _validation_data: hdk::ValidationData| { Ok(()) },
        
        links: [
            to!(
                "%agent_id",
                tag: "has_member",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: |_base: Address, _target: Address, _ctx: hdk::ValidationData| {
                    Ok(())
                }
            ),
            from!(
                "%agent_id",
                tag: "member_of",

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: |_base: Address, _target: Address, _ctx: hdk::ValidationData| {
                    Ok(())
                }
            ),
            to!(
                "seed",
                tag: "seeds",       // TODO: Distinguish - the same or different to "seeds" (smwh else)?

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: |_base: Address, _target: Address, _ctx: hdk::ValidationData| {
                    Ok(())
                }
            ),
            to!(
                "toss_result",
                tag: "toss_results",       // TODO: Distinguish - the same or different to "seeds" (smwh else)?

                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },

                validation: |_base: Address, _target: Address, _ctx: hdk::ValidationData| {
                    Ok(())
                }
            )
        ]
    )
} 