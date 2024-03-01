use pyo3::{pyclass, pymethods, PyRef, PyRefMut};

use std::fs::{File, OpenOptions};
use std::io::{
    BufReader,
    BufWriter,
    Read,
    Write,
};

const DEFAULT_TOKEN_ESCAPE_SEQUENCE: [u8; 2] = [0xff, 0xff];

#[pyclass]
struct TokensReader {
    escape_sequence: [u8; 2],
    tokens_file: BufReader<File>,
    r_buff: [u8; 2],
    e_buff: Vec<u16>,
    readable: bool,
}

#[pymethods]
impl TokensReader {
    #[new]
    fn new(tokens_file_path: String, escape_sequence: Option<[u8; 2]>) -> Self {
        Self {
            escape_sequence: match escape_sequence {
                Some(s) => s,
                None => DEFAULT_TOKEN_ESCAPE_SEQUENCE,
            },
            tokens_file: BufReader::new(
                OpenOptions::new()
                    .write(false)
                    .read(true)
                    .open(&tokens_file_path)
                    .expect("Failed to open out file for read."),
            ),
            readable: true,
            e_buff: Vec::new(),
            r_buff: [0, 0],
        }
    }

    fn read_line(&mut self) -> Option<Vec<u16>> {
        loop {
            if self.tokens_file.read_exact(&mut self.r_buff).is_ok() {
                if self.r_buff != self.escape_sequence {
                    self.e_buff
                        .push(((self.r_buff[0] as u16) << 8) | (self.r_buff[1] as u16));
                } else {
                    break;
                };
            } else {
                self.readable = false;
                return None;
            }
        }
        let out: Vec<u16> = self.e_buff.clone();
        self.e_buff.clear();
        return Some(out);
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Vec<u16>> {
        slf.read_line()
    }
}

#[pyclass]
struct TokensWriter {
    escape_sequence: [u8; 2],
    tokens_file: BufWriter<File>,
}

#[pymethods]
impl TokensWriter {
    #[new]
    fn new(tokens_file_path: String, escape_sequence: Option<[u8; 2]>) -> Self {
        Self {
            escape_sequence: match escape_sequence {
                Some(s) => s,
                None => DEFAULT_TOKEN_ESCAPE_SEQUENCE,
            },
            tokens_file: BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .open(&tokens_file_path)
                    .expect("Failed to open out file for write."),
            ),
        }
    }

    fn write_lines(&mut self, lines: Vec<Vec<u16>>) {
        #[cfg(target_endian = "big")]
        {
            compile_error!("Sorry.")
        };
        for (i, line) in lines.into_iter().enumerate() {
            self.write_line(line);
            if i % 10000 == 0 {
                self.tokens_file
                    .flush()
                    .expect("Failed to flush tokens file")
            };
        }
    }

    fn write_line(&mut self, line: Vec<u16>) {
        let mut bytes: Vec<u8> = line
            .iter()
            .flat_map(|x| x.to_le_bytes().into_iter())
            .collect();
        bytes.extend(self.escape_sequence);
        self.tokens_file
            .write_all(&bytes)
            .expect("Failed to write tokens");
    }
}

#[pyo3::pymodule]
fn pykenizer(_py: pyo3::Python<'_>, m: &pyo3::types::PyModule) -> pyo3::PyResult<()> {
    m.add_class::<TokensReader>()?;
    m.add_class::<TokensWriter>()?;
    Ok(())
}
