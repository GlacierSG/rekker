use pyo3::prelude::*;
use super::pipe::py::pipes;
use std::process;


#[pymodule]
#[pyo3(name = "rekker")]
fn rekker(py: Python, m: &PyModule) -> PyResult<()> {
    ctrlc::set_handler(move || {
        process::exit(130); 
    }).expect("Error setting Ctrl+C handler");

    let _ = pipes(py, &m);
    Ok(())
}

