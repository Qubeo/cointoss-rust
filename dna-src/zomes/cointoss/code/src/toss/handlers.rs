
use hdk::error::ZomeApiResult;
use hdk::AGENT_ADDRESS;
use hdk::holochain_core_types::{
    hash::HashString,
    entry::Entry,
    cas::content::Address,
    json::{ JsonString },
};

// Q: Do I need to do this?
use crate::toss::{ TossSchema, SeedSchema, TossResultSchema };
use crate::messaging::{ MsgType, TossResponseMsg, GeneralMsg, RequestMsg };
use crate::messaging::{ process_general_msg, process_request_msg, process_toss_response_msg };


// @ ----------------------------------------------------------------------------------------------
// @ Sends the request via P2P messaging
// @
pub fn handle_send_request(agent_to: Address, seed_hash: HashString) -> ZomeApiResult<String> {
    
    let _debug_res = hdk::debug("hdk::send(): ");
    let msg: RequestMsg = RequestMsg { agent_from: AGENT_ADDRESS.to_string().into(), seed_hash: seed_hash };    
    let send_msg: GeneralMsg = GeneralMsg { agent_from: AGENT_ADDRESS.to_string().into(), message_type: MsgType::RequestToss, message: json!(msg).to_string() };
    // Q: Is this the right way? Or use JsonString by default?
    // ISSUE: When I use string, I get JSON.serialize error (prolly in the JS). Good? Bad?
    // ISSUE: "agent_from" can be spoofed. Is this fixed yet?    
    hdk::send(agent_to, json!(send_msg).to_string(), 20000.into())
}

