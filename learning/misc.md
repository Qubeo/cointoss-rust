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
Links: The link definitions in the zome function vs. LinkEntries? The zome: a validation blueprint, template of what's possible?
LinkEntries just carrying it out then? Simila to commit?

Validation: To what extent do I need to do explicit validation vs. to what extent can I rely on the "subsconscious" validations?

Questions of persistence: persisting e.g. my seed structure? So I can then hash it to the toss, when receiving the response from B?
    Role of links?

Can I get the agent address from the Conductor in JS? How? Conductor functions in the HDK?

In N2N messaging, how do I, as an agent, know where does the message come from??
How to decide which logic goes where? (N2N vs. zome calls etc.))

When I break it do

How to properly break the code down - Idiomatic design choices? Do I break Messaging away? Toss and Seed?


**Issues**
Receive callback doesn't receive the sender address. What good are anonymous messages?
Call of the "send_message" expects the send_message to return String. Why not ZomeApiResult<String>?
    Issue#746
holochain-nodejs expets JsonS

Error: "const result_seedhash = container.callRaw("prdelA::./dist/bundle.json", "cointoss", "main", "send_message", JSON.stringify(init_message));
                                    ^unable to call zome function: InternalFailure(RibosomeFailed("Trap: Trap { kind: Unreachable }"))"
    Somethig killing my zome? In the process_received_message()?

TODO: Propose a "formatted" hdk::debug! macro PR?

Non-deterministic error: "Can't get write lock on storage" or sth like that?

Networking:
"About to serve path "./ui2" at http://127.0.0.1:3002
Server started for "ui-interface"
thread '<unnamed>' panicked at 'error binding to 127.0.0.1:3002: error creating server listener: Normálně je povoleno pouze jedno použití každé adresy (protokolu, síťové adresy, portu) soketu. (os error 10048)', C:\Users\travis\.cargo\registry\src\github.com-1ecc6299db9ec823\hyper-0.12.25\src\server\mod.rs:115:17
note: Run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
2019-03-22 17:04:52:conductor: err/conductor: Error running interface 'websocket_interface': Io Error: Normálně je povoleno pouze jedno použití každé adresy (protokolu, síťové adresy, portu) soketu. (os error 10048)"
But then the N2N communication works. What's wrong?


**Misc learnings, notes**
Setting environment values in PSH: $env:MyTestVariable = "My temporary test variable."
Silence the noisy DHT debug logs: $env:HC_N3H_LOG_LEVEL='x' ('x' can be: 't', 'd', 'i', 'v', 'e')


// Q: Doesn't work. thread_rng as well.
// A: Prolly prohibited so as not to break determinism.
/* fn generate_random_from_nanos() -> u8 {
    let a = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % 9) as u8;
    
    let _debug_res = hdk::debug(format!("Random seed gen: {}", a));
    a
}*/



**Notes**
// TODO: Send notification, get the data from the UI.
// A: send_request
// B: generate_seed -> generate_toss -> commit_toss -> send_response
// A: handle_response -> commit_toss -> validate_seed -> validate_toss -> send_result
// B: receive_result -> validate_result -> announce_result


**Resources**


**Other bits**

    A source chain is an agent's personal ledger of 'things that happened' (whatever that means for the app they're using -- transactions, temperature readings, messages, etc). It's a hashchain (just like Git trees and blockchains) so any modification to old data breaks the chain and gets noticed. The source chain contains private entries and entries meant to be shared.
    The DHT in a Holochain app is a Kademlia DHT. (Yes, each app -- and each fork of each app -- has its own DHT.) The difference between it and your typical DHT is that nodes that receive the data also validate it against their own copy of the app's shared validation rules (called the DNA). The things that live in the DHT are:
        The DNA of the app
        Each agent's public key
        The headers of each chain entry from each agent's source chain (the header contains the typical prev/current hash like you'd see in any hash chain, plus the agent's signature and a local timestamp)
        The data for all public chain entries
        Links, a special type of chain entry that connects a known piece of data to an unknown piece of data, with a string tag. For instance, you could have a bunch of 'handle' links that link from the app DNA hash, which everybody has, to each member's public key, with the user's handle as the tag.
        Warrants, a piece of evidence against a bad actor, signed by the validating agent who discovered it

