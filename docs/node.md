# Node Support

For both NPM and Yarn providers, the following environment variables are set.

- `NODE_ENV=production`
- `NPM_CONFIG_PRODUCTION=false`: Ensure that dev deps are always installed

## [NPM](https://www.npmjs.com/)

**Install**:

```
npm install
```

**Build**

If build script found in `package.json`

```
npm run build
```

**Start**

Start script found in `package.json`

```
npm run start
```

If main field found in `package.json`

```
node {packageJson.main}
```

If `index.js` found

```
node index.js
```

## [Yarn](https://yarnpkg.com/)

Yarn is detected if a `yarn.lock` file is found at the root level.

_Currently only Yarn 1 is supported_

**Install**:

```
yarn install --frozen-lockfile
```

**Build**

If build script found in `package.json`

```
yarn build
```

**Start**

Start script found in `package.json`

```
yarn start
```

If main field found in `package.json`

```
node {packageJson.main}
```

If `index.js` found

```
node index.js
```
