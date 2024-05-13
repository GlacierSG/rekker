# Rekker

Rekker is inspired by pwntools features for communicating with tcp sockets. 
It is still in development and a lot of features do not exist or have not been tested properly.

## Example
```python
import rekker
io = rekker.Tcp("localhost:1234")
io.sendline(b"abcd")
io.interactive()
```
