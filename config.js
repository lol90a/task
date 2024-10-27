// config.js
require('dotenv').config();

module.exports = {
  PORT: process.env.PORT || 3099,
  INFURA_PROJECT_ID: process.env.INFURA_PROJECT_ID,
  CLOUDINARY_NAME: process.env.CLOUDINARY_NAME,
  CLOUDINARY_API_KEY: process.env.CLOUDINARY_API_KEY,
  CLOUDINARY_API_SECRET: process.env.CLOUDINARY_API_SECRET,
};
