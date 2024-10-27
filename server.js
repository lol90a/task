// server.js

const express = require("express");
const app = express();
const Web3 = require("web3");  

const connectDatabase = require("./config/database");
const cloudinary = require("cloudinary");
const PORT = process.env.PORT || 3099;

// Set up web3 connection using Infura
const web3 = new Web3("https://mainnet.infura.io/v3/c3f3a4d97a9340c8b8cc588ab1c87f37");

// Public DAI contract address and ABI (sample contract)
const daiContractAddress = "0x6B175474E89094C44Da98b954EedeAC495271d0F"; // DAI Stablecoin contract address
const daiABI = [
  // Simplified ABI 
  {
    constant: true,
    inputs: [],
    name: "name",
    outputs: [{ name: "", type: "string" }],
    type: "function",
  },
];

// UncaughtException Error
process.on("uncaughtException", (err) => {
  console.log(`Error: ${err.message}`);
  process.exit(1);
});

// connectDatabase();

cloudinary.config({
  cloud_name: process.env.CLOUDINARY_NAME,
  api_key: process.env.CLOUDINARY_API_KEY,
  api_secret: process.env.CLOUDINARY_API_SECRET,
});

// Define the new API route
app.get("/YourNameApiTest", async (req, res) => {
  try {
    const contract = new web3.eth.Contract(daiABI, daiContractAddress);
    
    // Fetch contract name as a demonstration
    const contractName = await contract.methods.name().call();
    console.log(`Contract Name: ${contractName}`);

    // Send response to API caller 
    res.json({ contractName });
  } catch (error) {
    console.error("Error fetching contract details:", error);
    res.status(500).send("Failed to fetch contract details");
  }
});

const server = app.listen(PORT, () => {
  console.log(`Server running`);
});

// Unhandled Promise Rejection
process.on("unhandledRejection", (err) => {
  console.log(`Error: ${err.message}`);
  server.close(() => {
    process.exit(1);
  });
});
