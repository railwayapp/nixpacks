const canvas = require('canvas');

const box = canvas.createCanvas(100, 100);

const fs = require('fs')
const out = fs.createWriteStream(__dirname + '/test.png')
const stream = box.createPNGStream()
stream.pipe(out)
out.on('finish', () =>  console.log('The PNG file was created.'))

console.log("Hello from Node canvas")