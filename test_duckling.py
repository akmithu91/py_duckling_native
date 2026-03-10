#!/usr/bin/env python3
"""
Test suite for py_duckling_native.

Run with:
    python test_duckling.py
    # or
    python -m pytest test_duckling.py -v
"""

from py_duckling_native import DucklingParser, parse, supported_dimensions


def test_basic_parse():
    """Parse text with all dimensions."""
    parser = DucklingParser("America/New_York")
    results = parser.parse("Meet me tomorrow at 3pm, it costs $50")

    assert len(results) > 0, "Should extract at least one entity"

    dims_found = {r["dim"] for r in results}
    print(f"  Found dimensions: {dims_found}")

    assert "time" in dims_found, "Should find time dimension"
    assert "amount-of-money" in dims_found, "Should find money dimension"

    for r in results:
        print(f"  {r['dim']:>20s}: {r['body']!r}  [{r['start']}:{r['end']}]")
        assert r["start"] <= r["end"], "start must be <= end"
        assert isinstance(r["value"], dict), "value should be a dict"

    print("  PASSED\n")


def test_filtered_dimensions():
    """Parse with a dimension filter."""
    parser = DucklingParser("UTC")
    results = parser.parse(
        "I need $100 and 5 gallons by tomorrow",
        dimensions=["AmountOfMoney", "Volume"],
    )

    dims_found = {r["dim"] for r in results}
    print(f"  Found dimensions: {dims_found}")

    # Should only return the requested dimensions
    assert "amount-of-money" in dims_found, "Should find money"
    assert "volume" in dims_found, "Should find volume"
    assert "time" not in dims_found, "Should NOT find time (not requested)"

    for r in results:
        print(f"  {r['dim']:>20s}: {r['body']!r}")

    print("  PASSED\n")


def test_convenience_function():
    """Test the module-level parse() function."""
    results = parse("Call me at (415) 555-1234", timezone="UTC")

    assert len(results) > 0, "Should extract at least one entity"
    dims_found = {r["dim"] for r in results}
    assert "phone-number" in dims_found, "Should find phone number"

    for r in results:
        print(f"  {r['dim']:>20s}: {r['body']!r}")

    print("  PASSED\n")


def test_supported_dimensions():
    """Check that supported_dimensions() returns all 13 dimensions."""
    dims = supported_dimensions()
    print(f"  Supported: {dims}")
    assert len(dims) == 13, f"Expected 13 dimensions, got {len(dims)}"
    assert "Time" in dims
    assert "AmountOfMoney" in dims
    print("  PASSED\n")


def test_all_dimensions():
    """
    Parse a passage that contains all dimension types (mirrors the Rust test).
    """
    parser = DucklingParser("America/New_York")

    text = (
        "Tomorrow at 3pm, I'll drive 5 miles to the store "
        "and spend about 2 hours picking up 6 pounds of flour and 2 gallons of milk. "
        "The total will be $250, paid with card 4111111111111111. "
        "The weather is 72°F outside. For questions, call (415) 555-1234, "
        "email alice@example.com, or visit https://example.com. "
        "Oh and I need exactly 42 widgets."
    )

    results = parser.parse(text)
    dims_found = {r["dim"] for r in results}
    print(f"  Found dimensions: {sorted(dims_found)}")

    expected = {
        "amount-of-money",
        "credit-card-number",
        "distance",
        "duration",
        "email",
        "number",
        "phone-number",
        "quantity",
        "temperature",
        "time",
        "url",
        "volume",
    }

    missing = expected - dims_found
    assert not missing, f"Missing dimensions: {missing}"

    for r in sorted(results, key=lambda x: x["start"]):
        print(f"  {r['dim']:>20s}: {r['body']!r}  [{r['start']}:{r['end']}]")

    print("  PASSED\n")


def test_offsets_match_body():
    """Verify that start:end slicing the input matches body."""
    parser = DucklingParser("UTC")
    text = "I need $100 by tomorrow"
    results = parser.parse(text)

    for r in results:
        sliced = text[r["start"]:r["end"]]
        assert sliced == r["body"], (
            f"Offset mismatch: text[{r['start']}:{r['end']}] = {sliced!r} "
            f"but body = {r['body']!r}"
        )
        print(f"  OK: [{r['start']}:{r['end']}] -> {sliced!r}")

    print("  PASSED\n")


def test_empty_input():
    """Empty input should return an empty list."""
    parser = DucklingParser("UTC")
    results = parser.parse("")
    assert results == [], f"Expected empty list, got {results}"
    print("  PASSED\n")


def test_different_timezones():
    """Parsing the same time text with different timezones should work."""
    for tz in ["UTC", "America/New_York", "Europe/London", "Asia/Tokyo"]:
        parser = DucklingParser(tz)
        results = parser.parse("tomorrow at 3pm")
        assert len(results) > 0, f"Should find time entity for tz={tz}"
        print(f"  {tz:>20s}: {results[0]['value']}")

    print("  PASSED\n")


if __name__ == "__main__":
    tests = [
        ("Basic parse", test_basic_parse),
        ("Filtered dimensions", test_filtered_dimensions),
        ("Convenience function", test_convenience_function),
        ("Supported dimensions", test_supported_dimensions),
        ("All dimensions", test_all_dimensions),
        ("Offset matching", test_offsets_match_body),
        ("Empty input", test_empty_input),
        ("Different timezones", test_different_timezones),
    ]

    print("=" * 60)
    print("py_duckling_native test suite")
    print("=" * 60)
    print()

    passed = 0
    failed = 0

    for name, fn in tests:
        print(f"[TEST] {name}")
        try:
            fn()
            passed += 1
        except Exception as e:
            print(f"  FAILED: {e}\n")
            failed += 1

    print("=" * 60)
    print(f"Results: {passed} passed, {failed} failed, {passed + failed} total")
    print("=" * 60)

    if failed > 0:
        exit(1)
