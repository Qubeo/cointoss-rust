
use hdk::error::ZomeApiResult;
// use hdk::AGENT_ADDRESS;
use hdk::{AGENT_ADDRESS, AGENT_ID_STR, DNA_NAME, AGENT_INITIAL_HASH, CAPABILITY_REQ, DNA_ADDRESS, PUBLIC_TOKEN};
use hdk::holochain_core_types::{
    hash::HashString,
    entry::Entry,
    cas::content::Address, cas::content::AddressableContent,
    json::{ JsonString },
};

// Q: Do I need to do this?
use crate::toss::{ TossSchema, SeedSchema, TossResultSchema, TossOutcome, ResultAndRevealedSchema };
use crate::messaging::{ MsgType, TossResponseMsg, GeneralMsg, RequestMsg };
use crate::messaging::{ process_general_msg, process_request_msg, process_toss_response_msg };


// TODO: Fix the documentation format. Q: Use "///"?
/*
 * Initiates the game by doing the first seed commit and sending the request to the agent through gossip (?)
 * Step A_01
 *
 * @callingType { ZomeApiResult<HashString> }
 * @exposure { public }
 * @param { Address } { agent_to }
 * @param { u8 } { seed_value }
 * @return { ZomeApiResult<HashString> } 
 */
pub fn handle_request_toss(agent_to: Address, seed_value: u8) -> ZomeApiResult<HashString> {     // Q: Misleading name? Cause request over N2N?
        
    let _debug_res = hdk::debug("HCH/ #A_01 handle_request_toss()");    
    let salt = generate_salt();

    handle_prdel();
    
    let seed = SeedSchema {
        salt: salt, // TODO: randomize.
        seed_value: seed_value  // Q: Randomize or let user enter thru the UI?
     };

    let seed_addr = handle_commit_seed(seed);

    // Q: Sync chaining vs. async waiting? Fragility vs. composability?
    // Callback? Future?
    let _received = handle_send_request(agent_to, seed_addr.clone().unwrap());

    let _debug_res = hdk::debug(format!("HCH/ handle_request_toss(): received: {:?}", _received));
    
    // TODO: What to return here, ideally?
    seed_addr
}

