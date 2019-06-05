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
        // native_type: HandleSchema,                                // Q: Why does String, or even JsonString not work any more?
        
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
        
        validation: |_validation_data: hdk::EntryValidationData<HandleSchema>| { Ok(()) },
        
        links: [
            to!(
                "%agent_id",
                link_type: "has_member",

                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },

                validation: |_validation_data: hdk::LinkValidationData| {
                    Ok(())
                }
            ),
            from!(
                "%agent_id",
                link_type: "member_of",

                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },

                validation: |_validation_data: hdk::LinkValidationData| {
                    Ok(())
                }
            ),
            to!(
                "seed",
                link_type: "seeds",       // TODO: Distinguish - the same or different to "seeds" (smwh else)?

                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },

                validation: |_validation_data: hdk::LinkValidationData| {
                    Ok(())
                }
            ),
            to!(
                "toss_result",
                link_type: "toss_results",       // TODO: Distinguish - the same or different to "seeds" (smwh else)?

                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },

                validation: |_validation_data: hdk::LinkValidationData| {
                    Ok(())
                }
            )
        ]
    )
} 