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
    RequestToss
}

/// Represents the message 
/// 
#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct RequestMsg {
    agent_from: Address,
    seed_hash: HashString
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
struct GeneralMsg {
    message_type: MsgType,
    message: String
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
pub fn handle_request_toss(agent_to: Address, seed: u8) -> ZomeApiResult<HashString> {     // Q: Misleading name? Cause request over N2N?
        
    // TODO: Body of this function throws "Unable to call zome function" in the HolochainJS for some reason.
    // !!! TODO: This is the culprit block, causing the above mentioned error.
    // Yes, the rand statements. Why? No idea. External crate linking? Or some kind of buffer / array error?
    // TODO: Just a rough random salt and seed. Change to sth more secure.
    let seed = SeedSchema {
        salt: "del".to_string(), // TODO: randomize - rand::thread_rng().gen_range(0, 10).to_string()?
        seed_value: seed         // Q: Randomize or let user enter thru the UI? rand::thread_rng().gen_range(0, 10)
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

pub fn handle_receive_request(request: RequestMsg) -> ZomeApiResult<Address> {

    // TODO: Send notification, get the data from the UI.
    let my_seed = generate_seed("saltpr".to_string());    
    let my_seed_hash = handle_commit_seed(my_seed).unwrap();        // Q: Better use HashString or Address? (Idiomatic Holochain :) )

    hdk::debug("handle_receive_request() seed_address:");
    hdk::debug(my_seed_hash.clone());

    let toss = TossSchema {
        initiator: request.agent_from.clone(),
        initiator_seed_hash: request.seed_hash.clone(),
        responder: Address::from(AGENT_ADDRESS.to_string()), // Q: Why can't just use the AGENT_ADDRESS?
        responder_seed_hash: my_seed_hash, // HashString::from(&seed_address[12..58]), // TODO: What a dirty trick. BUG?: Shoots down zome function call when e.g. [14..3]. Should?
        call: 1
    };

    hdk::debug("handle_receive_request() toss.responder_seed_hash: ");
    hdk::debug(toss.clone().responder_seed_hash);
        
    let toss_entry = handle_commit_toss(toss.clone());

    hdk::debug("handle_receive_request() toss_entry:");
    hdk::debug(toss_entry.clone().unwrap());

    toss_entry
    // return toss_entry.into();
    //Ok(json!(toss).into())
}

pub fn handle_get_toss_history() -> ZomeApiResult<JsonString> {
        
        let prdel = "prdel".to_string();
        let prdel_hash = HashString::encode_from_str(&prdel.clone(), Hash::SHA2256);
        
        Ok(json!(prdel_hash).into())
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


// Commit functions - should they be a part of the zome? Or private? Or both?
/*
/ fn commit_seed()
/ return: ???
*/
fn handle_commit_seed(seed: SeedSchema) -> ZomeApiResult<Address> {

    // Validate if 9 <= seed >= 0 
    // Generate salt
    // Hash the salt + seed string (?)
    // Commit seed to own chain
    // Return 

    //let entry_arg = JsonString::from(RawString::from(_seed));
    //hdk::debug("Raw seed: ");
    //hdk::debug(entry_arg.clone());

    // let seed_entry = Entry::new(EntryType::App(CTEntryType::seed.to_string()), _seed);
    let seed_entry = Entry::App("seed".into(), seed.into());
    // hdk::debug(seed_entry.to_string());    
    hdk::commit_entry(&seed_entry)

      // Ok(address) => match hdk::link_entries(&AGENT_ADDRESS, &address, "seeds") {
      //      Ok(address) => Ok(address),
      //      Err(hdk_err) => hdk_err 
      //  },
      //  Err(hdk_err) => Err(hdk_err)
    //};
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


// N2N NETWORKING ---------------------------------------------------------------------------------

fn handle_send_request(agent_to: Address, seed_hash: HashString) -> ZomeApiResult<String> {
    
    hdk::debug("hdk::send(): ");
    let msg: RequestMsg = RequestMsg { agent_from: AGENT_ADDRESS.to_string().into(), seed_hash: seed_hash };
    
    let send_msg: GeneralMsg = GeneralMsg { message_type: MsgType::RequestToss, message: json!(msg).to_string() };
    // Q: Is this the right way? Or use JsonString by default?
    // ISSUE: When I use string, I get JSON.serialize error (prolly in the JS). Good? Bad?
    // ISSUE: "agent_from" can be spoofed. Is this fixed yet?
    hdk::send(agent_to, json!(send_msg).to_string(), 10000.into())

    //let result_unwrapped = &result.unwrap();       // Q: How to clone or debug output of ZomeApiResult ?? -> Issue? 
    //hdk::debug(result.unwrap().clone());        // Q: How to work with unwrapping and cloning without violating the move?
    //hdk::debug(result_unwrapped);
    //result
    // message.clone().to_string()
    /* match hdk::send(to_agent, message) {
        Ok(response) => response,
        Err(error) => error.to_string(),
    } */
}

// TODO: Add error handling.
fn process_request(request: String) -> RequestMsg {
    let request_msg: RequestMsg = serde_json::from_str(&request).unwrap();
    request_msg
}

fn process_received_message(payload: String) -> ZomeApiResult<String> {
        
       let msg: GeneralMsg = serde_json::from_str(&payload).unwrap();

        match msg.message_type {
            MsgType::RequestToss => {
                let request_msg = process_request(msg.message);
                // TODO: Error handling.
                Ok(json!(handle_receive_request(request_msg).unwrap()).to_string())
            },
            _ => Ok("process_received_message(): [other message type received]".to_string())
        }

        //let decoded = json!(payload);
        // TODO: Deserialize the payload. How? serde::? Into struct? Tuple?        
        // TODO: Error handling instead of unwrap()
        
        //let request_msg: RequestMsg = match serde_json::from_str(&payload) {
        //    Ok(response) => response,
        //    Err(_) => { "Error "} // RequestMsg { agent_to: "process_received_message(): Error in deserializing the payload.", seed_hash: "Error" } }
        //};

        //let request_msg: RequestMsg = serde_json::from_str(&payload).unwrap();

        // Q: Why the messages don't show? Where is this being actually run?
        //hdk::debug(serde_json::from_str(&payload).unwrap()); // Q: Or do we need some kind of debug signals?
        // let received = handle_receive_request(request_msg.agent_to.clone(), request_msg.seed_hash.clone()); 
        //Ok(received.unwrap().to_string())

        // let msg: RequestMsg = serde_json::from_str(json!(&payload.to_string()).unwrap();

        // panic!("Received");
        // Ok(msg.seed_hash)

        // RequestMsg::try_from(json!(payload).into()); //.deserialize();      
        //let agent = payload.
        // Ok(payload.clone()) //.unwrap().to_string())
        // let decoded = serde_json::
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
        // simply pass back the received value, appended to a modifier
        // format!("{}", payload)
        // TODO: Filter and process just the "toss request" messages in this way.

        process_received_message(payload).unwrap()
        // process_received_message(payload).unwrap()       // Q: Shoudn't be some kind of async / promise sth? What if blocking?
        // receive_toss_request();
        // process_received_message();
        //(|| {                                           // OPTIM: Can I do this without the side effects?
        //    let sender_addr = Address::from("prdel");
        //    let seed_hash = HashString::from("prdelhash");
        //    // let received = handle_receive_request(sender_addr, seed_hash);      // ISSUE: Hangs up w/o signals when call handle_receive_request.
        //    let received = handle_get_my_address();
        //    hdk::debug("HCH/ In the receive FFEI callback.");               // ERR: This doesn't get displayed :o
        //    //hdk::debug(received.unwrap());
        //    received.unwrap().to_string() // payload                   // Q: This does get returned.
        // })()
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
				inputs: |agent_to: Address, seed: u8|,
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

