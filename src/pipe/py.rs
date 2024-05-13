use pyo3::prelude::*;
use pyo3::types::PyBytes;
use crate::pipe::pipe::Pipe;

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
    fn recvline(&mut self, py: Python) -> PyResult<Py<PyBytes>> {
        let out = self.tcp.recvline()?;
        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvuntil(&mut self, py: Python, suffix: &[u8]) -> PyResult<Py<PyBytes>> {
        let out = self.tcp.recvuntil(suffix)?;
        Ok(PyBytes::new_bound(py, &out).into())
    }
    fn recvall(&mut self, py: Python) -> PyResult<Py<PyBytes>> {
        let out = self.tcp.recvall()?;
        Ok(PyBytes::new_bound(py, &out).into())
    }

    fn send(&mut self, _py: Python, data: &[u8]) -> PyResult<usize> {
        Ok(self.tcp.send(data)?)
    }
    fn sendline(&mut self, _py: Python, data: &[u8]) -> PyResult<usize> {
        Ok(self.tcp.sendline(data)?)
    }
    fn sendlineafter(&mut self, py: Python, data: &[u8], suffix: &[u8]) -> PyResult<Py<PyBytes>> {
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

