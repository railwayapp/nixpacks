---
title: Python
---

# {% $markdoc.frontmatter.title %}

Python is detected if any of the following files are found

- `main.py`
- `requirements.txt`
- `pyproject.toml`
- `Pipfile`

A venv is created at `/opt/venv` and `PATH` is modified to use the venv python binary.

## Setup

The following Python versions are available

- `2.7`
- `3.8`
- `3.9`
- `3.10`
- `3.11` (Default)
- `3.12`
- `3.13`

The version can be overridden by

- Setting the `NIXPACKS_PYTHON_VERSION` environment variable
- Setting the version in a `.python-version` file
- Setting the version in a `runtime.txt` file
- Setting the version in a `.tool-versions` file

You also specify the exact poetry, pdm, and uv versions:

- The `NIXPACKS_POETRY_VERSION` environment variable or `poetry` in a `.tool-versions` file
- The `NIXPACKS_PDM_VERSION` environment variable
- The `NIXPACKS_UV_VERSION` environment variable or `uv` in a `.tool-versions` file

You can specify a particular package manager, to override the lockfile-based choice, by setting the
`NIXPACKS_PYTHON_PACKAGE_MANAGER` environment variable to one of the following:

- `auto` to choose based on the available lockfiles (default)
- `requirements` to install using `pip` from `requirements.txt`
- `setuptools` to install using `pip` with `build` and `setuptools`
- `poetry` to install using `poetry` from `poetry.lock`
- `pdm` to install using `pdm` from `pdm.lock`
- `uv` to install using `uv` from `uv.lock`
- `pipenv` to install with `pipenv` from `Pipfile` (if a `Pipfile.lock` is present it will be used)
- `skip` to not install a package

## Install

If `requirements.txt`

```shell
pip install -r requirements.txt
```

If `pyproject.toml`

```shell
pip install --upgrade build setuptools && pip install .
```

If `pyproject.toml` (w/ `poetry.lock`)

```shell
poetry install --no-dev --no-interactive --no-ansi
```

If `pyproject.toml` (w/ `pdm.lock`)

```shell
pdm install --prod
```

If `Pipfile` (w/ `Pipfile.lock`)

```shell
PIPENV_VENV_IN_PROJECT=1 pipenv install --deploy
```

If `Pipfile` (without `Pipfile.lock`)

```shell
PIPENV_VENV_IN_PROJECT=1 pipenv install --skip-lock
```

if `uv.lock`:

```shell
uv sync --no-dev --frozen
```

## Start

if Django Application

```shell
python manage.py migrate && gunicorn {app_name}.wsgi
```

if `pyproject.toml`

```shell
python -m {module}
```

Otherwise

```shell
python main.py
```

## Caching

These directories are cached between builds

- Install: `~/.cache/pip`
- Install: `~/.cache/uv`
- Install: `~/.cache/pdm`

## Environment Variables

The following environment variables are set by default:

```shell
PYTHONFAULTHANDLER=1
PYTHONUNBUFFERED=1
PYTHONHASHSEED=random
PYTHONDONTWRITEBYTECODE=1

PIP_NO_CACHE_DIR=1
PIP_DISABLE_PIP_VERSION_CHECK=1
PIP_DEFAULT_TIMEOUT=100
```

These can be overwritten by the `--env` option.
