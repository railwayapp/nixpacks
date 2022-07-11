const canvas = require('canvas');

const c = canvas.createCanvas(25, 25);

c.toBuffer("image/png");

console.log("Hello from Bun with canvas!");