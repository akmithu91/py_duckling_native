# py_duckling_native

Python bindings for Facebook's Duckling NLP library, via Rust FFI (Haskell → Rust → Python).

## Prerequisites

- Python 3.8+
- Rust toolchain (`rustup`)
- `uv` (https://docs.astral.sh/uv/)
- `rust_duckling_host` crate published to your registry (with `links = "ducklingffi"`)

## Project Structure

```
py_duckling_native/
├── Cargo.toml               # Rust crate config, depends on rust_duckling_host
├── build.rs                  # Reads DEP_DUCKLINGFFI_LIB_DIR, sets up linking
├── pyproject.toml            # Maturin build backend (invoked by uv)
├── python/
│   └── py_duckling_native/
│       └── __init__.py       # Preloads Haskell .so files, re-exports API
├── src/
│   └── lib.rs                # PyO3 wrapper around rust_duckling_host
├── build_wheel.sh            # Builds wheel + bundles .so files
└── test_duckling.py          # Python tests
```

## Build & Install

### Option A: Development mode (quick iteration)

```bash
# Build and install editable
uv pip install maturin
maturin develop

# Copy .so files next to the installed module:
SITE=$(python -c "import py_duckling_native; print(py_duckling_native.__file__)" | xargs dirname)
LIB_DIR=$(cat .duckling_lib_dir)
mkdir -p "$SITE/libs"
cp "$LIB_DIR"/*.so* "$SITE/libs/"

# Test
python test_duckling.py
```

### Option B: Build a distributable wheel (recommended)

```bash
./build_wheel.sh

# Install the wheel
uv pip install dist/py_duckling_native-*.whl --force-reinstall

# Test
python test_duckling.py
```

## Usage

```python
from py_duckling_native import DucklingParser, parse, supported_dimensions

# Create a parser with a timezone
parser = DucklingParser("America/New_York")

# Parse all dimensions
results = parser.parse("Meet me tomorrow at 3pm, it costs $50")
for r in results:
    print(f"{r['dim']}: {r['body']} (chars {r['start']}:{r['end']})")
    print(f"  value: {r['value']}")

# Parse specific dimensions only
results = parser.parse(
    "Lunch costs $15 and takes 45 minutes",
    dimensions=["AmountOfMoney", "Duration"]
)

# Convenience function (no class needed)
results = parse("Call (415) 555-1234", timezone="UTC")

# List supported dimensions
print(supported_dimensions())
```

## How It Works

1. `rust_duckling_host` (your existing Rust crate) wraps the Haskell Duckling
   library via C FFI and bundles all `.so` dependencies in `ext_lib/`.

2. `py_duckling_native` depends on `rust_duckling_host` as a Cargo dependency.
   Its `build.rs` reads `DEP_DUCKLINGFFI_LIB_DIR` to find the `.so` files.

3. `src/lib.rs` uses PyO3 to expose `DucklingParser` and `parse()` to Python.

4. `build_wheel.sh` builds the wheel with maturin, then injects all Haskell
   `.so` files into a `libs/` subdirectory inside the wheel.

5. `__init__.py` preloads all `.so` files from `libs/` using `ctypes.CDLL`
   with `RTLD_GLOBAL` before importing the native extension, ensuring all
   Haskell symbols are available at runtime.

## API Reference

### `DucklingParser(timezone: str = "UTC")`
Create a parser pinned to the given IANA timezone.

### `DucklingParser.parse(text: str, dimensions: list[str] | None = None) -> list[dict]`
Parse text and return extracted entities. Each entity is a dict:
- `dim`: dimension name (e.g. `"time"`, `"amount-of-money"`)
- `body`: matched text span
- `start`: start character offset
- `end`: end character offset
- `value`: structured value (dict)

### `parse(text, timezone="UTC", dimensions=None) -> list[dict]`
Convenience function that creates a temporary parser.

### `supported_dimensions() -> list[str]`
Returns list of all valid dimension names.
