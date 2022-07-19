---
title: Staticfile
---

# {% $markdoc.frontmatter.title %}

The Staticfile provider allows you to serve a single directory via [NGINX](https://www.nginx.com/).

Staticfile is detected if

- a `Staticfile` file is found at the app root
- `./public` directory exists
- `./index` directory exists
- `./dist` directory exists
- `./index.html` file exists

if this provider is matched for one of these reasons, then that directory/file will be served.

## Setup

NGINX is installed.

## Install

_None_

## Build

The NGINX config is copied to the correct location.

## Start

NGINX is started and the directory/file is served.
