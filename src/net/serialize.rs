use std::io::{Error, Write, ErrorKind};

use tokio::io::AsyncReadExt;

pub async fn read_str<R: AsyncReadExt + std::marker::Unpin>(reader: &mut R, max_len: i16) -> Result<String, Error> {
    let len = reader.read_i16().await?;
    if len < 0 || len > max_len {
        return Err(Error::new(ErrorKind::InvalidInput, format!("string length {} is not within bounds 0..={}", len, max_len)))
    }

    let mut code_units = Vec::with_capacity(len as usize);
    for _ in 0..len {
        code_units.push(reader.read_u16().await?);
    }

    String::from_utf16(&code_units).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}

pub fn write_str<W: Write>(writer: &mut W, s: &str) -> Result<(), Error> {
    let utf16: Vec<_> = s.encode_utf16().collect();

    let len: i16 = match utf16.len().try_into() {
        Ok(i) => i,
        Err(_) => return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("{} code units is too long to encode", utf16.len())
        )),
    };

    writer.write(&(len as i16).to_be_bytes())?;
    for code_unit in &utf16 {
        writer.write(&code_unit.to_be_bytes())?;
    }

    Ok(())
}
