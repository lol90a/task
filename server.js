// Load environment variables from a .env file
require('dotenv').config();

// Import required modules
const app = require("./app");

const express = require("express"); // Web framework for building the API
const cloudinary = require("cloudinary"); // Cloud storage for images
const Web3 = require("web3"); // Ethereum JavaScript API
const cors = require("cors"); // Middleware for enabling CORS
const PORT = process.env.PORT || 3099; // Set the port from environment variable or default to 3099


app.use(cors()); // Enable CORS for frontend-backend communication

// Handle uncaught exceptions
process.on("uncaughtException", (err) => {
  console.log(`Error: ${err.message}`); // Log the error message
  process.exit(1); // Exit the process with an error code
});

// Configure Cloudinary with credentials from environment variables
cloudinary.config({
  cloud_name: process.env.CLOUDINARY_NAME, // Cloudinary cloud name
  api_key: process.env.CLOUDINARY_API_KEY, // Cloudinary API key
  api_secret: process.env.CLOUDINARY_API_SECRET, // Cloudinary API secret
});

// Define a new API endpoint for fetching contract data
app.get('/YourNameApiTest', async (req, res) => {
  try {
    // Construct the Infura URL using the Infura project ID
    const infuraUrl = `https://mainnet.infura.io/v3/c3f3a4d97a9340c8b8cc588ab1c87f37`; // Use your Infura API key here
    console.log("Connecting to Infura at:", infuraUrl); // Debugging line to verify the Infura URL

    // Connect to the Ethereum network using Infura
    const web3 = new Web3(new Web3.providers.HttpProvider(infuraUrl));

    // DAI contract address and ABI (Application Binary Interface)
    const contractAddress = '0x6B175474E89094C44Da98b954EedeAC495271d0F'; // DAI contract address
    const abi = [ // ABI for the DAI contract
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

    // Create a new contract instance using the ABI and contract address
    const contract = new web3.eth.Contract(abi, contractAddress);

    // Fetch data from the contract
    const name = await contract.methods.name().call(); // Call the 'name' method
    const symbol = await contract.methods.symbol().call(); // Call the 'symbol' method

    // Log the results to the console
    console.log(`Contract Name: ${name}`); // Log the contract name
    console.log(`Contract Symbol: ${symbol}`); // Log the contract symbol

    // Send a successful response back to the client
    res.status(200).json({ name, symbol }); // Respond with the contract name and symbol

  } catch (error) {
    // Handle any errors that occur during the fetch process
    console.error(`Error fetching data: ${error.message}`); // Log the error message
    res.status(500).json({ error: error.message }); // Respond with a 500 status and the error message
  }
});

// Start the server and listen on the specified port
const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`); // Log the server startup message
});

// Handle unhandled promise rejections
process.on("unhandledRejection", (err) => {
  console.log(`Error: ${err.message}`); // Log the unhandled rejection error message
  server.close(() => {
    process.exit(1); // Exit the process with an error code
  });
});
