/**
 * This is not a production server yet!
 * This is only a minimal backend to get started.
 */

import * as express from 'express';

const app = express();

app.get('/api', (_req, res) => {
  res.send({ message: 'Welcome to express-app!' });
});

const port = process.env.port || 3333;
const server = app.listen(port, () => {
  console.log(`nx express app works`);
});
server.on('error', console.error);
