---
title: PHP
---

# {% $markdoc.frontmatter.title %}

Php is detected if a `composer.json` OR `index.php` file is found.

Note that Laravel apps need an `APP_KEY` environment variable in order to work.

## Setup

The following PHP versions are available

- `8.0`
- `8.1`
- `8.2` (Default)

The version is automatically detected by parsing your `composer.json` file.

## Install

If composer.json

```
composer install
```

If package.json

```
[yarn|pnpm|npm] install
```

## Build

if package.json

```
[yarn|pnpm|npm] [prod|build]
```

## Start

This provider runs a Perl script to correct permissions and manage the Nginx configuration, and then starts Nginx.
```
{nginx_start_serving_cmd}
```
