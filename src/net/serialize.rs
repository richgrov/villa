use std::{io::{Error, Write, ErrorKind}, collections::HashMap};

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

pub async fn read_entity_attributes<R: AsyncReadExt + std::marker::Unpin>(
    reader: &mut R
) -> Result<HashMap<i8, EntityAttributeValue>, Error> {
    let mut entries = HashMap::with_capacity(2);

    loop {
        let header = reader.read_i8().await?;
        if header == 127 {
            break
        }

        let ty = header >> 5;
        let id = header & 0b11111;
        entries.insert(id, EntityAttributeValue::read(ty, reader).await?);
    }

    Ok(entries)
}

pub enum EntityAttributeValue {
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
    Str(String),
    Item { id: i16, num_items: i8, data: i16, },
    ChunkPos(i32, i32, i32),
}

impl EntityAttributeValue {
    pub async fn read<R: AsyncReadExt + std::marker::Unpin>(ty: i8, reader: &mut R) -> Result<EntityAttributeValue, Error> {
        Ok(match ty {
            0 => EntityAttributeValue::I8(reader.read_i8().await?),
            1 => EntityAttributeValue::I16(reader.read_i16().await?),
            2 => EntityAttributeValue::I32(reader.read_i32().await?),
            3 => EntityAttributeValue::F32(reader.read_f32().await?),
            4 => EntityAttributeValue::Str(read_str(reader, 64).await?),
            5 => EntityAttributeValue::Item{
                id: reader.read_i16().await?,
                num_items: reader.read_i8().await?,
                data: reader.read_i16().await?,
            },
            6 => EntityAttributeValue::ChunkPos(
                reader.read_i32().await?,
                reader.read_i32().await?,
                reader.read_i32().await?,
            ),
            other => return Err(Error::new(ErrorKind::InvalidInput, format!("entity data type {} is not supported", other))),
        })
    }
}
