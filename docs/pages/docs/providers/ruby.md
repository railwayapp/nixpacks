---
title: Ruby
---

# {% $markdoc.frontmatter.title %}

Ruby is detected if a `Gemfile` file is found.

## Setup

The Ruby version is installed using [RVM](https://rvm.io/). You can specify the version in a `.ruby-version` file or the versions found in the `Gemfile` is installed. You can also

## Install

```
bundle install
```

If a `package.json` file is found then the dependencies are installed with the respective package manage from the [Node provider](/docs/providers/node) (NPM or Yarn).

## Build

_None_

## Start

If a [Rails](https://rubyonrails.org/) application.

```
bundle exec rails server -b 0.0.0.0
```

If a `config/environment.rb` file is found

```
bundle exec ruby script/server
```

If a `config.ru` file is found

```
bundle exec rackup config.ru
```

Otherwise

```
bundle exec rake
```
