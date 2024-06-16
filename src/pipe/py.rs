use pyo3::prelude::*;
use crate::pipe::pipe::Pipe;
use pyo3::types::{PyBytes, PyString, PyAny};
use std::time::Duration;
use humantime::parse_duration;
use super::py_tcp::Tcp;
use super::py_udp::Udp;
use super::py_tls::Tls;

pub(crate) fn inp_to_bytes(obj: &PyAny) -> PyResult<Vec<u8>> {
    if obj.is_instance_of::<PyString>() {
        let s: String = obj.extract()?;
        Ok(s.as_bytes().to_vec())
    } else if obj.is_instance_of::<PyBytes>() {
        let b: Vec<u8> = obj.extract()?;
        Ok(b)
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected a string or bytes object",
        ))
    }
}

pub(crate) fn py_parse_duration(duration: Option<&str>) -> PyResult<Option<Duration>> {
    match duration {
        Some(dur) => {
            match parse_duration(dur) {
                Ok(d) => Ok(Some(d)),
                Err(e) => {
                    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        format!("{}", e),
                    ))
                },
            }
        },
        None => Ok(None),
    }
}


pub fn pipes(_py: Python, m: &PyModule)  -> PyResult<()> {
    m.add_class::<Tcp>()?;
    m.add_class::<Udp>()?;
    m.add_class::<Tls>()?;
    Ok(())
}

macro_rules! save_recv_timeout_wrapper {
    ($self:expr, $func:expr, $timeout:expr) => {{
        let save_timeout = $self.stream.recv_timeout()?;
        $self.stream.set_recv_timeout(py_parse_duration($timeout)?)?;
        let out = match $func {
            Ok(d) => d,
            Err(e) => {
                $self.stream.set_recv_timeout(save_timeout)?;
                return Err(e.into());
            }
        };

        $self.stream.set_recv_timeout(save_timeout)?;
        out
    }}
}
pub(crate) use save_recv_timeout_wrapper;

macro_rules! save_send_timeout_wrapper {
    ($self:expr, $func:expr, $timeout:expr) => {{
        let save_timeout = $self.stream.send_timeout()?;
        $self.stream.set_send_timeout(py_parse_duration($timeout)?)?;
        let out = match $func {
            Ok(d) => d,
            Err(e) => {
                $self.stream.set_send_timeout(save_timeout)?;
                return Err(e.into());
            }
        };

        $self.stream.set_send_timeout(save_timeout)?;
        out
    }}
}
pub(crate) use save_send_timeout_wrapper;


