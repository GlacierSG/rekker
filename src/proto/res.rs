use crate::literal::to_lit_colored;
use colored::*;
use crate::{Pipe, Result, Error};

pub struct Res {
    pub status: u8,
    pub raw_headers: Vec<(Vec<u8>, Vec<u8>)>,
    pub raw_body: Vec<u8>,
}

impl Res {
    pub fn new() -> Self {
        Self {
            status: 0,
            raw_headers: vec![],
            raw_body: vec![],
        }
    }

    pub fn from_pipe(mut io: impl Pipe) -> Result<()> {
        match io.recvline() {
            Ok(v) => {
                v.split(|&x| x == 32);
                return Ok(());
            },
            _ => return Err(Error::ParsingError("io error".to_string()))
        }

        Ok(())
    }
}


