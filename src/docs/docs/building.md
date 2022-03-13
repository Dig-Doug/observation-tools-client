# Building from source

## Python

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
maturin develop -m src/client/rust/Cargo.toml

# pip freeze > requirements.txt
```