---
title: PHP
---

# {% $markdoc.frontmatter.title %}

PHP is detected if a `composer.json` OR `index.php` file is found.

Note that Laravel apps need an `APP_KEY` environment variable in order to work.

If an `nginx.conf` or `nginx.template.conf` (see [this file](https://github.com/railwayapp/nixpacks/blob/main/src/providers/php/nginx.template.conf) for an example of template syntax) file is found in the project root directory, that configuration will be used.

By default, the server root is set at your project root, which is mounted in `/app`. If your application is supposed to have its server root in a subfolder, you must set it using `NIXPACKS_PHP_ROOT_DIR` variable (see below). For example, if you're using Symfony or Laravel, the server root directory is `public/`; you then need to set `NIXPACKS_PHP_ROOT_DIR=/app/public`.

To install PHP extensions, you can add them to the `require` section of your `composer.json`, following the [Composer platform dependencies specification](https://getcomposer.org/doc/articles/composer-platform-dependencies.md#different-types-of-platform-packages). The example below will add `ctype`, `iconv`, and `redis` extensions for PHP 8.2:

```json
{
  "require": {
    "php": ">=8.2",
    "ext-ctype": "*",
    "ext-iconv": "*",
    "ext-redis": "*"
  }
}
```

If a `NIXPACKS_PHP_ROOT_DIR` variable is passed, that will be used as the server root.
If a `NIXPACKS_PHP_FALLBACK_PATH` variable is passed, that will be used as a fallback for the server - for instance, if your app uses `index.php` as a router, you would set this variable to `/index.php`.

## Setup

The following PHP versions are available

- `8.1`
- `8.2` (Default)
- `8.3`
- `8.4`

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
