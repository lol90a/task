const express = require("express");
const Web3 = require("web3"); // Import Web3 for Ethereum interaction
const router = express.Router(); // Create a new router instance

// New API endpoint for fetching contract data
router.get('/YourNameApiTest', async (req, res) => {
  try {
    // Construct the Infura URL using the Infura project ID
    const infuraUrl = `https://mainnet.infura.io/v3/c3f3a4d97a9340c8b8cc588ab1c87f37`; // Use your Infura API key here
    console.log("Connecting to Infura at:", infuraUrl); // Debugging line

    // Connect to the Ethereum network (using Infura)
    const web3 = new Web3(new Web3.providers.HttpProvider(infuraUrl));

    // DAI contract address and ABI
    const contractAddress = '0x6B175474E89094C44Da98b954EedeAC495271d0F'; // DAI contract address
    const abi = [
      {
        "constant": true,
        "inputs": [],
        "name": "name",
        "outputs": [{"name": "", "type": "string"}],
        "payable": false,
        "stateMutability": "view",
        "type": "function"
      },
      {
        "constant": true,
        "inputs": [],
        "name": "symbol",
        "outputs": [{"name": "", "type": "string"}],
        "payable": false,
        "stateMutability": "view",
        "type": "function"
      }
    ];

    // Create a new contract instance
    const contract = new web3.eth.Contract(abi, contractAddress);

    // Fetch data from the contract
    const name = await contract.methods.name().call();
    const symbol = await contract.methods.symbol().call();

    // Log the results to the console
    console.log(`Contract Name: ${name}`);
    console.log(`Contract Symbol: ${symbol}`);

    // Send a response
    res.status(200).json({ name, symbol });

  } catch (error) {
    console.error(`Error fetching data: ${error.message}`);
    res.status(500).json({ error: error.message });
  }
});

// Export the router
module.exports = router;
