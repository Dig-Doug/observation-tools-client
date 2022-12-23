# Building from source

## Client libraries

### Python

Environment setup:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

Building:

```
maturin develop -m src/client/rust/Cargo.toml
```

Updating dependency list:

```
pip freeze > requirements.txt
```