// @ ----------------------------------------------------------------------------------------------
// @ First line message processing
// @
// Q: Design patterns - how to break those apart, decompose cleanly, idiomatically?
//      Perhaps use closures / callback, instead of calling function from another module?
pub fn process_received_message(payload: String) -> ZomeApiResult<String> {
        
       let msg: GeneralMsg = process_general_msg(payload);      

        // Parse the message type and choose the appropriate response--------------------------
        match msg.message_type {
            // Toss request received -------------------------------------------------------
            MsgType::RequestToss => {
                let request_msg = process_request_msg(msg.message);     
                let receive_request_result = handle_receive_request(request_msg).unwrap(); // TODO: Fix the error handling. What exactly should this return? Should the unwrap be there?
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
        
    let salt = generate_salt();
    
    let seed = SeedSchema {
        salt: salt, // TODO: randomize.
        seed_value: seed_value  // Q: Randomize or let user enter thru the UI?
     };

    let seed_addr = handle_commit_seed(seed);

    // Q: Sync chaining vs. async waiting? Fragility vs. composability?
    let _received = handle_send_request(agent_to, seed_addr.clone().unwrap());
    
    // TODO: What to return here, ideally?
    seed_addr
}

// TODO: Try making it interactive, pulling it from th UI instead? Or getting it somewhere else "outside"?
// Q: Should all this be public?
pub fn generate_seed(salt: String) -> SeedSchema {
    SeedSchema {
        salt: salt,
        seed_value: generate_random_seedval()
    }
}

pub fn generate_random_seedval() -> u8 {
    (generate_pseudo_random() % 9) as u8
}

// TODO: Possibly generates only even numbers? :o
pub fn generate_pseudo_random() -> usize {
    let ptr = Box::into_raw(Box::new(123));
    ptr as usize
    // rand::thread_rng().gen::<u8>()
    // let mut rng = pseudorand::PseudoRand::new(0);    
    // rng.rand_range(0, 255) as u32
}

// TODO: Generate a proper salt.
pub fn generate_salt() -> String {
    "[to be randomized string or something]".to_string()
}


pub fn send_response(agent_to: Address, response_msg: TossResponseMsg) -> ZomeApiResult<String> {

    let _debug_res = hdk::debug("send_response(): ");
    
    let wrapped_msg = GeneralMsg {
        agent_from: AGENT_ADDRESS.to_string().into(),
        message_type: MsgType::TossResponse,
        message: json!(response_msg).to_string()
    };

    let response = hdk::send(agent_to, json!(wrapped_msg).to_string(), 20000.into());
    let _debug_res = hdk::debug(response.clone());
    response
}

pub fn handle_get_toss_history() -> ZomeApiResult<JsonString> {              
    Ok(json!("[not implemented yet]".to_string()).into())
}

pub fn handle_commit_seed(seed: SeedSchema) -> ZomeApiResult<Address> {
    // TODO: Validate if 9 <= seed >= 0?

    let seed_entry = Entry::App("seed".into(), seed.into());
    let seed_address = hdk::commit_entry(&seed_entry);

    // Q: Link it to an anchor? Or?

    // Q: Naming conventions: seed_address or seed_hash?
    // Q: Borrowing and unwrapping - what's the operator priorities?
    let _link_res = hdk::link_entries(&AGENT_ADDRESS, &seed_address.clone().unwrap(), "seeds");

    seed_address
    // Q: What about multiple plays?
}

// TODO: Generalize through types - <T>? How?
pub fn get_seed_hash(seed: SeedSchema) -> ZomeApiResult<HashString> {
    hdk::entry_address(&Entry::App("seed".into(), seed.into()))
}

// Q: Better use reference, or pass the whole struct?
fn get_toss_hash(toss: TossSchema) -> ZomeApiResult<HashString> {
    hdk::entry_address(&Entry::App("toss".into(), toss.into()))
}

// TODO: Again, implement a version for general types <T>
pub fn confirm_seed(seed: SeedSchema, seed_hash: HashString) -> ZomeApiResult<bool> {
    
    let seed_hash_generated = get_seed_hash(seed).unwrap();
    let _debug_res = hdk::debug("confirm_seed(): ");
    let _debug_res = hdk::debug(seed_hash.clone());
    let _debug_res = hdk::debug(seed_hash_generated.clone());

    // TODO: Error handling.
    Ok(seed_hash_generated == seed_hash)
}

// TODO: Won't confirm now, because I'm not storing my seed yet.
// TODO: JsonString doesn't make much sense here. Also, should be public?
pub fn handle_confirm_toss(toss: TossSchema, toss_hash: HashString) -> ZomeApiResult<u32> {
    
    let toss_hash_generated = get_toss_hash(toss).unwrap();
    
    // !!! TODO: This - horrible temp. (ZomeApiResult doesn't implement bool.)
    /* Ok( match (toss_hash_generated == toss_hash) {
        true => json!("{ confirmed: true }").into(),
        false => json!("{ confirmed: false }").into()
    }) */
    Ok((toss_hash_generated == toss_hash) as u32)
}

pub fn handle_commit_toss(toss: TossSchema) -> ZomeApiResult<Address> {

    // TODO: Validate it the toss has the right format etc.?
    // Consider tying it with the validation logic somehow?

    let toss_entry = Entry::App("toss".into(), toss.into());
    let toss_address_result = hdk::commit_entry(&toss_entry);
    toss_address_result

    // TODO: Add to toss history.
    // Ok(address) => match hdk::link_entries(&AGENT_ADDRESS, &address, "tosses") {
}

// TODO: Update the name to reflect the function. Does it really handle just the receiving?
// @ Returns: ??? Toss entry address?? Or?? Custom error?
pub fn handle_receive_request(request: RequestMsg) -> ZomeApiResult<Address> {

    // Commit seed
    let _debug_res = hdk::debug("HCH/ handle_receive_request(): commiting seed");
    let my_seed = generate_seed("saltpr".to_string());    
    let my_seed_hash = handle_commit_seed(my_seed.clone()).unwrap();        // Q: Better use HashString or Address? (Idiomatic Holochain :) )
    
    let _debug_res = hdk::debug(format!("HCH/ seed_value: {}", &my_seed.seed_value));

    let toss = TossSchema {
        initiator: request.agent_from.clone(),
        initiator_seed_hash: request.seed_hash.clone(),
        responder: Address::from(AGENT_ADDRESS.to_string()), // Q: Why can't just use the AGENT_ADDRESS?
        responder_seed_hash: my_seed_hash,
        call: (generate_pseudo_random() % 2) as u8     // TODO: Randomize
    };
    
    // Commit toss
    let toss_entry = handle_commit_toss(toss.clone());

    let _debug_res = hdk::debug("handle_receive_request(): toss_entry:");
    let _debug_res = hdk::debug(toss_entry.clone().unwrap());

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
    
    let _debug_res = hdk::debug("handle_receive_request(): send_response result: ");
    let _debug_res = hdk::debug(send_result.unwrap());        // Q: Receiving {Ok: {Ok: ___}} construction. How come? Wrapping?

    // Q: What now here?
    toss_entry
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
        call: 1 // !!! TODO: Randomize
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
    let _seed_confirmed = match seed_confirmed.unwrap() {
        true => {
            let _debug_res = hdk::debug("HCH/ receive_toss_response(): Seed valid!");
            true
        },
        _ => {
             let _debug_res = hdk::debug("HCH/ receive_toss_response(): Seed invalid!");
             false
        }
    };

    let _toss_confirmed = match toss_confirmed.unwrap() {
        1 => {
            let _debug_res = hdk::debug("HCH/ receive_toss_response(): Toss valid!");           
            true
        },
        _ => {
            let _debug_res = hdk::debug("HCH/ receive_toss_response(): Toss invalid!");
            false
        }
    };

    // Learning: Can I tell what I need to know to do the eval, from the top of my head? No? Then I don't have it modelled properly yet.
    evaluate_winner(toss_response);

    toss_result
}


// TODO: Find a better, more general name.
// Q: What would be the ideal variable to return, according to best practices?
// Initiator
fn evaluate_winner(toss_response: TossResponseMsg) -> bool {

    // Q: Reveal my seed here?
    let my_seed_addr = read_my_seed_hash().unwrap();
    let my_seed_result = hdk::get_entry(&my_seed_addr).unwrap();     // Q: Why need to do two unwraps? TODO: Error handling.
    let _my_seed_entry = my_seed_result.unwrap();
    
    // !!! Q: It seems that it allows to somehow fit different type to my SeedSchema?? :o Cause w/ App etc.
    let my_seed: SeedSchema = hdk::utils::get_as_type::<SeedSchema>(my_seed_addr).unwrap(); // my_seed_entry.content();      // Q: It was neccessary to use hdk::holochain_core_types::cas::content::AddressableContent; Why?

    // TODO: How would an idiomatic way to write this look like?
    // let my_seed: Entry::App = serde_json::from_str(&Content::from(&my_seed_entry).to_string()).unwrap();
    // let my_seed = my_seed_entry.// entry::GetEntryResultItem::new(my_seed_entry);
    // serde_json::from_str(&Content::from(&my_seed_entry).to_string()).unwrap(); // json!(Content::from(my_seed_entry)).into();
    
    let did_responder_win = check_call(my_seed.seed_value, toss_response.responder_seed.seed_value, toss_response.call);
    // let _debug_res = hdk::debug(RawString::from(Content::from(&my_seed_entry).to_string()));

    // TODO: Convert the entry to the SeedSchema struct. How?
    // Evaluation: Evaluating whether "call" and "initiator_seed_val + responder_seed_val % 2" have same parity (odd / even)

    // let _debug_res = hdk::debug("HCH/ evaluate_winner(): my_seed.seed_value");    
    let result_formatted = format!("HCH/ evaluate_winner(): initiator seedval: {}, responder seedval: {}, responder call: {}, responder won: {}",
        my_seed.seed_value,
        toss_response.responder_seed.seed_value,
        toss_response.call,
        did_responder_win);

    let _debug_res = hdk::debug(result_formatted);

    did_responder_win

    // OPTIM: How not to need to query for my seed again?
    // Persistence? Or do it in one function? Or?
}

// TODO: More fitting name
fn check_call(initiator_seed_value: u8, responder_seed_value: u8, responder_call: u8) -> bool {
    if (initiator_seed_value + responder_seed_value) % 2 == responder_call {
        true
    }
    else {
        false
    }
}

// TODO: reveal_seed() public function? To allow others to ask for the seed, once tossess commited?

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
    
    let _debug_res = hdk::debug("HCH/ read_my_seed_hash()");
    let _debug_res = hdk::debug(my_seed_addrs.clone());

    let addrs = my_seed_addrs.unwrap().clone();
    // TODO: Find out what exactly am I getting here.
    // TODO: Error handling.
    // TODO: In case of multiple plays, figure out to get the actual one.
    Ok(addrs[0].clone())
}