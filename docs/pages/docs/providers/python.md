---
title: Python
---

# {% $markdoc.frontmatter.title %}

Python is detected if a `main.py` OR `requirements.txt` OR `pyproject.toml` file is found.

## Setup

The following Python versions are available

- `3.11`
- `3.10`
- `3.9`
- `3.8` (Default)
- `3.7`
- `2.7`

The version can be overridden by

- Setting the `NIXPACKS_PYTHON_VERSION` environment variable
- Setting the version in a `.python-version` file

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
