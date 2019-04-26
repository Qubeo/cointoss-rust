### Actual tasks


- Track the "receive callback fail" error

- Fix the N3H path problem




### Learnings 
Possibly:
- Async problems? :o
  
Tj. running next part of the test without waiting for the first part to end?
Check this.

Also, the "Private" entry started working in 0.0.11 I believe, so that might be the problem.
Might have been part of it.

Main problem, it seems, is that using AGENT_ADDRESS.to_string() (not sure what's the root of the problem) is breaking it down.
First I thought it's happening just to Bob, cause in the first send_request it works.
Then I thought it might be just in a callback function.
That might be se.
Now I'm going to test out the hypothesis, that it doesn't work in functions, that aren't Zome functions.