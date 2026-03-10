"""
py_duckling_native - Python bindings for Facebook's Duckling NLP library.

Usage:
    from py_duckling_native import DucklingParser

    parser = DucklingParser("America/New_York")
    results = parser.parse("Meet me tomorrow at 3pm, it costs $50")
    for r in results:
        print(f"  {r['dim']}: {r['body']} ({r['start']}:{r['end']})")

    # Or filter by dimension:
    results = parser.parse("dinner at 8pm", dimensions=["Time"])

    # Or use the convenience function:
    from py_duckling_native import parse
    results = parse("Send $100 tomorrow", timezone="UTC")
"""

import ctypes
import os
import sys
from pathlib import Path

def _preload_libs():
    """
    Preload all bundled Haskell/C shared libraries so the native
    extension can find them at import time.

    This is necessary because the Haskell RTS and Duckling's many
    transitive .so dependencies must be loaded with RTLD_GLOBAL
    before the Python extension module is loaded.
    """
    libs_dir = Path(__file__).parent / "libs"
    if not libs_dir.exists():
        return

    # Load libffi and libgmp first (low-level deps), then Haskell RTS,
    # then everything else. Order matters for symbol resolution.
    so_files = sorted(libs_dir.glob("*.so*"))

    # Prioritize loading order: system deps → RTS → everything else
    priority_prefixes = ["libffi", "libgmp", "libpcre", "libHSrts"]
    early = []
    late = []
    for f in so_files:
        name = f.name
        if any(name.startswith(p) for p in priority_prefixes):
            early.append(f)
        else:
            late.append(f)

    for lib_path in early + late:
        try:
            ctypes.CDLL(str(lib_path), mode=ctypes.RTLD_GLOBAL)
        except OSError:
            # Some versioned symlinks may fail; that's OK as long as
            # the real file loaded successfully.
            pass


# Preload before importing the native extension
_preload_libs()

# Import from the native Rust/PyO3 module
from py_duckling_native._native import DucklingParser, parse, supported_dimensions

__all__ = ["DucklingParser", "parse", "supported_dimensions"]
