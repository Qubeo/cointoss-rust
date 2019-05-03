#![allow(dead_code)]
#![feature(try_from)]
#[macro_use]
extern crate hdk;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_core_types_derive;
#[macro_use]
extern crate serde_json;
extern crate log;

use hdk::{
    error::ZomeApiResult,
    holochain_core_types::{
        cas::content::{ Address },
        error::HolochainError,
        json::{ JsonString },
        hash::HashString
    },
};

mod player;
mod anchor;
mod toss;
mod messaging;
mod pseudorand;

/*#[derive(Debug)]
pub enum CTEntryType {
    handle,
    seed,
    toss,
    toss_result
}
// Learning: Playing with an alternative to "entry_type".into() for creating entries
// Q: Wouldn't CTEntryName or CTEntry be more appropriate?
// Q: How to integrate this better with the entry! macro?
// Q: How to automatically convert the value into string, without needing to use value.to_string()?
impl fmt::Display for CTEntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}*/

// ZOME DEFINITION --------------------------------------------------------------------------------
define_zome! {
    entries: [

        player::handle_definition(),
        toss::toss_definition(),
        toss::toss_result_definition(),
        toss::seed_definition(),
        anchor::anchor_definition()

        // ISSUE: Q: It seems I can define multiple entries of the same type / content. Isn't this a bug?

       /* Q: Link entries. What to do with those?npm 
        entry!(
            name: "handle_links",
            native_type:
        ),
        entry!(
            name: "directory_links",
            native_type:
        ), 
        entry!(
            name: "history_link_base",
            native_type:
            sharing: Sharing::Public,
            validation_package: || { },
            validation: || {}
        ),
        entry!(
            name: "history_links",
            native_type: links 
        ) */
    ]

    genesis: || {        
            { Ok(()) }
    }
    
    receive: |payload| {
        
        //match hdk::debug(format!("Receive: AGENT_ADDRESS: {:?}", hdk::AGENT_ADDRESS.to_string()) {
        //    _ => "Test".to_string()
       // }
        //toss::handlers::process_received_message(payload).unwrap() // Q: Shoudn't be some kind of async / promise sth? What if blocking?
        hdk::AGENT_ADDRESS.to_string()
        // "Receive test".to_string()

     }

    functions: [
			get_my_address: {
				inputs: | |,
				outputs: |result: ZomeApiResult<Address>|,       // Q: Not sure about the return type. HashString? Or everything here JsonString?
				handler: player::handlers::handle_get_my_address                      // Q: If everything is expected to be JsonString, why ask the type at all - verbose?
			}
            register: {
                inputs: |handle: String|,
                outputs: |result: ZomeApiResult<Address>|,
                handler: player::handlers::handle_register 
            }
    		set_handle: {
				inputs: |handle: String|,
				outputs: |result: ZomeApiResult<Address>|,      // Q: How does this syntax work? Closure arguments without follow up function body? :o
				handler: player::handlers::handle_set_handle
			}
            get_handles: {
				inputs: | |,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: player::handlers::handle_get_handles
			}
            get_handle: {
				inputs: |agent_from: HashString|,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: player::handlers::handle_get_handle
			}
            get_my_handle: {
				inputs: | |,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: player::handlers::handle_get_my_handle
			}
            get_agent: {
				inputs: |handle: HashString|,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: player::handlers::handle_get_agent
			}
            request_toss: {
				inputs: |agent_to: Address, seed_value: u8|,
				outputs: |result: ZomeApiResult<HashString>|,
				handler: toss::handlers::handle_request_toss
			}
            receive_request: {
                inputs: |request: messaging::RequestMsg|,    // TODO: He should probably read it automatically from the message sender. How? Gossip?
                outputs: |result: ZomeApiResult<Address>|,
                handler: toss::handlers::handle_receive_request
            }
            confirm_toss: {
				inputs: |toss: toss::TossSchema, toss_hash: HashString|,         // Q: When using &TossSchema, expects a lifetime parameter. WKO?
				outputs: |result: ZomeApiResult<u32>|,
				handler: toss::handlers::handle_confirm_toss
			}
            get_toss_history: {
				inputs: | |,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: toss::handlers::handle_get_toss_history
			}
            commit_seed: {
                inputs: |seed: toss::SeedSchema|,
                outputs: |result: ZomeApiResult<Address>|,
                handler: toss::handlers::handle_commit_seed
            }
            send_request: {
                inputs: |agent_to: Address, seed_hash: HashString|,
                outputs: |result: ZomeApiResult<String>|,
                handler: toss::handlers::handle_send_request
            }
            // Just for testing purposes - can stay private.
            commit_toss: {
                inputs: |toss: toss::TossSchema|,
                outputs: |result: ZomeApiResult<Address>|,
                handler: toss::handlers::handle_commit_toss
            }
            reveal_toss_result: {
                inputs: |toss_result_addr: Address|,
                outputs: |result: ZomeApiResult<toss::TossResultSchema>|,
                handler: toss::handlers::handle_reveal_toss_result
            }
            prdel: {
                inputs: | |,
                outputs: |result: ZomeApiResult<String>|,
                handler: toss::handlers::handle_prdel
            }                       
    ]
    
    traits: {
        hc_public [
            get_my_address,
            register,
            set_handle,
            get_handles,
            get_handle,
            get_my_handle,
            get_agent,
            request_toss,
            receive_request,
            confirm_toss,
            get_toss_history,
            commit_seed,
            commit_toss,
            send_request,
            reveal_toss_result,
            prdel
        ]
    }
}
