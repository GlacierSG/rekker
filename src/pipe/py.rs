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

macro_rules! impl_py_stream {
    ($type:tt) => {
        #[pyclass]
        pub struct $type {
            stream: crate::$type
        }

        #[pymethods]
        impl $type {
            fn recv(&mut self, py: Python, size: usize, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
                let out = save_recv_timeout_wrapper!(self, self.stream.recv(size), timeout);

                Ok(PyBytes::new(py, &out).into())
            }
            fn recvn(&mut self, py: Python, size: usize, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
                let out = save_recv_timeout_wrapper!(self, self.stream.recvn(size), timeout);

                Ok(PyBytes::new(py, &out).into())
            }
            fn recvline(&mut self, py: Python, drop: Option<bool>, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
                let mut out = save_recv_timeout_wrapper!(self, self.stream.recvline(), timeout);
                
                match drop {
                    Some(true) => {
                        out = out[..out.len()-1].to_vec(); 
                        },
                    _ => {}
                }
                Ok(PyBytes::new(py, &out).into())
            }
            fn recvuntil(&mut self, py: Python, suffix: &PyAny, drop: Option<bool>, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
                let suffix = inp_to_bytes(&suffix)?;

                let mut out = save_recv_timeout_wrapper!(self, self.stream.recvuntil(suffix), timeout);

                match drop {
                    Some(true) => {
                        out = out[..out.len()-1].to_vec(); 
                        },
                    _ => {}
                }

                Ok(PyBytes::new(py, &out).into())
            }
            fn recvall(&mut self, py: Python, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
                let out = save_recv_timeout_wrapper!(self, self.stream.recvall(), timeout);

                Ok(PyBytes::new(py, &out).into())
            }

            fn send(&mut self, _py: Python, data: &PyAny, timeout: Option<&str>) -> PyResult<()> {
                let data = inp_to_bytes(&data)?;
                let out = save_send_timeout_wrapper!(self, self.stream.send(data), timeout);
                Ok(out)
            }
            fn sendline(&mut self, _py: Python, data: &PyAny, timeout: Option<&str>) -> PyResult<()> {
                let data = inp_to_bytes(&data)?;
                let out = save_send_timeout_wrapper!(self, self.stream.sendline(data), timeout);
                Ok(out)
            }
            fn sendlineafter(&mut self, py: Python, data: &PyAny, suffix: &PyAny, timeout: Option<&str>) -> PyResult<Py<PyBytes>> {
                let data = inp_to_bytes(&data)?;
                let suffix = inp_to_bytes(&suffix)?;
                let out = save_send_timeout_wrapper!(self, self.stream.sendlineafter(data, suffix), timeout);
                Ok(PyBytes::new(py, &out).into())
            }

            fn recv_timeout(&self, _py: Python) -> PyResult<Option<String>> {
                match self.stream.recv_timeout()? {
                    Some(duration) => Ok(Some(format!("{:?}", duration))),
                    None => Ok(None)
                }
            }
            fn set_recv_timeout(&mut self, _py: Python, duration: Option<&str>) -> PyResult<()> {
                Ok(self.stream.set_recv_timeout(py_parse_duration(duration)?)?)
            }

            fn send_timeout(&self, _py: Python) -> PyResult<Option<String>> {
                match self.stream.send_timeout()? {
                    Some(duration) => Ok(Some(format!("{:?}", duration))),
                    None => Ok(None)
                }
            }
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
    }
}

impl_py_stream!(Tcp);
impl_py_stream!(Tls);
impl_py_stream!(Udp);

#[pymethods]
impl Tcp {
    #[new] 
    fn connect(addr: &str) -> std::io::Result<Tcp> {
        Ok(Tcp {
            stream: crate::Tcp::connect(addr)?
        })
    }

    fn set_nagle(&mut self, _py: Python, nagle: bool) -> PyResult<()> {
        Ok(self.stream.set_nagle(nagle)?)
    }
    fn nagle(&self, _py: Python) -> PyResult<bool> {
        Ok(self.stream.nagle()?)
    }

    fn log(&mut self, _py: Python, logging: bool) -> () {
        self.stream.log(logging);
    }
}

#[pymethods]
impl Udp {
    #[new] 
    fn connect(addr: &str, listen: Option<bool>) -> std::io::Result<Udp> {
        if Some(true) == listen {
            return Ok(Udp {
                    stream: crate::Udp::listen(addr)?
                });
        }
        Ok(Udp {
            stream: crate::Udp::connect(addr)?
        })
    }
}

#[pymethods]
impl Tls {
    #[new] 
    fn connect(addr: &str) -> std::io::Result<Tls> {
        Ok(Tls {
            stream: crate::Tls::connect(addr)?
        })
    }
}


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


pub fn pipes(_py: Python, m: &PyModule)  -> PyResult<()> {
    m.add_class::<Tcp>()?;
    m.add_class::<TcpListen>()?;
    m.add_class::<Udp>()?;
    m.add_class::<Tls>()?;
    Ok(())
}
