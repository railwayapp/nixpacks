---
title: Erlang
---

# {% $markdoc.frontmatter.title %}

Erlang is detected if there is a `rebar.config` file found, it assumes the project uses [rebar3](http://rebar3.org/).

## Install

_None_

## Build

The erlang provider will build the project with `rebar3 release`.

## Start

The release will be started with the foreground task and the binary named by the `release` field in the `relx` section in `rebar.config`.
