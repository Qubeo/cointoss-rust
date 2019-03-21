//const { connect } = require('@holochain/hc-web-client');
import { connect } from '@holochain/hc-web-client';
const holochainUrl =  'ws://localhost:3401/'


    // connect().then(({callZome, close}) => {
    // callZome('instanceId', 'zome', 'funcName')(params)
    // })
    
    // window.holochainClient

function init() {
    console.log("init(): prdel");
connect(holochainUrl).then(({call, close}) => {
    document.getElementById('form').addEventListener('submit', e => {
    e.preventDefault()

        // First, get instance info...
    call('info/instances')().then(info => {
      const instance = getInstance(info, 'the_dna_hash', 'the_agent_hash')
      const zomeName = 'cointoss'
      const functionName = 'register'
      const params = {
        handle: "rdlel" // content: document.querySelector('#message').value,
        // in_reply_to: 'in reply to'
      }

      //const handle_alice = { handle: "prdelice" };
      //result_alice = await alice.call('cointoss', 'register', handle_alice);

      console.log(instance);
      const createPost = call(instance, zomeName, functionName)
      createPost(params)
    })
})

document.getElementById('info').addEventListener('click', e => {
    call('info/instances')().then(data => console.log(data))
  })

document.getElementById('close').addEventListener('click', e => {
    close()
})

})

}

function getInstance(info, dna, agent) {
  const entry = Object.entries(info)
    .find(([id, value]) => value.dna === dna && value.agent === agent)
  if (entry) {
    return entry[1].id
  } else {
    return null
  }
}
