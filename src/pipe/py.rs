use pyo3::prelude::*;
use crate::pipe::pipe::Pipe;
use pyo3::types::{PyBytes, PyString, PyAny};
use std::time::Duration;
use humantime::parse_duration;

fn inp_to_bytes(obj: &PyAny) -> PyResult<Vec<u8>> {
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


pub fn pipes(_py: Python, m: &PyModule)  -> PyResult<()> {
    m.add_class::<Tcp>()?;
    Ok(())
}

#[pyclass]
struct Tcp {
    tcp: crate::Tcp
}

#[macro_export]
macro_rules! save_recv_timeout_wrapper {
    ($self:expr, $func:expr, $timeout:expr) => {{
        let save_timeout = $self.tcp.recv_timeout()?;
        $self.tcp.set_recv_timeout(py_parse_duration($timeout)?)?;
        println!("{:?}", $self.tcp.recv_timeout());
        let out = match $func {
            Ok(d) => d,
            Err(e) => {
                $self.tcp.set_recv_timeout(save_timeout)?;
                return Err(e.into());
            }
        };

        $self.tcp.set_recv_timeout(save_timeout)?;
        out
    }}
}

#[macro_export]
macro_rules! save_send_timeout_wrapper {
    ($self:expr, $func:expr, $timeout:expr) => {{
        let save_timeout = $self.tcp.send_timeout()?;
        $self.tcp.set_send_timeout(py_parse_duration($timeout)?)?;
        println!("{:?}", $self.tcp.send_timeout());
        let out = match $func {
            Ok(d) => d,
            Err(e) => {
                $self.tcp.set_send_timeout(save_timeout)?;
                return Err(e.into());
            }
        };

        $self.tcp.set_send_timeout(save_timeout)?;
        out
    }}
}


#[pymethods]
impl Tcp {
    #[new] 
    pub fn connect(addr: &str) -> std::io::Result<Tcp> {
        Ok(Tcp {
            tcp: crate::Tcp::connect(addr)?
        })
    }

    fn recv(&mut self, py: Python, size: usize, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let out = save_recv_timeout_wrapper!(self, self.tcp.recv(size), timeout);

        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvn(&mut self, py: Python, size: usize, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let out = save_recv_timeout_wrapper!(self, self.tcp.recvn(size), timeout);

        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvline(&mut self, py: Python, drop: Option<bool>, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let mut out = save_recv_timeout_wrapper!(self, self.tcp.recvline(), timeout);
        
        match drop {
            Some(true) => {
                out = out[..out.len()-1].to_vec(); 
                },
            _ => {}
        }
        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvuntil(&mut self, py: Python, suffix: &PyAny, drop: Option<bool>, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let suffix = inp_to_bytes(&suffix)?;

        let mut out = save_recv_timeout_wrapper!(self, self.tcp.recvuntil(suffix), timeout);

        match drop {
            Some(true) => {
                out = out[..out.len()-1].to_vec(); 
                },
            _ => {}
        }

        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvall(&mut self, py: Python, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let mut out = save_recv_timeout_wrapper!(self, self.tcp.recvall(), timeout);

        Ok(PyBytes::new_bound(py, &out).into())
    }

    fn send(&mut self, _py: Python, data: &PyAny, timeout: Option<&str>) -> PyResult<()> {
        let data = inp_to_bytes(&data)?;
        let out = save_send_timeout_wrapper!(self, self.tcp.send(data), timeout);
        Ok(out)
    }
    fn sendline(&mut self, _py: Python, data: &PyAny, timeout: Option<&str>) -> PyResult<()> {
        let data = inp_to_bytes(&data)?;
        let out = save_send_timeout_wrapper!(self, self.tcp.sendline(data), timeout);
        Ok(out)
    }
    fn sendlineafter(&mut self, py: Python, data: &PyAny, suffix: &PyAny, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
        let data = inp_to_bytes(&data)?;
        let suffix = inp_to_bytes(&suffix)?;
        let out = save_send_timeout_wrapper!(self, self.tcp.sendlineafter(data, suffix), timeout);
        Ok(PyBytes::new_bound(py, &out).into())
    }

    fn recv_timeout(&self, py: Python) -> PyResult<Option<String>> {
        match self.tcp.recv_timeout()? {
            Some(duration) => Ok(Some(format!("{:?}", duration))),
            None => Ok(None)
        }
    }
    fn set_recv_timeout(&mut self, py: Python, duration: Option<&str>) -> PyResult<()> {
        Ok(self.tcp.set_recv_timeout(py_parse_duration(duration)?)?)
    }

    fn send_timeout(&self, py: Python) -> PyResult<Option<String>> {
        match self.tcp.send_timeout()? {
            Some(duration) => Ok(Some(format!("{:?}", duration))),
            None => Ok(None)
        }
    }
    fn set_send_timeout(&mut self, py: Python, duration: Option<&str>) -> PyResult<()> {
        Ok(self.tcp.set_send_timeout(py_parse_duration(duration)?)?)
    }

    fn set_nagle(&mut self, py: Python, nagle: bool) -> PyResult<()> {
        Ok(self.tcp.set_nagle(nagle)?)
    }
    fn nagle(&self, py: Python) -> PyResult<bool> {
        Ok(self.tcp.nagle()?)
    }

    fn debug(&mut self, _py: Python) -> PyResult<()> {
        Ok(self.tcp.debug()?)
    }
    fn interactive(&mut self, _py: Python) -> PyResult<()> {
        Ok(self.tcp.interactive()?)
    }

    fn close(&mut self, _py: Python) -> PyResult<()> {
        Ok(self.tcp.close()?)
    }

}

