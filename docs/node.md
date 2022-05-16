# Node Support

The NPM, Yarn and PNPM providers all have the following environment variables set:
- `NODE_ENV=production`
- `NPM_CONFIG_PRODUCTION=false`: Ensure that dev deps are always installed

## [NPM](https://www.npmjs.com/)

**Install**:
If a lockfile is found in the source code
```
npm ci
```
If a lockfile isn't found in the source code
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

**Install**:

For [Yarn 1](https://classic.yarnpkg.com/)
```
yarn install --frozen-lockfile
```

For [Yarn 2+](https://yarnpkg.com/)
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
yarn run start
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
pnpm run start
```

If main field found in `package.json`

```
node {packageJson.main}
```

If `index.js` found

```
node index.js
```
