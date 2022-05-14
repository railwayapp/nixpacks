# Node Support

For all three NPM, Yarn and PNPM providers, the following environment variables are set.

- `NODE_ENV=production`
- `NPM_CONFIG_PRODUCTION=false`: Ensure that dev deps are always installed

## [NPM](https://www.npmjs.com/)

**Install**:
If lockfile is found in source code
```
npm ci
```
If lockfile isn't found in source code
```
npm i
```

**Build**

If build script found in `package.json`

```
npm run build
```

**Start**

Start script found in `package.json`

```
npm start
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

**Install**:

For Yarn 1
```
yarn install --frozen-lockfile
```

For Yarn 2+
```
yarn install --immutable --check-cache
```
**Build**

If build script found in `package.json`

```
yarn run build
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

## [PNPM](https://pnpm.io/)

PNPM is detected if a `pnpm-lock.yaml` file is found at the root level.

**Install**:

```
pnpm i --frozen-lockfile
```
**Build**

If build script found in `package.json`

```
pnpm run build
```

**Start**

Start script found in `package.json`

```
pnpm start
```

If main field found in `package.json`

```
node {packageJson.main}
```

If `index.js` found

```
node index.js
```
