# Python Support

Python is detected if a `main.py` OR `requirements.txt` OR `pyproject.toml` file is found.

**Install**:

if `requirements.txt`
```
pip install -r requirements.txt
```

if `pyproject.toml`
```
pip install --upgrade build setuptools && pip install .
```

if `pyproject.toml` (w/ `poetry.lock`)
```
poetry install --no-dev --no-interactive --no-ansi
```

**Build**

```
go build -o out
```

**Start**

if Django Application
```
python manage.py migrate && gunicorn {app_name}
```

if `pyproject.toml`

```
python -m {module}
```

Otherwise
```
python main.py
```
