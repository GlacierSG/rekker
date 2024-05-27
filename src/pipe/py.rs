use pyo3::prelude::*;
use crate::pipe::pipe::Pipe;
use pyo3::types::{PyBytes, PyString, PyAny};

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


pub fn pipes(_py: Python, m: &PyModule)  -> PyResult<()> {
    m.add_class::<Tcp>()?;
    Ok(())
}

#[pyclass]
struct Tcp {
    tcp: crate::Tcp
}

#[pymethods]
impl Tcp {
    #[new] 
    pub fn connect(addr: &str) -> std::io::Result<Tcp> {
        Ok(Tcp {
            tcp: crate::Tcp::connect(addr)?
        })
    }

    fn recv(&mut self, py: Python, size: usize) -> PyResult<Py<PyBytes>> {
        let out = self.tcp.recv(size)?;
        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvn(&mut self, py: Python, size: usize) -> PyResult<Py<PyBytes>> {
        let out = self.tcp.recvn(size)?;
        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvline(&mut self, py: Python, drop: Option<bool>) -> PyResult<Py<PyBytes>> {
        let mut out = self.tcp.recvline()?;
        
        match drop {
            Some(true) => {
                out = out[..out.len()-1].to_vec(); 
                },
            _ => {}
        }
        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvuntil(&mut self, py: Python, suffix: &PyAny, drop: Option<bool>) -> PyResult<Py<PyBytes>> {
        let suffix = inp_to_bytes(&suffix)?;
        let mut out = self.tcp.recvuntil(suffix)?;

        match drop {
            Some(true) => {
                out = out[..out.len()-1].to_vec(); 
                },
            _ => {}
        }

        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvall(&mut self, py: Python) -> PyResult<Py<PyBytes>> {
        let out = self.tcp.recvall()?;
        Ok(PyBytes::new_bound(py, &out).into())
    }

    fn send(&mut self, _py: Python, data: &PyAny) -> PyResult<()> {
        let data = inp_to_bytes(&data)?;
        Ok(self.tcp.send(data)?)
    }
    fn sendline(&mut self, _py: Python, data: &PyAny) -> PyResult<()> {
        let data = inp_to_bytes(&data)?;
        Ok(self.tcp.sendline(data)?)
    }
    fn sendlineafter(&mut self, py: Python, data: &PyAny, suffix: &PyAny) -> PyResult<Py<PyBytes>> {
        let data = inp_to_bytes(&data)?;
        let suffix = inp_to_bytes(&suffix)?;
        let out = self.tcp.sendlineafter(data, suffix)?;
        Ok(PyBytes::new_bound(py, &out).into())
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

