---
title: Coherence
---

# {% $markdoc.frontmatter.title %}

[Coherence](https://www.withcoherence.com?utm_source=nixpacks) uses Nixpacks as the default build type for web applications, workers, scheduled tasks, and static sites. It uses the built container images for deploying Cloud IDE Workspaces, Cloud SSH Toolboxes, full-stack preview environments, as well as staging and production deployments to your own GCP or AWS cloud account. No confiuration is required to use Nixpacks on Coherence, simply omit a Dockerfile, and Nixpacks will be used. You can read more on the Coherence [docs](https://docs.withcoherence.com/docs/configuration/services#using-nixpacks). You can configure advanced functionality using Coherence variables for `NIXPACKS_*` settings as well as using a `nicpacks.toml` or `nixpacks.json` file in your repo.

![](/images/coherence.png)