// @ ----------------------------------------------------------------------------------------------
// @ Sends the request via P2P messaging
// @ STEP A_02
pub fn handle_send_request(agent_to: Address, seed_hash: HashString) -> ZomeApiResult<String> {
    
    // !!! TODO: It seems AGENT_ADDRESS.to_string() works here. Unlike at other places (Bob? Callback?). How come? Hash vs. Address field perhaps?
    let _debug_res = hdk::debug("HCH/ #A_02 handle_send_request():");
    let _debug_res = hdk::debug(format!("HCH/ handle_send_request(): AGENT_ADDRESS.to_string(): {:?}", AGENT_ADDRESS.to_string()));

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

/* pub fn process_received_message(payload: String) -> ZomeApiResult<String> {

     Ok(json!({
           "key": "value"
       }).to_string())
} */

pub fn process_received_message(payload: String) -> ZomeApiResult<String> {
        
    let msg: GeneralMsg = process_general_msg(payload);      
    let _debug_res = hdk::debug(format!("HCH/ #B_01 received_message(): msg: {:?}", msg.clone()));

    let ag_addr = match "prdel".to_string() {
        _ => "Aaa".to_string()
    };

    let _debug_res = hdk::debug(format!("HCH/ #B_01 received_message(): msg: {:?}", ag_addr));

    // Parse the message type and choose the appropriate response--------------------------
    let result = match msg.message_type {
        // Toss request received -------------------------------------------------------
        MsgType::RequestToss => {
            let request_msg = process_request_msg(msg.message);     
            let _debug_res = hdk::debug(format!("HCH/ received_message(): RequestToss: request_msg: {:?}", request_msg.clone()));
                
            // !!! TODO: Here prolly a string escaping error? Breaking change in 0.0.11?
            let receive_request_result = handle_receive_request(request_msg).unwrap(); //.expect("Error receiving request."); // TODO: Fix the error handling. What exactly should this return? Should the unwrap be there?
            let _debug_res = hdk::debug(format!("HCH/ received_message(): RequestToss: receive_request_result: {:?}", receive_request_result.clone()));
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
    };

    let _debug_res = hdk::debug(format!("HCH/ received_message(): result: {:?}", result.clone()));

    result
}


pub fn handle_receive_request(request: RequestMsg) -> ZomeApiResult<Address> {

    // Commit seed
    //let _debug_res = hdk::debug(format!("HCH/ handle_receive_request(): request: {:?}", request.clone()));

    let my_seed = generate_seed("saltpr".to_string());    
    let _debug_res = hdk::debug(format!("HCH/ handle_receive_request(): my_seed: {:?}", my_seed.clone()));

    let my_seed_hash = handle_commit_seed(my_seed.clone()).unwrap();        // Q: Better use HashString or Address? (Idiomatic Holochain :) )
    
    let _debug_res = hdk::debug(format!("HCH/ handle_receive_request(): seed_value: {}", &my_seed.seed_value));

/*    let toss = TossSchema {
        initiator: request.agent_from.clone(),
        initiator_seed_hash: request.seed_hash.clone(),
        responder: Address::from(AGENT_ADDRESS.to_string()), // Q: Why can't just use the AGENT_ADDRESS?
        responder_seed_hash: my_seed_hash,
        call: (generate_pseudo_random() % 2) as u8     // TODO: Randomize
    }; */
    
    let toss = TossSchema {
        initiator: "HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(),
        initiator_seed_hash: "HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(),
        responder: "HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(), // Q: Why can't just use the AGENT_ADDRESS?
        responder_seed_hash: "HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(),
        call: 1     // TODO: Randomize
    };

    let _debug_res = hdk::debug(format!("HCH/ handle_receive_request(): toss: {:?}", toss.clone()));

    // Commit toss
    let toss_entry = handle_commit_toss(toss.clone());

    let _debug_res = hdk::debug(format!("handle_receive_request(): toss_entry: {}", toss_entry.clone().unwrap()));

    // Send call / response triplet - responder_seed, toss_hash, call
    // Q: Decomposition. Should be called from here or from some "central" function?
    // Q: Am I, as B, already revealing the seed here? Should I?
    /* let response_msg = TossResponseMsg {
        agent_from: //AGENT_ADDRESS.to_string().into(),
        responder_seed: my_seed.clone(),                                
        toss_hash: toss_entry.clone().unwrap(),
        call: 1     // TODO: Randomize or let the call be entered otherwise.
    }; */

    let response_msg = TossResponseMsg {
        agent_from: "HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(),//AGENT_ADDRESS.to_string().into(),
        responder_seed: my_seed.clone(), //"HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(),                                
        toss_hash: "HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(),
        call: 1     // TODO: Randomize or let the call be entered otherwise.
    };

    let send_result = send_response(toss.initiator.clone(), response_msg);
    
    let _debug_res = hdk::debug(format!("handle_receive_request(): send_response result: {:?}", send_result.unwrap()));
    // Q: Receiving {Ok: {Ok: ___ }} construction. How come? Wrapping?

    // Q: What now here?
    toss_entry
}



// TODO: Try making it interactive, pulling it from th UI instead? Or getting it somewhere else "outside"?
// Q: Should all this be public?
pub fn generate_seed(salt: String) -> SeedSchema {
    SeedSchema {
        salt: salt,
        seed_value: generate_random_seedval()
    }
}

pub fn reveal_seed(seed_addr: Address) -> ZomeApiResult<SeedSchema> {
    // Q: Some validation that it's okay to ask for seed revelation?
    hdk::utils::get_as_type::<SeedSchema>(seed_addr)
}

pub fn reveal_outcome(outcome_revealed_addr: Address) -> ZomeApiResult<ResultAndRevealedSchema> {
    hdk::utils::get_as_type::<ResultAndRevealedSchema>(outcome_revealed_addr)
}



pub fn generate_random_seedval() -> u8 {
    (generate_pseudo_random() % 9) as u8
}

// TODO: Placeholder. Also, possibly generates only even numbers.
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
   
   // !!! TODO: It seems the error is somewhere here (too). Either AGENT_ADDRESS or json!, I'd say.
   // But why / how?? In send_request the AGENT_ADDRESS doesn't do for any trouble? :o
   // !!! Okay, so AGENT_ADDRESS.to_string() here breaks it. Why? Why not in the "handle_send_request"?
   // "prdel".to_string() works on the other hand.
   // Perhaps only Bob? Or?
    let ag_addr: Address = "HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(); //AGENT_ADDRESS.to_string().into();
    let _dbg_res = hdk::debug(format!("HCH/ send_response: AGENT_ADDRESS.to_string(): {:?}", ag_addr));

    let wrapped_msg = GeneralMsg {
        agent_from: "HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into(), //AGENT_ADDRESS.to_string().into(),
        message_type: MsgType::TossResponse,
        message: json!(response_msg).to_string()
    };

    let _res_dbg = hdk::debug(format!("HCH/ send_response(): wrapped_msg: {:?}", wrapped_msg.clone()));

    let response = hdk::send(agent_to, json!(wrapped_msg).to_string(), 20000.into());
    let _debug_res = hdk::debug(format!("HCH/ send_response(): response: {:?}", response.clone()));

    response
}

pub fn handle_get_toss_history() -> ZomeApiResult<JsonString> {              
    Ok(json!("[not implemented yet]".to_string()).into())
}

pub fn handle_commit_seed(seed: SeedSchema) -> ZomeApiResult<Address> {
    // TODO: Validate if 9 <= seed >= 0?

    let seed_entry = Entry::App("seed".into(), seed.into());
    let seed_address = hdk::commit_entry(&seed_entry);

    let _res_dbg = hdk::debug(format!("HCH/ handle_commit_seed(): seed_address: {:?}", seed_address.clone()));

    // Q: Link it to an anchor? Or?

    // Q: Naming conventions: seed_address or seed_hash?
    // Q: Borrowing and unwrapping - what's the operator priorities?
    // let link_res = hdk::link_entries(&AGENT_ADDRESS, &seed_address.clone().unwrap(), "seeds");

    // let _res_dbg = hdk::debug(format!("HCH/ handle_commit_seed(): link_res: {:?}", link_res.clone()));

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

// TODO: Again, perhaps implement a version for general types <T>? Unify result.
pub fn confirm_seed(seed: SeedSchema, seed_hash: HashString) -> ZomeApiResult<bool> {

    let seed_hash_generated = get_seed_hash(seed).unwrap();
    let _debug_res = hdk::debug(format!("confirm_seed(): {}, {}", seed_hash.clone(), seed_hash_generated.clone()));
    // TODO: Error handling.
    Ok(seed_hash_generated == seed_hash)
}

// TODO: Won't confirm now, because I'm not storing my seed yet.
// TODO: JsonString doesn't make much sense here. Also, should be public?
pub fn handle_confirm_toss(toss: TossSchema, toss_hash: HashString) -> ZomeApiResult<u32> {
    
    let toss_hash_generated = get_toss_hash(toss).unwrap();
    
    // TODO: Unify error handling. (ZomeApiResult doesn't implement bool it seems.)
    Ok((toss_hash_generated == toss_hash) as u32)
}

pub fn handle_commit_toss(toss: TossSchema) -> ZomeApiResult<Address> {

    // TODO: Validate it the toss has the right format etc.?
    // Consider tying it with the validation logic somehow?

    let _res_dbg = hdk::debug(format!("HCH/ handle_commit_toss(): toss: {:?}", toss.clone()));

    let toss_entry = Entry::App("toss".into(), toss.into());
    let toss_address_result = hdk::commit_entry(&toss_entry);

    let _res_dbg = hdk::debug(format!("HCH/ handle_commit_toss(): toss_address_result: {:?}", toss_address_result.clone()));
    toss_address_result

    // TODO: Add to toss history.
    // Ok(address) => match hdk::link_entries(&AGENT_ADDRESS, &address, "tosses") {
}

// TODO: Update the name to reflect the function. Does it really handle just the receiving?
// @ Returns: ??? Toss entry address?? Or?? Custom error?

/*
pub fn handle_receive_request(request: RequestMsg) -> ZomeApiResult<Address> {
    Ok("HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string().into())
}
*/

pub fn handle_prdel() -> ZomeApiResult<String> {
    let res_dbg = hdk::debug(format!("HCH/ handle_prdel(): AGENT_ADDRESS.to_string(), {:?}", AGENT_ADDRESS.to_string()));
    Ok("prdel".to_string())
}

// TODO: Write down the lesson learnt from this bug explicitly into learning.md.

fn receive_toss_response(toss_response: TossResponseMsg) -> ZomeApiResult<Address> {
    
    // TODO: Read my seed hash from my chain. Or?
    // TODO: Unify nomenclature reference point - my vs. initiator.
    let my_seed_hash = read_my_seed_hash().unwrap();
    let responder_seed_hash = get_seed_hash(toss_response.responder_seed.clone()).unwrap();

    // !!! TODO: Possibly the AGENT_ADDRESS error again? Check. Is it the escaping? RawString? Why in callbacks tho?
    // Cause if here, then not just Bob.
    // Could it be it's not ZOME functions? So perhaps they don't have access? Or?
    let toss = TossSchema {
        initiator: Address::from("HcScjwO9ji9633ZYxa6IYubHJHW6ctfoufv5eq4F7ZOxay8wR76FP4xeG9pY3ui".to_string()), //AGENT_ADDRESS.to_string()),
        initiator_seed_hash: my_seed_hash,
        responder: toss_response.agent_from.clone(),
        responder_seed_hash: responder_seed_hash.clone(),
        call: 1 // !!! TODO: Randomize
    };

    let toss_entry_result = handle_commit_toss(toss.clone());
    let toss_addr = toss_entry_result.clone().unwrap();
    
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

    // Q: Where does this belong, in terms of best practices?

    let (outcome, my_seed) = evaluate_winner_and_reveal(toss_response);

    // TODO: Do both commit the toss result?
    // commit_toss_result();

    let toss_result: TossResultSchema = TossResultSchema {
        toss,
        outcome,
        time_stamp: "[to be implemented]".to_string()
    };

    // TODO: This is probably redundant and doesn't respect concern separation that much.
    // Figure out how to pass the revealed seed back to the UI.
    let outcome_and_revealed = ResultAndRevealedSchema {
        toss_result: toss_result.clone(),
        initiator_seed: my_seed        
    };

    // Add the result to history
    // Q: Does only one player, the initiator store it? Or?
    let toss_result_entry = Entry::App("toss_result".into(), toss_result.into());
    let toss_result_address = hdk::commit_entry(&toss_result_entry);
    let _link_res = hdk::link_entries(&AGENT_ADDRESS, &toss_result_address.clone().unwrap(), "toss_results", "");

    // Q: Isn't "None" a mistake? Shouldn't be an empty string?
    let get_result = hdk::get_links_and_load(&AGENT_ADDRESS, Some("toss_results".to_string()), None).unwrap();
    let _debug_res = hdk::debug(format!("HCH/ receive_toss_response(): get linking result: {:?}", get_result));

    // Ok(outcome_and_revealed)
    toss_result_address

}

// !!! TODO: Ta adresa jako parametr je teď nerelevantní, ne?
pub fn handle_reveal_toss_result(toss_result_addr: Address) -> ZomeApiResult<TossResultSchema> {
    let toss_result_str = "toss_result";
    let _debug_res = hdk::debug(format!("HCH/ reveal_toss_result(): toss_result_addr: {}", toss_result_addr.clone()));
    // let get_result = hdk::utils::get_as_type::<TossResultSchema>(toss_result_addr.clone());

    // Q: Why need to specify the S: String type parameter? How is the "toss_result" entry type relevant - why not needed here?
    // let get_result = hdk::get_links_and_load(&AGENT_ADDRESS, "toss_results").unwrap();

    // ERR: creates a temporary which is freed while still in use
    // let get_result = hdk::get_links(&AGENT_ADDRESS, "toss_results")?.addresses();
    let get_result = hdk::get_links(&AGENT_ADDRESS, Some("toss_results".to_string()), None); //.addresses();

    // let get_result = hdk::utils::get_links_and_load_type::<String, TossResultSchema>(&AGENT_ADDRESS.to_string().into(), "toss_results".to_string()).unwrap();
    let _debug_res = hdk::debug(format!("HCH/ reveal_toss_result(): {:?}", get_result)); //.clone()));

    // Q: Is the index 0 the right index (cause stack) or not (cause queue)?
    // Q: What exactly am I cloning / should I be cloning / should be cloning at all, vs. borrow / reference?
    // let element_addr = get_result[0].clone(); //.unwrap().address();
    // let got_element = hdk::utils::get_as_type::<TossResultSchema>(element_addr);

    let tmp_result_schema = get_dummy_toss_result();

    Ok(tmp_result_schema)
    //Ok(AGENT_ADDRESS.into()) // got_element.unwrap())
}

pub fn get_dummy_toss_result() -> TossResultSchema {
    let toss_result_addr: Address = AGENT_ADDRESS.to_string().into();
    TossResultSchema {
        outcome: TossOutcome::InitiatorWon,
        time_stamp: "prdel".to_string(),
        toss: TossSchema {
            call: 1,
            initiator: toss_result_addr.clone(),
            initiator_seed_hash: toss_result_addr.clone(),
            responder: toss_result_addr.clone(),
            responder_seed_hash: toss_result_addr        
        }
    }
}

// TODO: Somehow distinguish which functions can be called by what "role"?
// I.e. when I'm an initiator, I'm inhabiting the "initiator" role, hence such capabilities?

// TODO: Find a better, more general name.
// Q: What would be the ideal variable to return, according to best practices?
// Initiator
fn evaluate_winner_and_reveal(toss_response: TossResponseMsg) -> (TossOutcome, SeedSchema) {

    // Q: Reveal my seed here?
    let my_seed_addr = read_my_seed_hash().unwrap();    
    let my_seed = reveal_seed(read_my_seed_hash().unwrap()).unwrap();
    
    // let my_seed_entry = hdk::get_entry(&my_seed_addr).unwrap();   // Q: Why need to do two unwraps? TODO: Error handling.
    // Q: How not to need to query for my seed again - is that even possible? Persistence? Or do it in one function? Or?
    
    // TODO: Refactor. 
    let did_initiator_win = compute_outcome_for_initiator(my_seed.seed_value, toss_response.responder_seed.seed_value, toss_response.call);
 
    let result_formatted = format!("HCH/ evaluate_winner_and_reveal(): initiator seedval: {}, responder seedval: {}, responder call: {}, responder won: {:?}",
        my_seed.seed_value,
        toss_response.responder_seed.seed_value,
        toss_response.call,
        did_initiator_win);

    let _debug_res = hdk::debug(result_formatted);

    (did_initiator_win, my_seed)
}



// TODO: Possibility of automatic multiple tossing if draw?

// TODO: More fitting name
fn compute_outcome_for_initiator(initiator_seed_value: u8, responder_seed_value: u8, responder_call: u8) -> TossOutcome {
    let flipped_call = (initiator_seed_value + responder_seed_value) % 2;
    
    if flipped_call == responder_call {
        TossOutcome::InitiatorLost
    }
    else {
        TossOutcome::InitiatorWon
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
    
    let _debug_res = hdk::debug(format!("HCH/ read_my_seed_hash(): {:?}", my_seed_addrs.clone()));

    let addrs = my_seed_addrs.unwrap().clone();
    // TODO: Find out what exactly am I getting here.
    // TODO: Error handling.
    // TODO: In case of multiple plays, figure out to get the actual one.
    Ok(addrs[0].clone())
}