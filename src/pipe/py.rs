use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString, PyAny};
use std::time::Duration;
use humantime::parse_duration;
use crate::{Error, Result};

pub fn pipes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pipe>()?;
    m.add_function(wrap_pyfunction!(remote, m)?)?;
    Ok(())
}

#[pyfunction]
pub fn remote(addr: String, port: Option<usize>, tls: Option<bool>, udp: Option<bool>) -> PyResult<Pipe> {
    match (tls, udp) {
        (None, None) => {
            Ok(Pipe::tcp(&addr)?)
        },
        (None, Some(true)) => {
            Ok(Pipe::udp(&addr)?)
        },
        (Some(true), None) => {
            Ok(Pipe::tls(&addr)?)
        },
        _ => todo!()
    }

}

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


fn py_parse_duration(duration: Option<&str>) -> PyResult<Option<Duration>> {
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

#[pyclass]
pub struct Pipe {
    stream: crate::Pipe
}
impl Pipe {
    fn tcp(addr: &str) -> Result<Pipe> {
        Ok(Pipe { stream: crate::Pipe::tcp(addr)? })
    }
    fn udp(addr: &str) -> Result<Pipe> {
        Ok(Pipe { stream: crate::Pipe::udp(addr)? })
    }
    fn tls(addr: &str) -> Result<Pipe> {
        Ok(Pipe { stream: crate::Pipe::tls(addr)? })
    }
}

#[pymethods]
impl Pipe {
    fn set_nagle(&mut self, value: bool) -> PyResult<()> {
        Ok(self.stream.set_nagle(value)?)
    }
    pub fn nagle(&mut self) -> Result<bool> {
        Ok(self.stream.nagle()?)
    }

    fn log(&mut self, value: bool) -> () {
        self.stream.log(value);
    }
    fn is_logging(&mut self) -> () {
        self.stream.is_logging();
    }

    #[pyo3(signature = (size, timeout=None))]
    fn recv(&mut self, py: Python, size: usize, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let out = save_recv_timeout_wrapper!(self, self.stream.recv(size), timeout);

        Ok(PyBytes::new_bound(py, &out).into())
    }
    #[pyo3(signature = (size, timeout=None))]
    fn recvn(&mut self, py: Python, size: usize, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let out = save_recv_timeout_wrapper!(self, self.stream.recvn(size), timeout);

        Ok(PyBytes::new_bound(py, &out).into())
    }
    #[pyo3(signature = (drop=None, timeout=None))]
    fn recvline(&mut self, py: Python, drop: Option<bool>, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let mut out = save_recv_timeout_wrapper!(self, self.stream.recvline(), timeout);
        
        match drop {
            Some(true) => {
                out = out[..out.len()-1].to_vec(); 
                },
            _ => {}
        }
        Ok(PyBytes::new_bound(py, &out).into())
    }
    #[pyo3(signature = (suffix, drop=None, timeout=None))]
    fn recvuntil(&mut self, py: Python, suffix: Bound<'_, PyAny>, drop: Option<bool>, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let suffix = pyany_to_bytes(&suffix)?;

        let mut out = save_recv_timeout_wrapper!(self, self.stream.recvuntil(suffix), timeout);

        match drop {
            Some(true) => {
                out = out[..out.len()-1].to_vec(); 
                },
            _ => {}
        }

        Ok(PyBytes::new_bound(py, &out).into())
    }
    #[pyo3(signature = (timeout=None))]
    fn recvall(&mut self, py: Python, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let out = save_recv_timeout_wrapper!(self, self.stream.recvall(), timeout);

        Ok(PyBytes::new_bound(py, &out).into())
    }
    #[pyo3(signature = (data, timeout=None))]
    fn send(&mut self, _py: Python, data: Bound<'_, PyAny>, timeout: Option<&str>) -> PyResult<()> {
        let data = pyany_to_bytes(&data)?;
        let out = save_send_timeout_wrapper!(self, self.stream.send(data), timeout);
        Ok(out)
    }
    #[pyo3(signature = (data, timeout=None))]
    fn sendline(&mut self, _py: Python, data: Bound<'_, PyAny>, timeout: Option<&str>) -> PyResult<()> {
        let data = pyany_to_bytes(&data)?;
        let out = save_send_timeout_wrapper!(self, self.stream.sendline(data), timeout);
        Ok(out)
    }
    #[pyo3(signature = (data, suffix, timeout=None))]
    fn sendlineafter(&mut self, py: Python, data: Bound<'_, PyAny>, suffix: Bound<'_, PyAny>, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let data = pyany_to_bytes(&data)?;
        let suffix = pyany_to_bytes(&suffix)?;
        let out = save_send_timeout_wrapper!(self, self.stream.sendlineafter(data, suffix), timeout);
        Ok(PyBytes::new_bound(py, &out).into())
    }

    fn recv_timeout(&self, _py: Python) -> PyResult<Option<String>> {
        match self.stream.recv_timeout()? {
            Some(duration) => Ok(Some(format!("{:?}", duration))),
            None => Ok(None)
        }
    }
    #[pyo3(signature = (duration))]
    fn set_recv_timeout(&mut self, _py: Python, duration: Option<&str>) -> PyResult<()> {
        Ok(self.stream.set_recv_timeout(py_parse_duration(duration)?)?)
    }

    fn send_timeout(&self, _py: Python) -> PyResult<Option<String>> {
        match self.stream.send_timeout()? {
            Some(duration) => Ok(Some(format!("{:?}", duration))),
            None => Ok(None)
        }
    }
    #[pyo3(signature = (duration))]
    fn set_send_timeout(&mut self, _py: Python, duration: Option<&str>) -> PyResult<()> {
        Ok(self.stream.set_send_timeout(py_parse_duration(duration)?)?)
    }

    fn debug(&mut self, _py: Python) -> PyResult<()> {
        Ok(self.stream.debug()?)
    }
    fn interactive(&mut self, _py: Python) -> PyResult<()> {
        Ok(self.stream.interactive()?)
    }

    fn close(&mut self, _py: Python) -> PyResult<()> {
        Ok(self.stream.close()?)
    }

}



/*


#[pyclass]
pub struct TcpListen {
    listener: super::tcp_listen::TcpListen
}


#[pymethods]
impl TcpListen {
    #[new]
    fn new(address: &str) -> PyResult<Self> {
        Ok( TcpListen{ listener: super::tcp_listen::TcpListen::new(address)? } )
    }

    fn accept(&self, py: Python) -> PyResult<(PyObject, String)> {
        let (stream, addr) = self.listener
            .accept()?;
        let py_stream = Py::new(py, Tcp { stream })?;
        Ok((py_stream.to_object(py), addr.to_string()))
    }
}

*/

