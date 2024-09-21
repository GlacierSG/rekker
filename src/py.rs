use crate::pipe::py::*;
use crate::http::py::*;
use crate::inject::py::*;
use std::process;
use crate::Error;

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use pyo3::{PyErr, PyAny, Bound, PyResult, wrap_pyfunction, pyfunction};
use pyo3::prelude::{PyModuleMethods, PyAnyMethods};
use pyo3::types::{PyBytes, PyString, PyModule};
use num_bigint::{BigInt, Sign};

fn pyany_to_bytes(data: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if data.is_instance_of::<PyString>() {
        let data: String = data.extract()?;
        return Ok(data.into_bytes());
    }
    else {
        let data: Vec<u8> = data.extract()?;
        return Ok(data);
    }
}


impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyRuntimeError::new_err(format!("Error: {}", err))
    }
}


#[pymodule]
#[pyo3(name = "rekker")]
fn rekker(m: &Bound<'_, PyModule>) -> PyResult<()> {
    ctrlc::set_handler(move || {
        process::exit(130); 
    }).expect("Error setting Ctrl+C handler");

    let _ = pipes(&m);
    let _ = http(&m);
    let _ = util(&m);
    let _ = inject(&m);

    Ok(())
}
pub fn util(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(xor, m)?)?;
    m.add_function(wrap_pyfunction!(b2l, m)?)?;
    m.add_function(wrap_pyfunction!(l2b, m)?)?;
    Ok(())
}


#[pyfunction]
fn xor(py: Python, lhs: Bound<'_, PyAny>, rhs: Bound<'_, PyAny>) -> PyResult<Py<PyBytes>> {
    let lhs = pyany_to_bytes(&lhs)?;
    let rhs= pyany_to_bytes(&rhs)?;
    let out = crate::xor(lhs, rhs);
    return Ok(PyBytes::new_bound(py, &out).into())
}

#[pyfunction]
fn b2l(py: Python, b: Bound<'_, PyAny>) -> PyResult<BigInt> {
    let b: Vec<u8> = b.extract()?;
    return Ok(BigInt::from_bytes_be(Sign::Plus, &b));
}
#[pyfunction]
fn l2b(py: Python, b: BigInt) -> PyResult<Py<PyBytes>> {
    return Ok(PyBytes::new_bound(py, &b.to_bytes_be().1).into())
}
