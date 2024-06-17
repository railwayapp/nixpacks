---
title: Python
---

# {% $markdoc.frontmatter.title %}

Python is detected if any of the following files are found

- `main.py`
- `requirements.txt`
- `pyproject.toml`
- `Pipfile`

## Setup

The following Python versions are available

- `2.7`
- `3.8`
- `3.9`
- `3.10`
- `3.11` (Default)
- `3.12`

The version can be overridden by

- Setting the `NIXPACKS_PYTHON_VERSION` environment variable
- Setting the version in a `.python-version` file
- Setting the version in a `runtime.txt` file

## Install

If `requirements.txt`

```
pip install -r requirements.txt
```

If `pyproject.toml`

```
pip install --upgrade build setuptools && pip install .
```

If `pyproject.toml` (w/ `poetry.lock`)

```
poetry install --no-dev --no-interactive --no-ansi
```

If `pyproject.toml` (w/ `pdm.lock`)

```
pdm install --prod
```

If `Pipfile` (w/ `Pipfile.lock`)

```
PIPENV_VENV_IN_PROJECT=1 pipenv install --deploy
```

## Start

if Django Application

```
python manage.py migrate && gunicorn {app_name}.wsgi
```

if `pyproject.toml`

```
python -m {module}
```

Otherwise

```
python main.py
```

## Caching

These directories are cached between builds

- Install: `~/.cache/pip`

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
