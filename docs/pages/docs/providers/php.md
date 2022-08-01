---
title: PHP
---

# {% $markdoc.frontmatter.title %}

Php is detected if a `composer.json` OR `index.php` file is found.

## Setup

The following PHP versions are available

- `7.4`
- `8.0`
- `8.1` (Default)

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

```
{nginx_start_serving_cmd}
```
