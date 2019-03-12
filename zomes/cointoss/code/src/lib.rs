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
        cas::content::{ Address, Content, AddressableContent },
        dna::entry_types::Sharing, entry::Entry,
        error::HolochainError,
        json::{ JsonString, RawString },
        hash::HashString
    },
    holochain_wasm_utils::api_serialization::{
        get_entry::{ * },
        get_links::GetLinksResult,
    },
    utils,
    api::AGENT_ID_STR, AGENT_ADDRESS
};

// use hdk::api::AGENT_ADDRESS;
mod entries;
mod anchor;
mod utils;

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


// ----------------------  MESSAGE PROCESSING  ----------------------------------------------------
// P2P MESSAGING- ---------------------------------------------------------------------------------

// @ ----------------------------------------------------------------------------------------------
// @ First line message processing
// @
fn process_received_message(payload: String) -> ZomeApiResult<String> {
        
       let msg: GeneralMsg = serde_json::from_str(&payload).unwrap();

        // Parse the message type and choose the appropriate response--------------------------
        match msg.message_type {
            // Toss request received -------------------------------------------------------
            MsgType::RequestToss => {
                let request_msg = process_request(msg.message);
                let receive_request_result = handle_receive_request(request_msg).unwrap();     // TODO: Fix the error handling. What exactly should this return? Should the unwrap be there?
                Ok(json!(receive_request_result).to_string())
            },
            // Response after the other party commited the toss ----------------------------
            MsgType::TossResponse => {
                let toss_response = process_toss_response_msg(msg.message);
                // TODO: Better naming. handle_toss_response? But, potentially confusing "handle".
                let result = receive_toss_response(toss_response);

                // handle_commit_toss()
                Ok(json!(result).to_string())
            }
            // Other message type received -------------------------------------------------
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
    hdk::send(agent_to, json!(send_msg).to_string(), 20000.into())
}

// TODO: Add error handling.
fn process_request(request: String) -> RequestMsg {
    let request_msg: RequestMsg = serde_json::from_str(&request).unwrap();
    request_msg
}

// @ ----------------------------------------------------------------------------------------------
// @ Processing of the initiation request
// @
fn process_toss_response_msg(message: String) -> TossResponseMsg {
    // Q: Beware some unhealthy coupling / tangling? (i.e. "process_" function shouldn't commit entries I'd say.)

    hdk::debug("process_toss_response_msg()");
    let toss_response: TossResponseMsg = serde_json::from_str(&message).unwrap();

    toss_response
}

fn receive_toss_response(toss_response: TossResponseMsg) -> ZomeApiResult<Address> {
    
    // TODO: Read my seed hash from my chain. Or?
    // TODO: Unify nomenclature reference point - my vs. initiator.
    let my_seed_hash = read_my_seed_hash().unwrap();
    let responder_seed_hash = get_seed_hash(toss_response.responder_seed.clone()).unwrap();

    let toss = TossSchema {
        initiator: Address::from(AGENT_ADDRESS.to_string()),
        initiator_seed_hash: my_seed_hash,
        responder: toss_response.agent_from.clone(),
        responder_seed_hash: responder_seed_hash.clone(),
        call: 1
    };

    let toss_result = handle_commit_toss(toss.clone());
    let toss_addr = toss_result.clone().unwrap();
    // Q: How to verify the toss is right?
    
    // TODO: confirm seed, confirm toss, unify the return results.

    // Q: Do I need to do this, or can I just use received hash? Would defy the purpose tho, right?
    // TODO: Confirm / validate seed hash here? Or just store and then - so everyone can see, can't be refuted, if subterfuge?
    // TODO: Persist my seed? Initiator: me, right? What in case of generalizing for more agents?

    // TODO: reveal seed    
    // TODO: clarify what to return, this prolly doesn't make much sense now

    let seed_confirmed = confirm_seed(toss_response.responder_seed.clone(), responder_seed_hash.clone());
    // TODO: What exactly am I confirming here? I think now I'm just confirming my toss, not his.
    let toss_confirmed = handle_confirm_toss(toss.clone(), toss_addr);

    // TODO: Horrible temp. Rewrite proper error handling.
    let seed_confirmed = match seed_confirmed.unwrap() {
        true => {
            hdk::debug("HCH/ receive_toss_response(): Seed valid!");
            true
        },
        _ => {
             hdk::debug("HCH/ receive_toss_response(): Seed invalid!");
             false
        }
    };

    let toss_confirmed = match toss_confirmed.unwrap() {
        1 => {
            hdk::debug("HCH/ receive_toss_response(): Toss valid!");           
            true
        },
        _ => {
            hdk::debug("HCH/ receive_toss_response(): Toss invalid!");
            false
        }
    };

    // Learning: Can I tell what I need to know to do the eval, from the top of my head? No? Then I don't have it modelled properly yet.
    evaluate_winner(toss_response);

    toss_result
}



// TODO: Find a better, more general name.
// Q: What would be the ideal variable to return, according to best practices?
fn evaluate_winner(toss_response: TossResponseMsg) -> bool {

    // Q: Reveal my seed here?
    let my_seed_addr = read_my_seed_hash().unwrap();
    let my_seed_result = hdk::get_entry(&my_seed_addr).unwrap();     // Q: Why need to do two unwraps? TODO: Error handling.
    let my_seed_entry = my_seed_result.unwrap();
    let my_seed: SeedSchema = hdk::utils::get_as_type::<SeedSchema>(my_seed_entry); // my_seed_entry.content();      // Q: It was neccessary to use hdk::holochain_core_types::cas::content::AddressableContent; Why?

    //pub fn get_as_type<R: TryFrom<AppEntryValue>>(
    //address: Address
    //) -> ZomeApiResult<R>

    // let my_seed_json: Entry::App(, val2) = serde_json::from_str(my_seed.clone().to_string()); // my_seed.into();
    
    // TODO: How would an idiomatic way to write this look like?
    // let my_seed: Entry::App = serde_json::from_str(&Content::from(&my_seed_entry).to_string()).unwrap();
    // let my_seed = my_seed_entry.// entry::GetEntryResultItem::new(my_seed_entry);
    // serde_json::from_str(&Content::from(&my_seed_entry).to_string()).unwrap(); // json!(Content::from(my_seed_entry)).into();
    

    //let my_seed: SeedSchema = my_seed_entry

    hdk::debug("HCH/ evaluate_winner(): my_seed entry");    
    hdk::debug(my_seed.unwrap());
    // hdk::debug(RawString::from(Content::from(&my_seed_entry).to_string()));

    // TODO: Convert the entry to the SeedSchema struct. How?

    // Evaluation: Evaluating whether "call" and "initiator_seed_val + responder_seed_val % 2" have same parity (odd / even)
    let did_i_win = true; //toss_response.call == ((my_seed.seed_value + toss_response.responder_seed.seed_value) % 2);
        
    did_i_win

    // OPTIM: How not to need to query for my seed again?
    // Persistence? Or do it in one function? Or?
}

fn read_my_seed_hash() -> ZomeApiResult<Address> {

    // TODO: Read from my chain through the link.
    // let my_seed_hash = hdk::get_links();
    // Q: What does this do??
    // Q: Use anchor or not? Difference?
    // let anchor_addr = hdk::get?
    // let my_seed_addrs = hdk::get_links(&AGENT_ADDRESS, "seeds")?.addresses().to_owned();

    // Q: Querying local chain (instead of the DHT). Would it make sense to rather query the DHT?
    // As in the original cointoss?
    let my_seed_addrs = hdk::query("seed".into(), 0, 0);
    
    hdk::debug("HCH/ read_my_seed_hash()");
    hdk::debug(my_seed_addrs.clone());

    let addrs = my_seed_addrs.unwrap().clone();
    // TODO: Find out what exactly am I getting here.
    // TODO: Error handling.
    // TODO: In case of multiple plays, figure out to get the actual one.
    Ok(addrs[0].clone())
    // Ok("QmbydC6m2UGJzAaCv6nQWZu4aHJq7YA1BcLBzA4jQ7f7hQ".to_string().into())
    //Ok(AGENT_ADDRESS.to_string().into())
}

// -------------------------------------- TOSS FUNCTIONS ------------------------------------------

pub fn handle_get_my_address() -> ZomeApiResult<Address> {
  
    hdk::debug("HCH/ handle_get_my_address()");
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
    hdk::link_entries(&anchor_address.clone().unwrap(), &AGENT_ADDRESS, "member_tag");

    let handle_entry = Entry::App(
        "handle".into(),
        entries::HandleSchema { handle: name }.into()
    );

    let handle_addr = hdk::commit_entry(&handle_entry);
    hdk::link_entries(&AGENT_ADDRESS, &handle_addr.clone().unwrap(), "handle");

    // Q: When leaving ...)?; syntax, there were problems, the handle_register function didn't return properly... or perhaps returned too soon, IDK
    // TODO: Find what exactly does the "?" do.

    hdk::debug("HCH/ handle_register(): handle_addr");
    hdk::debug(handle_addr.clone().unwrap());

    // Q: How come in handle_set_handle it works w/o the Ok() wrapping??
    handle_addr // Ok(AGENT_ADDRESS.to_string().into())
}


pub fn handle_set_handle(handle_string: String) -> ZomeApiResult<Address> {

    hdk::debug("handle_set_handle()::_handle: ");
    let handle = entries::HandleSchema { handle: handle_string };
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
    hdk::debug(handle_address.clone());
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

// TODO: Fix the documentation format.
/*
 * Initiates the game by doing the first seed commit and sending the request to the agent through gossip (?)
 *
 * @callingType { ZomeApiResult<HashString> }
 * @exposure { public }
 * @param { Address } { agent_to }
 * @param { u8 } { seed_value }
 * @return { ZomeApiResult<HashString> } 
 */
pub fn handle_request_toss(agent_to: Address, seed_value: u8) -> ZomeApiResult<HashString> {     // Q: Misleading name? Cause request over N2N?
        
    // TODO: Body of this function throws "Unable to call zome function" in the HolochainJS for some reason.
    // !!! TODO: This is the culprit block, causing the above mentioned error.
    // Yes, the rand statements. Why? No idea. External crate linking? Or some kind of buffer / array error?
    // TODO: Just a rough random salt and seed. Change to sth more secure.
    let salt = generate_salt();
    
    let seed = SeedSchema {
        salt: salt, // TODO: randomize - rand::thread_rng().gen_range(0, 10).to_string()?
        seed_value: seed_value         // Q: Randomize or let user enter thru the UI? rand::thread_rng().gen_range(0, 10)
     };

    let seed_addr = handle_commit_seed(seed);

    /* let toss = TossSchema {
        initiator: AGENT_ADDRESS.to_string().into(),
        initiator_seed_hash: seed_addr,
        responder: agent_to,
        responder_seed_hash: 
    };

    let toss_addr = handle_commit_toss(); */

    // Q: Sync chaining vs. async waiting? Fragility vs. composability?
    let received = handle_send_request(agent_to, seed_addr.clone().unwrap());
    seed_addr

    // seed_entry
}

fn generate_seed(salt: String) -> SeedSchema {
    SeedSchema {
        salt: salt,
        seed_value: 5                   // TODO: Randomize or whatever, this is a temporary hardcode hack.
    }
}

// TODO: Generate a proper salt!
fn generate_salt() -> String {
    "[to be randomized string or something]".to_string()
}

// TODO: Update the name to reflect the function. Does it really handle just the receiving?
// @ Returns: ??? Toss entry address?? Or?? Custom error?
pub fn handle_receive_request(request: RequestMsg) -> ZomeApiResult<Address> {

    // Commit seed
    hdk::debug("handle_receive_request(): commiting seed");
    let my_seed = generate_seed("saltpr".to_string());    
    let my_seed_hash = handle_commit_seed(my_seed.clone()).unwrap();        // Q: Better use HashString or Address? (Idiomatic Holochain :) )

    let toss = TossSchema {
        initiator: request.agent_from.clone(),
        initiator_seed_hash: request.seed_hash.clone(),
        responder: Address::from(AGENT_ADDRESS.to_string()), // Q: Why can't just use the AGENT_ADDRESS?
        responder_seed_hash: my_seed_hash, // HashString::from(&seed_address[12..58]), // TODO: What a dirty trick. BUG?: Shoots down zome function call when e.g. [14..3]. Should?
        call: 1
    };

    hdk::debug("handle_receive_request(): toss initiator: ");
    hdk::debug(toss.initiator.clone().to_string());
    
    // Commit toss
    let toss_entry = handle_commit_toss(toss.clone());

    hdk::debug("handle_receive_request(): toss_entry:");
    hdk::debug(toss_entry.clone().unwrap());

    // Send call / response triplet - responder_seed, toss_hash, call
    // Q: Decomposition. Should be called from here or from some "central" function?
    // Q: Am I, as B, already revealing the seed here? Should I?
    let response_msg = TossResponseMsg {
        agent_from: AGENT_ADDRESS.to_string().into(),
        responder_seed: my_seed.clone(),                                
        toss_hash: toss_entry.clone().unwrap(),
        call: 1     // TODO: Randomize or let the call be entered otherwise.
    };

    let send_result = send_response(toss.initiator.clone(), response_msg);
    
    hdk::debug("handle_receive_request(): send_response result: ");
    hdk::debug(send_result.unwrap());        // Q: Receiving {Ok: {Ok: ___}} construction. How come? Wrapping?

    // Q: What now here?
    toss_entry
}

fn send_response(agent_to: Address, response_msg: TossResponseMsg) -> ZomeApiResult<String> {

    hdk::debug("send_response(): ");
    
    let wrapped_msg = GeneralMsg {
        agent_from: AGENT_ADDRESS.to_string().into(),
        message_type: MsgType::TossResponse,
        message: json!(response_msg).to_string()
    };

    let response = hdk::send(agent_to, json!(wrapped_msg).to_string(), 20000.into());
    hdk::debug(response.clone());
    response
}

pub fn handle_get_toss_history() -> ZomeApiResult<JsonString> {              
    Ok(json!("[not implemented yet]".to_string()).into())
}

fn handle_commit_seed(seed: SeedSchema) -> ZomeApiResult<Address> {
    // TODO: Validate if 9 <= seed >= 0?

    let seed_entry = Entry::App("seed".into(), seed.into());
    let seed_address = hdk::commit_entry(&seed_entry);

    // Q: Link it to an anchor? Or?

    // Q: Naming conventions: seed_address or seed_hash?
    // Q: Borrowing and unwrapping - what's the operator priorities?
    hdk::link_entries(&AGENT_ADDRESS, &seed_address.clone().unwrap(), "seeds");

    seed_address
    // Q: What about multiple plays?
}

// TODO: Generalize through types - <T>
fn get_seed_hash(seed: SeedSchema) -> ZomeApiResult<HashString> {
    let seed_entry = Entry::App("seed".into(), seed.into());
    let seed_hash_generated = hdk::entry_address(&seed_entry);
    seed_hash_generated
}

// Q: Better use reference, or pass the whole struct?
fn get_toss_hash(toss: TossSchema) -> ZomeApiResult<HashString> {
    let toss_entry = Entry::App("toss".into(), toss.into());
    let toss_hash_generated = hdk::entry_address(&toss_entry);
    toss_hash_generated
}

// TODO: Again, implement a version for general types <T>
fn confirm_seed(seed: SeedSchema, seed_hash: HashString) -> ZomeApiResult<bool> {
    
    let seed_hash_generated = get_seed_hash(seed).unwrap();
    hdk::debug("confirm_seed(): ");
    hdk::debug(seed_hash.clone());
    hdk::debug(seed_hash_generated.clone());

    // TODO: Error handling.
    Ok((seed_hash_generated == seed_hash))
}

// TODO: Won't confirm now, because I'm not storing my seed yet.
// TODO: JsonString doesn't make much sense here. Also, should be public?
fn handle_confirm_toss(toss: TossSchema, toss_hash: HashString) -> ZomeApiResult<u32> {
    
    let toss_hash_generated = get_toss_hash(toss).unwrap();
    hdk::debug("confirm_toss(): ");
    hdk::debug(toss_hash.clone());
    hdk::debug(toss_hash_generated.clone());

    // !!! TODO: This - horrible temp. (ZomeApiResult doesn't implement bool.)
    /* Ok( match (toss_hash_generated == toss_hash) {
        true => json!("{ confirmed: true }").into(),
        false => json!("{ confirmed: false }").into()
    }) */
    Ok(1)
}

pub fn handle_commit_toss(toss: TossSchema) -> ZomeApiResult<Address> {

    // TODO: Validate it the toss has the right format etc.?
    // Consider tying it with the validation logic somehow?

    let toss_entry = Entry::App("toss".into(), toss.into());
    let toss_address_result = hdk::commit_entry(&toss_entry);
    toss_address_result

    // Ok(address) => match hdk::link_entries(&AGENT_ADDRESS, &address, "tosses") {
    // Ok(address) => json!({ "address": address }).into(),
    // Err(hdk_err) => { hdk_err.into() }
    // },
    // Err(hdk_err) => hdk_err.into()
}


// ZOME DEFINITION --------------------------------------------------------------------------------
define_zome! {
    entries: [

        entries::handle_definition(),
        entries::toss_definition(),
        entries::toss_result_definition(),
        entries::seed_definition(),
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
            {
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
            register: {
                inputs: |handle: String|,
                outputs: |result: ZomeApiResult<Address>|,
                handler: handle_register 
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
				inputs: |toss: TossSchema, toss_hash: HashString|,         // Q: When using &TossSchema, expects a lifetime parameter. WKO?
				outputs: |result: ZomeApiResult<u32>|,
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

