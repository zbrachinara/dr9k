#[tokio::main]
async fn main() {
    let secrets_file = include_bytes!("../conf.properties");
    let secrets = java_properties::read(secrets_file.as_slice()).unwrap();
    println!("{secrets:?}");
}
