# Building from source

## Python

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
cd src/client/rust
maturin develop

# pip freeze > requirements.txt
```