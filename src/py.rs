use pyo3::prelude::*;
use super::pipe::py::pipes;
use std::process;


#[pymodule]
#[pyo3(name = "rekker")]
fn rekker(m: &Bound<'_, PyModule>) -> PyResult<()> {
    ctrlc::set_handler(move || {
        process::exit(130); 
    }).expect("Error setting Ctrl+C handler");

    let _ = pipes(&m);
    Ok(())
}

