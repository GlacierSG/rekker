# Rekker

Rekker is inspired by pwntools features for communicating with tcp sockets. 

Rekker is still in development.

## Example
```python
import rekker
io = rekker.Tcp("localhost:1234")
io.send(b"abc")
io.log(True)
io.sendline(b"abcd")
io.sendlineafter(b"abc", b"cde")
io.recv(123)
io.recvn(123)
io.recvline()
io.recvuntil(b"abc")
io.recvall()
io.interactive()
io.debug()
io.close()
```
## Install
### Rust
```bash
cargo add rekker
```
### Python
```bash
pip install rekker
```
