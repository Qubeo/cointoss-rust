#![feature(try_from)]
use std::convert::TryFrom;
#[macro_use]
extern crate hdk;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate holochain_core_types_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate multihash;

use std::io;
use rand::Rng;
use multihash::{encode, decode, Hash};
use std::fmt;
// use snowflake;
use hdk::{
    // self,
    error::ZomeApiResult,
    holochain_core_types::{
        cas::content::Address, dna::entry_types::Sharing, entry::Entry, error::HolochainError, json::{ JsonString, RawString }, hash::HashString 
    },
    holochain_wasm_utils::api_serialization::{
        get_entry::GetEntryOptions, get_links::GetLinksResult,
    },
    api::AGENT_ID_STR, AGENT_ADDRESS
};

// use hdk::api::AGENT_ADDRESS;
mod entries;
use crate::entries::{CTEntryType, TossSchema, TossResultSchema, SeedSchema, AddrSchema};

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
enum MsgType {
    WannaPlay,
    RequestToss,
    TossResponse
}

/// Represents the message 
/// 
/// TODO: Add "agent_from" here?
#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
struct GeneralMsg {
    agent_from: Address,
    message_type: MsgType,
    message: String
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct RequestMsg {
    agent_from: Address,
    seed_hash: HashString
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct TossResponseMsg {
    agent_from: Address,
    pub responder_seed: SeedSchema,
    pub toss_hash: HashString,
    call: u8
}

// ----------------------  MESSAGE PROCESSING -----------------------------------------------------
// N2N NETWORKING ---------------------------------------------------------------------------------


// @ ----------------------------------------------------------------------------------------------
// @ First line message processing
// @
fn process_received_message(payload: String) -> ZomeApiResult<String> {
        
       let msg: GeneralMsg = serde_json::from_str(&payload).unwrap();

        // Parse the message type and choose the appropriate response
        match msg.message_type {
            // Toss request received
            MsgType::RequestToss => {
                let request_msg = process_request(msg.message);
                let receive_request_result = handle_receive_request(request_msg).unwrap();     // TODO: Fix the error handling. What exactly should this return? Should the unwrap be there?
                Ok(json!(receive_request_result).to_string())
            },
            // Response after the other party commited the toss
            MsgType::TossResponse => {
                let toss_address = process_toss_response(msg.message);
                // handle_commit_toss()
                Ok(json!(toss_address).to_string())
            }
            _ => Ok("process_received_message(): [other message type received]".to_string())
        }
}

// @ ----------------------------------------------------------------------------------------------
// @ Sends the request via P2P messaging
// @
fn handle_send_request(agent_to: Address, seed_hash: HashString) -> ZomeApiResult<String> {
    
    hdk::debug("hdk::send(): ");
    let msg: RequestMsg = RequestMsg { agent_from: AGENT_ADDRESS.to_string().into(), seed_hash: seed_hash };    
    let send_msg: GeneralMsg = GeneralMsg { agent_from: AGENT_ADDRESS.to_string().into(), message_type: MsgType::RequestToss, message: json!(msg).to_string() };
    // Q: Is this the right way? Or use JsonString by default?
    // ISSUE: When I use string, I get JSON.serialize error (prolly in the JS). Good? Bad?
    // ISSUE: "agent_from" can be spoofed. Is this fixed yet?    
    hdk::send(agent_to, json!(send_msg).to_string(), 10000.into())
}

// TODO: Add error handling.
fn process_request(request: String) -> RequestMsg {
    let request_msg: RequestMsg = serde_json::from_str(&request).unwrap();
    request_msg
}


// TODO: This prolly should be (connected to) confirm seed.
fn generate_seed_hash(seed: SeedSchema) -> ZomeApiResult<HashString> {
    let seed_hash = HashString::encode_from_str(&"prdel".to_string(), Hash::SHA2256);    // !!! TODO: Use real HCH hashing. This temporary hardcoded nonsense.
    Ok(seed_hash)
    // TODO: Hash it, or just commit it and get the hash in one step?
    // Q: Beware some unhealthy coupling / tangling? (i.e. "process_" function shouldn't commit entries I'd say.)
}

// @ ----------------------------------------------------------------------------------------------
// @ Processing of the initiation request
// @
fn process_toss_response(message: String) -> ZomeApiResult<Address> {
    let toss_response: TossResponseMsg = serde_json::from_str(&message).unwrap();

    // Q: Do I need to do this, or can I just use received hash? Would defy the purpose tho, right?
    //let initiator: Address = ???; // TODO: get agent_from somehow. How?
    let seed_hash = generate_seed_hash(toss_response.responder_seed).unwrap();

    // TODO: Confirm / validate seed hash here? Or just store and then - so everyone can see, can't be refuted, if subterfuge?

    // TODO: Persist my seed? Initiator: me, right? What in case of generalizing for more agents?
    let toss = TossSchema {
        initiator: Address::from(AGENT_ADDRESS.to_string()),
        initiator_seed_hash: seed_hash.clone(),  // !!! TODO: My seed hash. This is just a VERY temp hack. Am I persisting my seed somewhere? Links?
        responder: toss_response.agent_from.clone(),
        responder_seed_hash: seed_hash.clone(), // HashString::from(&seed_address[12..58]), // TODO: What a dirty trick. BUG?: Shoots down zome function call when e.g. [14..3]. Should?
        call: 1
    };

    let toss_result = handle_commit_toss(toss.clone());
    
    toss_result    
}

// -------------------------------------- TOSS FUNCTIONS ------------------------------------------

pub fn handle_get_my_address() -> ZomeApiResult<Address> {
  
    hdk::debug("HCH/ handle_get_my_address()");
    Ok(AGENT_ADDRESS.to_string().into())
}

/*
 * Returns the list of Ratings of a particular Ratee.
 *
 * @callingType {json}
 * @exposure {public}
 * @param {json} { "Ratee": "<agenthash>" }
 * @return {json}[] {"Result": true, "Entries": ["Rater": "<hash>", "Rating": "<string>"]}
 */
pub fn handle_set_handle(handle_string: String) -> ZomeApiResult<Address> {
   
    // let handle_hash = HashString::encode_from_str(&_handle, Hash::SHA2256);

    // Q: It works with the JsonString(RawString) wrapping. How come?
    // What are the allowed forms for the argument? Can I see the memory / byte structure somewhere?    
    // let raw_handle = JsonString::from(RawString::from(_handle.clone()));

    // TODO: Propose a "formatted" hdk::debug! macro PR?
    hdk::debug("handle_set_handle()::_handle: ");
    // hdk::debug(raw_handle.clone());
   
    // let post_entry = Entry::App("post".into(), Post::new(&content, "now").into());
    let handle = entries::HandleSchema { handle: handle_string };

    let handle_entry = Entry::App("handle".into(), handle.clone().into());
    // hdk::debug(handle_entry.to_string());
    
    // Q: It seems having this in genesis doesn't work - throws an exception within the holochain-nodejs. How to?
    // let handle_address = hdk::commit_entry(&handle_entry);
    
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
    hdk::debug(handle_address.clone());

    // Q: Still not completely clear on the error handling.
    handle_address
}

// returns all the handles in the directory
pub fn handle_get_handles() -> ZomeApiResult<JsonString> {
    Ok("prdelgethandle".into())
}

// returns the handle of an agent by looking it up on the user's DHT entry, the last handle will be the current one?
pub fn handle_get_handle(_handle: HashString) -> ZomeApiResult<JsonString> {
    Ok(HashString::new().into())
}

pub fn handle_get_my_handle() -> ZomeApiResult<JsonString> {         
    Ok("QmWLKuaVVLpHbCLiHuwjpuZaGpY3436HWkKKaqAmz2Axxh".into())
}

// gets the AgentID (userAddress) based on handle
pub fn handle_get_agent(_handle: HashString) -> ZomeApiResult<JsonString> {  
    Ok(Address::new().into())
}

/*
/ pub fn handle_request_toss()
/ Request the toss - initiating the game by doing the first seed commit and sending the request to the agent through gossip (?)
*/

// pub fn handle_request_toss(_agent_from: Address) -> ZomeApiResult<JsonString> {
pub fn handle_request_toss(agent_to: Address, seed_value: u8) -> ZomeApiResult<HashString> {     // Q: Misleading name? Cause request over N2N?
        
    // TODO: Body of this function throws "Unable to call zome function" in the HolochainJS for some reason.
    // !!! TODO: This is the culprit block, causing the above mentioned error.
    // Yes, the rand statements. Why? No idea. External crate linking? Or some kind of buffer / array error?
    // TODO: Just a rough random salt and seed. Change to sth more secure.
    let seed = SeedSchema {
        salt: "del".to_string(), // TODO: randomize - rand::thread_rng().gen_range(0, 10).to_string()?
        seed_value: seed_value         // Q: Randomize or let user enter thru the UI? rand::thread_rng().gen_range(0, 10)
     };

    // hdk::debug("Generated seed: ");
    // hdk::debug(seed.seed_value.to_string());

    let seed_entry = handle_commit_seed(seed);

    // Q: Can I call gossip functions from here? If yes, how? Or should I do it from the outside of the container?
    // TODO: Reconsider the design when I get the info. For now, passing the commited seed address to the JS.
     
    seed_entry
}

fn generate_seed(salt: String) -> SeedSchema {
    SeedSchema {
        salt: salt,
        seed_value: 5                   // TODO: Randomize or whatever, this is a temporary hardcode hack.
    }
}

// TODO: Update the name to reflect the function. Does it really handle just the receiving?
// @ Returns: ??? Toss entry address?? Or?? Custom error?
pub fn handle_receive_request(request: RequestMsg) -> ZomeApiResult<Address> {

    // TODO: Send notification, get the data from the UI.
    // generate_seed -> generate_toss -> commit_toss -> send_response 

    // Commit seed
    let my_seed = generate_seed("saltpr".to_string());    
    let my_seed_hash = handle_commit_seed(my_seed.clone()).unwrap();        // Q: Better use HashString or Address? (Idiomatic Holochain :) )

    let toss = TossSchema {
        initiator: request.agent_from.clone(),
        initiator_seed_hash: request.seed_hash.clone(),
        responder: Address::from(AGENT_ADDRESS.to_string()), // Q: Why can't just use the AGENT_ADDRESS?
        responder_seed_hash: my_seed_hash, // HashString::from(&seed_address[12..58]), // TODO: What a dirty trick. BUG?: Shoots down zome function call when e.g. [14..3]. Should?
        call: 1
    };

    hdk::debug("handle_receive_request() toss initiator: ");
    hdk::debug(toss.initiator.clone().to_string());
    
    // Commit toss
    let toss_entry = handle_commit_toss(toss.clone());

    hdk::debug("handle_receive_request() toss_entry:");
    hdk::debug(toss_entry.clone().unwrap());

    // Send call / response triplet - responder_seed, toss_hash, call
    // Q: Decomposition. Should be called from here or from some "central" function?
    // send_response();
    let response_msg = TossResponseMsg {
        agent_from: AGENT_ADDRESS.to_string().into(),
        responder_seed: my_seed.clone(),                                
        toss_hash: toss_entry.clone().unwrap(),
        call: 1     // TODO: Randomize or let the call be entered otherwise.
    };

    let send_result = send_response(toss.initiator.clone(), response_msg);
    
    hdk::debug("handle_receive_request() send_response result: ");
    hdk::debug(send_result);

    toss_entry
}

fn send_response(agent_to: Address, response_msg: TossResponseMsg) -> ZomeApiResult<String> {

    let wrapped_msg = GeneralMsg {
        agent_from: AGENT_ADDRESS.to_string().into(),
        message_type: MsgType::TossResponse,
        message: json!(response_msg).to_string()
    };

    hdk::send(agent_to, json!(wrapped_msg).to_string(), 10000.into())
}

pub fn handle_get_toss_history() -> ZomeApiResult<JsonString> {
        
        // let prdel_hash = HashString::encode_from_str(&"haf".to_string(), Hash::SHA2256);        
        Ok(json!("haf".to_string()).into())
}

fn handle_confirm_toss(toss: TossSchema) -> ZomeApiResult<JsonString> {
  
    hdk::debug("handle_confirm_toss(): _toss: ");
    hdk::debug(toss.clone());
    
    // TODO: The toss confirmation code here. Do the values fit?
  
    let toss_entry = Entry::App("toss".into(), toss.into()); // Q: my_key? &my_key? Nebo "prdel"?
    
    // Q: It seems having this in genesis doesn't work - throws an exception within the holochain-nodejs.
    // TODO: Ask in Mattermost.

    let toss_address: JsonString = match hdk::commit_entry(&toss_entry) {
         
        // Ok(address) => match hdk::link_entries(&AGENT_ADDRESS, &address, "toss") {
            Ok(address) => json!({ "address": address }).into(),
            Err(hdk_err) => { hdk_err.into() }
        // },
        // Err(hdk_err) => hdk_err.into()
    };
    
    hdk::debug("handle_confirm_toss(): toss_address: ");
    hdk::debug(toss_address.clone());

    Ok(toss_address.into()) //toss_address.into();
}


fn handle_commit_seed(seed: SeedSchema) -> ZomeApiResult<Address> {

    // TODO: Validate if 9 <= seed >= 0?
    
    // let entry_arg = JsonString::from(RawString::from(_seed));
    // hdk::debug("Raw seed: ");
    // hdk::debug(entry_arg.clone());

    let seed_entry = Entry::App("seed".into(), seed.into());
    hdk::commit_entry(&seed_entry)

    // Ok(address) => match hdk::link_entries(&AGENT_ADDRESS, &address, "seeds") {
    //      Ok(address) => Ok(address),
    //      Err(hdk_err) => hdk_err 
    //  },
    //  Err(hdk_err) => Err(hdk_err)
    // };
}

fn confirm_seed() -> ZomeApiResult<JsonString> {
    Ok(HashString::new().into())
}

pub fn handle_commit_toss(toss: TossSchema) -> ZomeApiResult<Address> {

    // TODO: Validate it the toss has the right format etc.?
    // Consider tying it with the validation logic somehow?

    let toss_entry = Entry::App("toss".into(), toss.into());
    let toss_address_result = hdk::commit_entry(&toss_entry);
    toss_address_result
}

fn _commit_toss(toss: TossSchema) -> ZomeApiResult<Address> {

    let toss_entry = Entry::App("toss".into(), toss.into());
    let toss_address_result = hdk::commit_entry(&toss_entry); // {

        // Ok(address) => match hdk::link_entries(&AGENT_ADDRESS, &address, "tosses") {
            //Ok(address) => json!({ "address": address }).into(),
            //Err(hdk_err) => { hdk_err.into() }
        // },
        // Err(hdk_err) => hdk_err.into()
    // };

    // hdk::debug("commit_toss(): toss_entry: ");
    // hdk::debug(toss_address_result.clone().unwrap().to_string());

    toss_address_result
}

fn generate_salt() -> ZomeApiResult<JsonString> {
    Ok(HashString::new().into())
}



// ZOME DEFINITION --------------------------------------------------------------------------------
define_zome! {
    entries: [

        entries::handle_definition(),
        entries::toss_definition(),
        entries::toss_result_definition(),
        entries::seed_definition()

        // TODO: Q: It seems I can define multiple entries of the same type / content. Isn't this a bug?

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
         // TODO workaround around not-yet-implemented hdk::api::AGENT_ADDRESS
         // Commit a tomporarily created agent hash to my chain and return the entry address?
            {
                //handle_set_handle(&AGENT_ADDRESS);
                Ok(())
            }
         }
    
    receive: |payload| {

        process_received_message(payload).unwrap() // Q: Shoudn't be some kind of async / promise sth? What if blocking?
     }

    functions: [
			get_my_address: {
				inputs: | |,
				outputs: |result: ZomeApiResult<Address>|,       // Q: Not sure about the return type. HashString? Or everything here JsonString?
				handler: handle_get_my_address                      // Q: If everything is expected to be JsonString, why ask the type at all - verbose?
			}
    		set_handle: {
				inputs: |handle: String|,
				outputs: |result: ZomeApiResult<Address>|,      // Q: How does this syntax work? Closure arguments without follow up function body? :o
				handler: handle_set_handle
			}
            get_handles: {
				inputs: | |,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: handle_get_handles
			}
            get_handle: {
				inputs: |agent_from: HashString|,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: handle_get_handle
			}
            get_my_handle: {
				inputs: | |,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: handle_get_my_handle
			}
            get_agent: {
				inputs: |handle: HashString|,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: handle_get_agent
			}
            request_toss: {
				inputs: |agent_to: Address, seed_value: u8|,
				outputs: |result: ZomeApiResult<HashString>|,
				handler: handle_request_toss
			}
            receive_request: {
                inputs: |request: RequestMsg|,    // TODO: He should probably read it automatically from the message sender. How? Gossip?
                outputs: |result: ZomeApiResult<Address>|,
                handler: handle_receive_request
            }
            confirm_toss: {
				inputs: |toss: TossSchema|,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: handle_confirm_toss
			}
            get_toss_history: {
				inputs: | |,
				outputs: |result: ZomeApiResult<JsonString>|,
				handler: handle_get_toss_history
			}
            commit_seed: {
                inputs: |seed: SeedSchema|,
                outputs: |result: ZomeApiResult<Address>|,
                handler: handle_commit_seed
            }
            send_request: {
                inputs: |agent_to: Address, seed_hash: HashString|,
                outputs: |result: ZomeApiResult<String>|,
                handler: handle_send_request
            }
            test_fn: {
                inputs: |message: String|,
                outputs: |result: String|,
                handler: handle_test_fn
            }
            // Just for testing purposes - can stay private.
            commit_toss: {
                inputs: |toss: TossSchema|,
                outputs: |result: ZomeApiResult<Address>|,
                handler: handle_commit_toss
            }                       
    ]
    
    traits: {
        hc_public [
            get_my_address,
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
            test_fn
        ]
    }
}



pub fn handle_test_fn(message: String) -> String {

    // ISSUE: This seems to kill the instance somehow, but I don't get the error report / log. Why?
    // let request_msg: RequestMsg = serde_json::from_value(json!("{prdel:housky}")).unwrap();
    // let request_msg: RequestMsg = serde_json::from_str("{ron: 3}").unwrap();
    
    // TODO: Zjistit, jestli to failuje i mimo HCH aplikaci, prostě jen v Rustu nebo WASM Rustu.
    let foo_json = json!({"agent_to": "prdel", "message": "housky"});
    let msg: RequestMsg = serde_json::from_str(&foo_json.to_string()).unwrap();

    hdk::debug("HCH/ RequestMsg: ");
    hdk::debug(msg);
  
    // hdk::debug(serde_json::from_str("{ron: 3}").unwrap().to_string());

    return "prdel returned".to_string();
    
    //hdk::debug(serde_json::from_str(&payload).unwrap()); // Q: Or do we need some kind of debug signals?
    //let received = handle_receive_request(request_msg.agent_to.clone(), request_msg.seed_hash.clone()); 
}

