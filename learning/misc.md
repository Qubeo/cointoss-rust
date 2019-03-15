**TODO**
Break the code down to files (messaging, etc.)
Implement links
    Describe how the link need emerged from the question: "How to persist my seed structure?"
Document everything dilligently

**Implementation ideas**
When agent B receives the toss request, he could get notified (how?) and prompted to play his move.
    If he doesn't, until a certain timeout, his hand gets played randomly and automatically.

Create a VSCode macro, copying any "//Q: " statements into this file as questions.
    Ideally even linking them and reflecting changes, ha :)

Variation ideas:
    Two-sided timed-out stake (alá HTLC (Hash Time Lock Time) / ILP (Inter Ledger Protocol) - Chris Chung mentioned, implemented by XRP)
    Weighed coin
    

**Questions**
Questions of persistence: persisting e.g. my seed structure? So I can then hash it to the toss, when receiving the response from B?
    Role of links?

Can I get the agent address from the Conductor in JS? How? Conductor functions in the HDK?

In N2N messaging, how do I, as an agent, know where does the message come from??
How to decide which logic goes where? (N2N vs. zome calls etc.))


**Issues**
Receive callback doesn't receive the sender address. What good are anonymous messages?
Call of the "send_message" expects the send_message to return String. Why not ZomeApiResult<String>?
    Issue#746
holochain-nodejs expets JsonS

Error: "const result_seedhash = container.callRaw("prdelA::./dist/bundle.json", "cointoss", "main", "send_message", JSON.stringify(init_message));
                                    ^unable to call zome function: InternalFailure(RibosomeFailed("Trap: Trap { kind: Unreachable }"))"
    Somethig killing my zome? In the process_received_message()?

TODO: Propose a "formatted" hdk::debug! macro PR?


**Misc learnings, notes**
Setting environment values in PSH: $env:MyTestVariable = "My temporary test variable."
Silence the noisy DHT debug logs: $env:HC_N3H_LOG_LEVEL='x' ('x' can be: 't', 'd', 'i', 'v', 'e')


**Notes**
// TODO: Send notification, get the data from the UI.
// A: send_request
// B: generate_seed -> generate_toss -> commit_toss -> send_response
// A: handle_response -> commit_toss -> validate_seed -> validate_toss -> send_result
// B: receive_result -> validate_result -> announce_result


**Resources**