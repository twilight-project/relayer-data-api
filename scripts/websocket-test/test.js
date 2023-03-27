const WebSocket = require('websocket').w3cwebsocket;

let ws = new WebSocket("ws://localhost:8990");

var subscriptions = {};

ws.addEventListener('open', () => {
	ws.send(JSON.stringify({
		jsonrpc: "2.0",
		id: "price_feeder",
		method: "subscribe_live_price_data",
		params: null,
	}));
});

process.on('SIGINT', function () {
    console.log("Cancelling subscriptions");
    for ( const [key, id] of Object.entries(subscriptions)) {
        console.log(`Unsubscribe ${key} with id ${id}`);

        const method = `unsubscribe_${key}`;

        ws.send(JSON.stringify({
            jsonrpc: "2.0",
            id: method,
            method: method,
            params: { id: id },
        }));
    }
    process.exit();
});


ws.addEventListener('message', (message) => {
  const obj = JSON.parse(message.data);

  if (obj.id !== undefined) {
      console.log('Subscription id: ', obj.result);
      subscriptions[obj.id] = obj.result;
  } else {
      if (obj.method == "subscribe_live_price_data") {
          console.log(`Got live_data method: ${JSON.stringify(obj.params)}`);
      } else {
	      console.log(`Other method: ${JSON.stringify(obj.params)}`);
      }
  }
});

console.log('Starting');
