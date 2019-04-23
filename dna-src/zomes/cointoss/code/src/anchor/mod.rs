use hdk::entry_definition::ValidatingEntryType;

use hdk::holochain_core_types::{
    dna::entry_types::Sharing,
    cas::content::Address,
    json::RawString,
};


pub fn anchor_definition() -> ValidatingEntryType {
    entry!(
        name: "anchor",
        description: "",
        sharing: Sharing::Public,
        // native_type: RawString,

        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },
  
        validation: |_validation_date: hdk::EntryValidationData<RawString>| {
            Ok(())
        },

        links: [
            to!(
                "%agent_id",
                tag: "member_tag",

                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },

                validation: |_validation_data: hdk::LinkValidationData| {
                    Ok(())
                }
            ),
            to!(
                "toss",
                tag: "toss",

                validation_package: || {
                    hdk::ValidationPackageDefinition::ChainFull
                },

                validation: |_validation_data: hdk::LinkValidationData| {
                    Ok(())
                }
            ),
            to!(
                "seed",
                tag: "seeds",

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