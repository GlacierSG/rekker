use pyo3::prelude::*;
use crate::Pipe;
use crate::pipe::py::*;
use std::process;
use crate::Error;

use pyo3::exceptions::PyRuntimeError;
use pyo3::PyErr;

impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        match err {
            _ => PyRuntimeError::new_err("Another error occurred"),
        }
    }
}


#[pymodule]
#[pyo3(name = "rekker")]
fn rekker(m: &Bound<'_, PyModule>) -> PyResult<()> {
    ctrlc::set_handler(move || {
        process::exit(130); 
    }).expect("Error setting Ctrl+C handler");

    let _ = pipes(&m);
    Ok(())
}

