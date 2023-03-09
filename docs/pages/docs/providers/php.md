---
title: PHP
---

# {% $markdoc.frontmatter.title %}

PHP is detected if a `composer.json` OR `index.php` file is found.

Note that Laravel apps need an `APP_KEY` environment variable in order to work.

If an `nginx.conf` or `nginx.template.conf` (see [this file](https://github.com/railwayapp/nixpacks/blob/main/src/providers/php/nginx.template.conf) for an example of template syntax) file is found in the project root directory, that configuration will be used.

If a `NIXPACKS_PHP_ROOT_DIR` variable is passed, that will be used as the server root.

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
