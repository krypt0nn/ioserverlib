use std::io::{Read, Write, BufRead};

pub trait Serializer<R: Read, W: Write> {
    type Error;
    type Message;

    fn try_read(&self, reader: &mut R) -> Result<Option<Self::Message>, Self::Error>;

    fn read(&self, reader: &mut R) -> Result<Self::Message, Self::Error> {
        loop {
            if let Some(message) = self.try_read(reader)? {
                return Ok(message);
            }
        }
    }

    fn write(&self, writer: &mut W, message: Self::Message) -> Result<(), Self::Error>;
}

#[cfg(feature = "json-serializer")]
pub trait JsonSerializer {
    type Error: From<std::io::Error> + From<serde_json::Error>;
    type Message: serde::Serialize + serde::de::DeserializeOwned;
}

#[cfg(feature = "json-serializer")]
impl<R: BufRead, W: Write, S: JsonSerializer> Serializer<R, W> for S {
    type Error = S::Error;
    type Message = S::Message;

    fn try_read(&self, reader: &mut R) -> Result<Option<Self::Message>, Self::Error> {
        let mut buf = String::new();

        reader.read_line(&mut buf)?;

        let buf = buf.trim_ascii();

        if !buf.is_empty() {
            let message = serde_json::from_slice::<Self::Message>(buf.as_bytes())?;

            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    fn write(&self, writer: &mut W, message: Self::Message) -> Result<(), Self::Error> {
        let message = serde_json::to_vec(&message)?;

        writer.write_all(&message)?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        Ok(())
    }
}
