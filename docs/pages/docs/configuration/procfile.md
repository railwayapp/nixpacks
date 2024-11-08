---
title: Procfiles
---

# {% $markdoc.frontmatter.title %}

The standard Procfile format is supported by Nixpacks. However, only a single process is supported. The command specified in the Procfile overrides the provider start command.

```toml
web: npm run start
```

If you have multiple entries in Procfile, here's how we choose which command:

- `release` is never picked
- `web` is picked
- `worker` is picked if `web` is not found
- If `web` and `worker` are not found, the first entry is picked sorted by the proc name alphabetically.

## Release process

If a release process is found, a new phase is added that will run this command. The release phase will run after the build.

```toml
web: npm run start

# Will be run after the build phase
release: npm run migrate:deploy
```
