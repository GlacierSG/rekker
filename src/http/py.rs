use pyo3::types::PyModule;
use pyo3::{Bound, pymethods, PyResult, pyclass, pyfunction, wrap_pyfunction};
use pyo3::prelude::PyModuleMethods;
use crate::{Result, http};

pub fn http(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let http_m = PyModule::new_bound(m.py(), "http")?;

    http_m.add_class::<Req>()?;
    http_m.add_function(wrap_pyfunction!(get, &http_m)?)?;
    http_m.add_function(wrap_pyfunction!(post, &http_m)?)?;
    http_m.add_function(wrap_pyfunction!(put, &http_m)?)?;
    http_m.add_function(wrap_pyfunction!(delete, &http_m)?)?;

    m.add_submodule(&http_m)?;
    Ok(())
}

#[pyfunction]
pub fn get(addr: String) -> Result<Req> { Ok(Req::get(&addr)?) }
#[pyfunction]
pub fn post(addr: String) -> Result<Req> { Ok(Req::post(&addr)?) }
#[pyfunction]
pub fn put(addr: String) -> Result<Req> { Ok(Req::put(&addr)?) }
#[pyfunction]
pub fn delete(addr: String) -> Result<Req> { Ok(Req::delete(&addr)?) }

#[pyclass]
#[derive(Clone)]
pub struct Req {
    req: crate::http::Req
}

impl Req {
    pub fn get(addr: &str) -> Result<Req> { Ok(Req { req: http::Req::get(addr) }) }
    pub fn post(addr: &str) -> Result<Req> { Ok(Req { req: http::Req::post(addr) }) }
    pub fn put(addr: &str) -> Result<Req> { Ok(Req { req: http::Req::put(addr) }) }
    pub fn delete(addr: &str) -> Result<Req> { Ok(Req { req: http::Req::delete(addr) }) }
}

#[pymethods]
impl Req {
    fn __str__(&self) -> String {
        self.req.to_string()
    }
    fn send(&self) -> Result<Res> {
        Ok(Res { res: self.req.send()? })
    }
}

#[pyclass]
pub struct Res {
    res: crate::http::Res
}
