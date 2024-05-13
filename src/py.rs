use pyo3::prelude::*;
use super::pipe::py::pipes;


#[pymodule]
#[pyo3(name = "rekker")]
fn rekker(py: Python, m: &PyModule) -> PyResult<()> {
    let _ = pipes(py, &m);
    Ok(())
}

