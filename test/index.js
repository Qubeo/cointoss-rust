const { Config, Container, Scenario } = require('@holochain/holochain-nodejs');
Scenario.setTape(require('tape'));

const dnaPath = "dist/bundle.json";
const dna = Config.dna(dnaPath, 'happs');    // Q: What's this?

const name_alice = "alice";
const name_bob   = "bob";

const agentAlice = Config.agent(name_alice);
const agentBob   = Config.agent(name_bob);
const instanceAlice = Config.instance(agentAlice, dna);
const instanceBob   = Config.instance(agentBob, dna);

const scenario = new Scenario([instanceAlice, instanceBob]);

// test.createStream()
//  .pipe(tapSpec())
//  .pipe(process.stdout);

/*----------  Events  ----------*/

var g_address_alice;
var g_address_bob;
  
var handle_address_a;
var handle_address_b;

var g_seed_hash_a;
var g_seed_hash_b;
var g_received_toss;

// Q: How to get the agent address?


// Q: How and where exactly are the "{alice, bob}" related to my configurations? Matched to the strings?
scenario.runTape('Can get address of both players', async (t, {alice, bob}) => {

  const result_alice = await alice.callSync('cointoss', 'get_my_address', {});
  const result_bob   = await bob.callSync('cointoss', 'get_my_address', {});    
  console.log("addr_result_Alice: " + result_alice.Ok);
  console.log("addr_result_Alice: " + result_bob.Ok);

  // Q: What if the returned value is not Ok, but Err? What then?
  // Test if the returned value has 92 bytes - lenght of the address
  // Q: Why?? 2x46, but why?
  t.deepEqual(result_alice.Ok.length, 92);
  t.deepEqual(result_bob.Ok.length, 92);

  g_address_alice = result_alice.Ok;
  g_address_bob = result_bob.Ok;
});


scenario.runTape('Call the set_handle() function, expect entry address as a result', async (t, {alice, bob}) => {

  const handle_alice = { handle: name_alice };
  const handle_bob   = { handle: name_bob };
  const result_alice = await alice.callSync('cointoss', 'set_handle', handle_alice);
  const result_bob = await bob.callSync('cointoss', 'set_handle', handle_bob);

  console.log("JS/ set_handle() result:" + result_alice.Ok);
  console.log("JS/ set_handle() result:" + result_bob.Ok);

  t.deepEqual(result_alice.Ok.length, 46);
  t.deepEqual(result_bob.Ok.length, 46);

  // g_address_alice = result_alice.Ok;
  // g_address_bob = result_bob.Ok;

});

// ISSUE: I don't know why it works now. Just commented out a section?! Non-deterministic?
scenario.runTape('Initiate a toss by calling request_toss()', async (t, {alice}) => {

  const request = { agent_to: g_address_bob, seed: 12 };
  const result_request = await alice.callSync('cointoss', 'request_toss', request);
  console.log("JS/ result_request:")
  console.log(result_request);

  t.deepEqual(result_request.Ok.length, 46);
  g_seed_hash_a = result_request.Ok;
});


scenario.runTape('Agent A/ Send the seed hash through N3H', async (t, {alice, bob}) => {

  // let request_message_json = JSON.stringify("{message_type: RequestMsg, seed_hash: " + g_seed_hash_a.toString() + " }");    // ISSUE: This works to bypass the JSON.parse error in holochain-nodejs
  // let request_message = "{message_type: RequestMsg, seed_hash: " + g_seed_hash_a.toString() + " }";    // ISSUE: This works to bypass the JSON.parse error in holochain-nodejs
  const init_message = { agent_to: g_address_bob, seed_hash: g_seed_hash_a};

  // let request_message = { agent_from: g_address_alice, seed_hash: g_seed_hash_a.toString() };
  // request_message = JSON.stringify(request_message);                        // Q: Still not sure, whether needed.
  // const init_message = { agent_to: g_address_B, message: request_message };
  // const result_seedhash = container.callRaw("prdelA::./dist/bundle.json", "cointoss", "main", "send_message", JSON.stringify(init_message));
  // const result_seedhash = await alice.callSync('cointoss', 'send_message', JSON.stringify(init_message));

  const result_seedhash = await alice.callSync('cointoss', 'send_request', init_message);

  console.log("JS/ result_seedhash B (???): ");
  console.log(result_seedhash.Ok);

  // console.log("Stringified init_message: " + JSON.stringify(init_message));
   


});

/*
test('Agent A/ Send the seed hash through N3H', (t) => {

  // let msg_json = JSON.stringify("{toss_request: prdel}");    // ISSUE: This works to bypass the JSON.parse error in holochain-nodejs
  // const init_message = { to_agent: g_address_B, message: msg_json};

 // let request_message = "{ agent_from:" + g_address_A + ", seed_hash:" + g_seed_hash_a.toString() + " }";
  let request_message = { agent_from: + g_address_A, seed_hash: g_seed_hash_a.toString() };
  // request_message = JSON.stringify(request_message);                        // Q: Still not sure, whether needed.
  const init_message = { agent_to: g_address_B, message: request_message };

  console.log("Stringified init_message: " + JSON.stringify(init_message));
    
  // ISSUE: container.call automatically expects JsonString as a result, not taking into account send_message returns string?
  
  // Q: What am I doing here, in architectural terms? Shouldn't I be calling the messaging through the container and instances functions?
  // Q: Doesn't the send_message return bullshit, instead of receive() of B returning?
  const result_seedhash = container.callRaw("prdelA::./dist/bundle.json", "cointoss", "main", "send_message", JSON.stringify(init_message));
  // const result_seedhash = player_A.call("cointoss", "main", "send_message", init_message);
  // const result_seedhash = container.callRaw("prdelA::./dist/bundle.json", "cointoss", "main", "test_fn", JSON.stringify(init_message));

  console.log("JS/ send_message() result (hash of the commited seed): ");
  console.log(result_seedhash);
  t.end();
});

test('Agent A/ Commit a seed and return the entry address', (t) => {

  // Q: Where should the "salt" be generated? UI? App instance? How much freedom for the agent? Visibility?
  const seed_schema_a = { salt: "prdel", seed_value: 22 };
  const result_request = player_A.call("cointoss", "main", "commit_seed", { seed: seed_schema_a });

  g_seed_hash_a = result_request.Ok;

  console.log("JS/ commit_seed() result: ");
  console.log(g_seed_hash_a);

  t.end();
});

test('Agent B/ Receive the toss request and commit the toss', (t) => {

  //const result_receive = player_B.call("cointoss", "main", "receive_request", { agent_key: g_address_A, seed_hash: g_seed_hash_a });
  //g_received_toss = result_receive.Ok;

  //console.log("JS/ receive_request() result: ");
  //console.log(result_receive);

  t.end();
});

test('Agent A/ Receive the toss response, confirm the toss and commit it too', (t) => {

  const result_confirm = player_B.call("cointoss", "main", "confirm_toss", { toss: g_received_toss });

  console.log("JS/ receive_request() result: ");
  console.log(result_confirm);

  t.end();
});

test('Agent A/ reveals the result', (t) => {

  // console.log("JS/ ... ");
  t.end();
});


*/

// Misc learning bits:
// Decode from bs58 to hex, slice the leading 2 bytes, encode back to bs58
// var recoded_result = bs58.encode(Buffer.from(bs58.decode(result)).slice(2));
// var hashed_key1 = shajs('sha256').update(key1).digest();
// const b58_prdel = bs58.encode(Buffer.from(hashed_key1));  
   