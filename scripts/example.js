const { JsonRpc } = require("node-jsonrpc-client");

const client = new JsonRpc("http://localhost:8989");

client.call("hello_method", {})
  .then((result) => {
    console.log("output", result);
  })
  .catch((err) => {
    console.error("Oopsies! Error code " + err.code + ": " + err.message);
  });
