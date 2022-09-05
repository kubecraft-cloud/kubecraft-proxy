use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub mod packets;

/// It reads a variable length integer from a stream
///
/// Arguments:
///
/// * `stream`: The stream to read from
///
/// Returns:
///
/// A Result<i32>
pub async fn read_var_int<T>(stream: &mut T) -> Result<i32>
where
    T: AsyncReadExt + std::marker::Unpin,
{
    let mut num_read: i32 = 0;
    let mut result: i32 = 0;

    loop {
        let read = stream.read_u8().await? as i32;
        let value = read & 0b0111_1111;

        result |= value << (7 * num_read);
        num_read += 1;

        if num_read > 5 {
            return Err(anyhow!("VarInt too big!"));
        }

        if (read & 0b1000_0000) == 0 {
            break;
        }
    }

    Ok(result)
}

/// It writes a variable length integer to a stream
///
/// Arguments:
///
/// * `stream`: The stream to write to
/// * `value`: The value to write
///
/// Returns:
///
/// A Result<()>
pub async fn write_var_int<T>(stream: &mut T, mut value: i32) -> Result<()>
where
    T: AsyncWriteExt + std::marker::Unpin,
{
    loop {
        let mut temp: i16 = (value & 0b0111_1111) as i16;
        value >>= 7;

        if value != 0 {
            temp |= 0b1000_0000;
        }

        stream.write_i8(temp as i8).await?;
        if value == 0 {
            break Ok(());
        }
    }
}

/// It reads a string from a stream
///
/// Arguments:
///
/// * `stream`: The stream to read from.
///
/// Returns:
///
/// A Result<String>
pub async fn read_string<T>(stream: &mut T) -> Result<String>
where
    T: AsyncReadExt + std::marker::Unpin,
{
    let length = read_var_int(stream).await?;
    let mut buf = vec![0u8; length as usize];

    stream.read_exact(&mut buf).await?;

    Ok(String::from_utf8_lossy(&buf).to_string())
}

/// It writes a string to a stream
///
/// Arguments:
///
/// * `stream`: The stream to write to.
/// * `string`: The string to write to the stream.
///
/// Returns:
///
/// Result<()>
pub async fn write_string<T>(stream: &mut T, string: &str) -> Result<()>
where
    T: AsyncWriteExt + std::marker::Unpin,
{
    write_var_int(stream, string.len() as i32).await?;
    stream.write_all(string.as_bytes()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // test read_var_int function with data from https://wiki.vg/VarInt_And_VarLong

    #[tokio::test]
    async fn test_read_var_int_0() {
        let mut stream = &b"\x00"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_read_var_int_1() {
        let mut stream = &b"\x01"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn test_read_var_int_2() {
        let mut stream = &b"\x02"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 2);
    }

    #[tokio::test]
    async fn test_read_var_int_127() {
        let mut stream = &b"\x7f"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 127);
    }

    #[tokio::test]
    async fn test_read_var_int_128() {
        let mut stream = &b"\x80\x01"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 128);
    }

    #[tokio::test]
    async fn test_read_var_int_255() {
        let mut stream = &b"\xff\x01"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 255);
    }

    #[tokio::test]
    async fn test_read_var_int_25565() {
        let mut stream = &b"\xdd\xc7\x01"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 25565);
    }

    #[tokio::test]
    async fn test_read_var_int_2097151() {
        let mut stream = &b"\xff\xff\x7f"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 2097151);
    }

    #[tokio::test]
    async fn test_read_var_int_2147483647() {
        let mut stream = &b"\xff\xff\xff\xff\x07"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, 2147483647);
    }

    #[tokio::test]
    async fn test_read_var_int_minus_1() {
        let mut stream = &b"\xff\xff\xff\xff\x0f"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, -1);
    }

    #[tokio::test]
    async fn test_read_var_int_minus_2147483648() {
        let mut stream = &b"\x80\x80\x80\x80\x08"[..];
        let result = super::read_var_int(&mut stream).await.unwrap();
        assert_eq!(result, -2147483648);
    }

    #[tokio::test]
    async fn test_read_var_int_too_long_err() {
        let mut stream = &b"\xff\xff\xff\xff\xcf"[..];
        let err = super::read_var_int(&mut stream).await.is_err();
        assert!(err);
    }

    #[tokio::test]
    async fn test_write_var_int_0() {
        let mut stream = Vec::new();
        super::write_var_int(&mut stream, 0).await.unwrap();
        assert_eq!(stream, b"\x00");
    }

    #[tokio::test]
    async fn test_write_var_int_128() {
        let mut stream = Vec::new();
        super::write_var_int(&mut stream, 128).await.unwrap();
        assert_eq!(stream, b"\x80\x01");
    }

    #[tokio::test]
    async fn test_write_var_int_25565() {
        let mut stream = Vec::new();
        super::write_var_int(&mut stream, 25565).await.unwrap();
        assert_eq!(stream, b"\xdd\xc7\x01");
    }

    #[tokio::test]
    async fn test_write_var_int_2147483647() {
        let mut stream = Vec::new();
        super::write_var_int(&mut stream, 2147483647).await.unwrap();
        assert_eq!(stream, b"\xff\xff\xff\xff\x07");
    }

    #[tokio::test]
    async fn test_read_string_hello_world() {
        let mut stream = &b"\x0cHello, world"[..];
        let result = super::read_string(&mut stream).await.unwrap();
        assert_eq!(result, "Hello, world");
    }

    #[tokio::test]
    async fn test_read_string_hello_world_to_short() {
        let mut stream = &b"\x0bHello, world"[..];
        let result = super::read_string(&mut stream).await.unwrap();
        assert_eq!(result, "Hello, worl");
    }

    #[tokio::test]
    async fn test_write_string_hello_world() {
        let mut stream = Vec::new();
        super::write_string(&mut stream, "Hello, world")
            .await
            .unwrap();
        assert_eq!(stream, b"\x0cHello, world");
    }
}
