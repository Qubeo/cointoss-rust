use hdk::error::ZomeApiResult;
use hdk::AGENT_ADDRESS;
use hdk::holochain_core_types::{
    hash::HashString,
    entry::Entry,
    cas::content::Address,
    json::{ JsonString, RawString },
};

use crate::player::{ HandleSchema };

pub fn handle_get_my_address() -> ZomeApiResult<Address> {
  
    let _debug_res = hdk::debug("HCH/ handle_get_my_address()");
    Ok(AGENT_ADDRESS.to_string().into())
}

/*
 * Registers the handle and connects it to the anchor
 *
 * @callingType {json}
 * @exposure {public}
 * @param {json} { "Ratee": "<agenthash>" }
 * @return {json}[] {"Result": true, "Entries": ["Rater": "<hash>", "Rating": "<string>"]}
 */
pub fn handle_register(name: String) -> ZomeApiResult<Address> {

    let anchor_entry = Entry::App(
        "anchor".into(),
        RawString::from("member_directory").into(),
    );

    // TODO: Check if the handle already exists?
    
    // Q: Why creating the anchor here? Perhaps just once? Perhaps in genesis?
    //    Or even in genesis of the application - just the first agent?
    // Link the anchor to the new agent under the "member_tag" tag (?)
    let anchor_address = hdk::commit_entry(&anchor_entry);
    let _link_res = hdk::link_entries(&anchor_address.clone().unwrap(), &AGENT_ADDRESS, "member_tag");

    let handle_entry = Entry::App(
        "handle".into(),
        HandleSchema { handle: name }.into()
    );

    let handle_addr = hdk::commit_entry(&handle_entry);
    let _link_res = hdk::link_entries(&AGENT_ADDRESS, &handle_addr.clone().unwrap(), "handle");

    // Q: When leaving the ...)?; syntax, there were problems, the handle_register function didn't return properly... or perhaps returned too soon, IDK
    // TODO: Find what exactly does the "?" do.

    let _debug_res = hdk::debug("HCH/ handle_register(): handle_addr");
    let _debug_res = hdk::debug(handle_addr.clone().unwrap());

    // Q: How come in handle_set_handle it works w/o the Ok() wrapping??
    handle_addr
}


pub fn handle_set_handle(handle_string: String) -> ZomeApiResult<Address> {

    let _debug_res = hdk::debug("handle_set_handle()::_handle: ");
    let handle = HandleSchema { handle: handle_string };
    let handle_entry = Entry::App("handle".into(), handle.clone().into());    
    let handle_address = hdk::commit_entry(&handle_entry);
    /* {
        // Ok(address) => match hdk::link_entries(&AGENT_ADDRESS, &address, "handle") {
            // Ok(address) => json!({ "address": address }).into(),
            Ok(address) => address,
            Err(hdk_err) => { hdk_err }
        // },
        // Err(hdk_err) => hdk_err.into()
    }; */    
    // let my_key_entry_address = match hdk::get_entry(hdk::entry_address(&my_key_entry)) {
    let _debug_res = hdk::debug(handle_address.clone());
    // Q: Still not completely clear on the error handling.
    handle_address
}

// Returns all the handles in the directory
pub fn handle_get_handles() -> ZomeApiResult<JsonString> {
    Ok("[not yet implemented]".into())
}

// Returns the handle of an agent by looking it up on the user's DHT entry, the last handle will be the current one?
pub fn handle_get_handle(_handle: HashString) -> ZomeApiResult<JsonString> {
    Ok(HashString::new().into())
}

pub fn handle_get_my_handle() -> ZomeApiResult<JsonString> {         
    Ok("[not yet implemented]".into())
}

// Gets the AgentID (userAddress) based on handle
pub fn handle_get_agent(_handle: HashString) -> ZomeApiResult<JsonString> {  
    Ok(Address::new().into())
}