const { copyFileSync, mkdirSync } = require('node:fs');

mkdirSync(__dirname + '/../../dist/apps/node-app', {
  recursive: true,
});

copyFileSync(
  __dirname + '/src/index.js',
  __dirname + '/../../dist/apps/node-app/index.js'
);
