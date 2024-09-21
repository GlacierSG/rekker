use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString, PyAny, PyLong, PyFunction};
use pyo3::wrap_pyfunction;

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

pub fn inject(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Injector>()?;
    Ok(())
}


#[pyclass]
pub struct Injector {
    v: crate::Injector
}
#[pymethods]
impl Injector {
    #[new]
    #[pyo3(signature = ())]
    fn new() -> Injector {
        let mut inj = crate::Injector::new();
        Injector { v: inj }
    }

    #[getter]
    fn get_file(&self, py: Python) -> PyResult<Option<Py<PyBytes>>> {
        if let Some(file) = &self.v.file {
            Ok(Some(PyBytes::new_bound(py, file).into()))
        }
        else {
            Ok(None)
        }
    }
    #[setter]
    fn set_file(&mut self, value: Option<Bound<'_, PyAny>>) -> PyResult<()> {
        if let Some(v) = value {
            let v = pyany_to_bytes(&v)?;
            self.v.file = Some(v);
            Ok(())
        }
        else {
            self.v.file = None;
            Ok(())
        }
    }

    #[getter]
    fn get_host(&self, py: Python) -> PyResult<Option<Py<PyBytes>>> {
        if let Some(host) = &self.v.host {
            Ok(Some(PyBytes::new_bound(py, host).into()))
        }
        else {
            Ok(None)
        }
    }
    #[setter]
    fn set_host(&mut self, value: Option<Bound<'_, PyAny>>) -> PyResult<()> {
        if let Some(v) = value {
            let v = pyany_to_bytes(&v)?;
            self.v.host = Some(v);
            Ok(())
        }
        else {
            self.v.host = None;
            Ok(())
        }
    }

    #[getter]
    fn get_port(&self) -> PyResult<Option<u16>> {
        Ok(self.v.port)
    }
    #[setter]
    fn set_port(&mut self, value: Option<u16>) {
        self.v.port = value;
    }


    #[getter]
    fn get_use_http(&self) -> PyResult<bool> {
        Ok(self.v.use_http)
    }
    #[setter]
    fn set_use_http(&mut self, value: bool) {
        self.v.use_http = value;
    }

    #[getter]
    fn get_use_https(&self) -> PyResult<bool> {
        Ok(self.v.use_https)
    }
    #[setter]
    fn set_use_https(&mut self, value: bool) {
        self.v.use_https = value;
    }

    pub fn payload(&self, py: Python, n:u64) -> PyResult<Option<Py<PyBytes>>> {
        Ok(Some(PyBytes::new_bound(py, &self.v.payload(n)?).into()))
    }
}
