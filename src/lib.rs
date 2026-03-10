use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::Py;
use duckling_rust::{Dimension, Duckling};

/// Maps a Python dimension string to the Rust enum.
fn parse_dimension(dim: &str) -> PyResult<Dimension> {
    match dim {
        "AmountOfMoney" | "amount-of-money" | "amount_of_money" => Ok(Dimension::AmountOfMoney),
        "CreditCardNumber" | "credit-card-number" | "credit_card_number" => {
            Ok(Dimension::CreditCardNumber)
        }
        "Distance" | "distance" => Ok(Dimension::Distance),
        "Duration" | "duration" => Ok(Dimension::Duration),
        "Email" | "email" => Ok(Dimension::Email),
        "Numeral" | "numeral" | "number" => Ok(Dimension::Numeral),
        "Ordinal" | "ordinal" => Ok(Dimension::Ordinal),
        "PhoneNumber" | "phone-number" | "phone_number" => Ok(Dimension::PhoneNumber),
        "Quantity" | "quantity" => Ok(Dimension::Quantity),
        "Temperature" | "temperature" => Ok(Dimension::Temperature),
        "Time" | "time" => Ok(Dimension::Time),
        "Url" | "url" => Ok(Dimension::Url),
        "Volume" | "volume" => Ok(Dimension::Volume),
        other => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unknown dimension: '{}'. Valid values: AmountOfMoney, CreditCardNumber, \
             Distance, Duration, Email, Numeral, Ordinal, PhoneNumber, Quantity, \
             Temperature, Time, Url, Volume",
            other
        ))),
    }
}

/// A Python-accessible Duckling parser.
///
/// Usage:
///     from py_duckling_native import DucklingParser
///     parser = DucklingParser("America/New_York")
///     results = parser.parse("Meet me tomorrow at 3pm, it costs $50")
///     results = parser.parse("dinner at 8pm", dimensions=["Time"])
#[pyclass]
struct DucklingParser {
    inner: Duckling,
}

#[pymethods]
impl DucklingParser {
    /// Create a new parser with the given IANA timezone.
    ///
    /// Args:
    ///     timezone: IANA timezone string (e.g. "America/New_York", "UTC").
    ///               Defaults to "UTC".
    #[new]
    #[pyo3(signature = (timezone="UTC"))]
    fn new(timezone: &str) -> Self {
        DucklingParser {
            inner: Duckling::new(timezone),
        }
    }

    /// Parse text and extract structured entities.
    ///
    /// Args:
    ///     text: The input text to parse.
    ///     dimensions: Optional list of dimensions to extract.
    ///                 If None or empty, all dimensions are extracted.
    ///                 Valid values: "AmountOfMoney", "CreditCardNumber",
    ///                 "Distance", "Duration", "Email", "Numeral", "Ordinal",
    ///                 "PhoneNumber", "Quantity", "Temperature", "Time",
    ///                 "Url", "Volume"
    ///
    /// Returns:
    ///     List of dicts, each with keys: dim, body, start, end, value
    #[pyo3(signature = (text, dimensions=None))]
    fn parse(
        &self,
        py: Python<'_>,
        text: &str,
        dimensions: Option<Vec<String>>,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let dims: Vec<Dimension> = match &dimensions {
            Some(dim_strs) => dim_strs
                .iter()
                .map(|s| parse_dimension(s))
                .collect::<PyResult<Vec<_>>>()?,
            None => vec![],
        };

        let entities = self
            .inner
            .parse(text, &dims)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

        let mut results = Vec::with_capacity(entities.len());
        for entity in entities {
            let dict = PyDict::new(py);
            dict.set_item("dim", &entity.dim)?;
            dict.set_item("body", &entity.body)?;
            dict.set_item("start", entity.start)?;
            dict.set_item("end", entity.end)?;

            let value_str = entity.value.to_string();
            let json_mod = py.import("json")?;
            let py_value = json_mod.call_method1("loads", (value_str,))?;
            dict.set_item("value", py_value)?;

            results.push(dict.into());
        }

        Ok(results)
    }
}

/// Convenience function: parse text without creating a parser object.
///
/// Args:
///     text: Input text to parse.
///     timezone: IANA timezone (default "UTC").
///     dimensions: Optional list of dimensions to filter by.
///
/// Returns:
///     List of entity dicts.
#[pyfunction]
#[pyo3(signature = (text, timezone="UTC", dimensions=None))]
fn parse(
    py: Python<'_>,
    text: &str,
    timezone: &str,
    dimensions: Option<Vec<String>>,
) -> PyResult<Vec<Py<PyAny>>> {
    let parser = DucklingParser::new(timezone);
    parser.parse(py, text, dimensions)
}

/// List all supported dimension names.
#[pyfunction]
fn supported_dimensions() -> Vec<&'static str> {
    vec![
        "AmountOfMoney",
        "CreditCardNumber",
        "Distance",
        "Duration",
        "Email",
        "Numeral",
        "Ordinal",
        "PhoneNumber",
        "Quantity",
        "Temperature",
        "Time",
        "Url",
        "Volume",
    ]
}

/// Python module definition
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<DucklingParser>()?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(supported_dimensions, m)?)?;
    Ok(())
}
