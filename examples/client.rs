use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub const PASSWORD: &str = "8d969eef6ecad3c29a3a629280e686cf0c3f5d5a86aff3ca12020c923adc6c92";

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let addr = "127.0.0.1:25131";
    let mut stream = TcpStream::connect(addr).await.unwrap();

    let code = stream.read_u8().await.unwrap();
    println!("read code: {code}");
    // code=100: IntractInputPassword
    if code != 100 {
        panic!("server accept failed. error code {code}")
    }
    // auth
    println!("write password: {PASSWORD}");
    stream.write_all(PASSWORD.as_bytes()).await?;

    let code = stream.read_u8().await.unwrap();
    if code != 0 {
        panic!("server auth failed. error code {code}")
    }
    println!("server auth success.");
    println!("close connection.");
    Ok(())
}
