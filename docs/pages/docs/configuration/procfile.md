---
title: Procfiles
---

# {% $markdoc.frontmatter.title %}

The standard Procfile format is supported by Nixpacks. However, only a single process is supported. The command specified in the Procfile will override the provider start command.

```toml
web: npm run start
```

## Release process

If a release process is found, a new phase is added that will run this command. The release phase will run after the build.

```toml
web: npm run start

# Will be run after the build phase
release: npm run migrate:deploy
```
