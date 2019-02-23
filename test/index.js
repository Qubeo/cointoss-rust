const { Config, Conductor, Scenario } = require('@holochain/holochain-nodejs');
Scenario.setTape(require('tape'));

const test = require('tape');
const tapSpec = require('tap-spec');

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


/*----------  Test globals  ----------*/

var g_address_alice, g_address_bob;
var g_alice, g_bob;
var handle_address_a, handle_address_b;
var g_seed_hash_a, g_seed_hash_b;
var g_toss_hash_a, g_toss_hash_b;
var g_received_toss;

// Q: How and where exactly are the "{alice, bob}" related to my configurations? Matched to the strings?
scenario.runTape('Can get address of both players', async (t, {alice, bob}) => {

  var result_alice = await alice.callSync('cointoss', 'get_my_address', {});
  var result_bob   = await bob.callSync('cointoss', 'get_my_address', {});    
  console.log("addr_result_Alice: " + result_alice.Ok);
  console.log("addr_result_Alice: " + result_bob.Ok);

  // Q: What if the returned value is not Ok, but Err? What then?
  // Test if the returned value has 92 bytes - lenght of the address
  // Q: Why?? 2x46, but why?
  //t.deepEqual(result_alice.Ok.length, 92);
  //t.deepEqual(result_bob.Ok.length, 92);

  g_address_alice = result_alice.Ok;
  g_address_bob = result_bob.Ok;
//});


//test('Call the set_handle() function, expect entry address as a result', async (test) => {
//scenario.runTape('Call the set_handle() function, expect entry address as a result', async (t, {alice, bob}) => {
console.log("//*************** 'Call the set_handle() function, expect entry address as a result'");

  const handle_alice = { handle: name_alice };
  const handle_bob   = { handle: name_bob };
  result_alice = await alice.callSync('cointoss', 'set_handle', handle_alice);
  result_bob = await bob.callSync('cointoss', 'set_handle', handle_bob);

  console.log("JS/ set_handle() result:" + result_alice.Ok);
  console.log("JS/ set_handle() result:" + result_bob.Ok);

  t.deepEqual(result_alice.Ok.length, 46);
  t.deepEqual(result_bob.Ok.length, 46);

  // g_address_alice = result_alice.Ok;
  // g_address_bob = result_bob.Ok;
  // test.end();

//});


// test('Initiate a toss by calling request_toss()', async (t) => {
// scenario.runTape('Initiate a toss by calling request_toss()', async (t, {alice}) => {
  console.log("//**************** 'Initiate a toss by calling request_toss()'");

  const request = { agent_to: g_address_bob, seed_value: 12 };
  const result_request = await alice.callSync('cointoss', 'request_toss', request);
  console.log("JS/ result_request:")
  console.log(result_request);

  t.deepEqual(result_request.Ok.length, 46);
  g_seed_hash_a = result_request.Ok;
  // t.end();
//});

// Q: Spinning up new instances for each scenario might break the addressess -> into one scenario?

console.log("//********************* 'Agent A/ Send the seed hash through N3H'");
//scenario.runTape('Agent A/ Send the seed hash through N3H', async (t, {alice, bob}) => {
// test('Agent A/ Send the seed hash through N3H', async (t) => {
  const send_message = { agent_to: g_address_bob, seed_hash: g_seed_hash_a};
  const result_seedhash_a = await alice.callSync('cointoss', 'send_request', send_message);

  console.log("JS/ result_seedhash B: ");
  console.log(result_seedhash_a);   // Q: .Ok doesn't work. Why? Need to deserialize into JSON here? Why?
  
  //t.deepEqual(result_seedhash.Ok.length, 46);
  g_seed_hash_b = result_seedhash_a.Ok;
  // t.end();
//});

// Receive the hashed entry. Create the TossRequest and commit it.
// Send the hash for that.
// Let B do the same.

// test('Agent B/ Commit toss', async (t) => {
// scenario.runTape('Agent B/ Commit toss', async (t, {alice, bob}) => {
console.log("//************************* Agent B Commit the toss");
  // Q: How and where should the orchestration happen? Now orchestrated by the "test" entity more or less. -> Distributed?
  // Q: Where should the toss construction logic happen? In HCH or in JS?
  // Q: Shouldn't it be included in the validation logic somehow? What might be the attack vectors here?
  
  const toss_struct = {
    initiator: g_address_alice,
    initiator_seed_hash: g_seed_hash_a,
    responder: g_address_bob,
    responder_seed_hash: g_seed_hash_b,
    call: 1
  };

  const result_toss = await bob.callSync('cointoss', 'commit_toss', { toss: toss_struct });

  console.log("JS/ result_toss: ");
  console.log(result_toss.Ok);
  
  //t.deepEqual(result_toss.Ok.length, 46);
  g_toss_hash_b = result_toss.Ok;

});

  /*

//test('Agent A/ Send the seed hash through N3H and receive the hash of the commited seed', (t) => {
console.log("//************************* Agent A commit the toss too");

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
*/
/*
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
   