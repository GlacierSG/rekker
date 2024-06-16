use super::py::{save_recv_timeout_wrapper, save_send_timeout_wrapper, py_parse_duration, inp_to_bytes};
use super::pipe::Pipe;
use pyo3::{Py, PyAny, Python, PyResult, pyclass, pymethods, types::PyBytes};

#[pyclass]
pub struct Udp {
    stream: crate::Udp
}

#[pymethods]
impl Udp {
    #[new] 
    fn connect(addr: &str) -> std::io::Result<Udp> {
        Ok(Udp {
            stream: crate::Udp::connect(addr)?
        })
    }

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
