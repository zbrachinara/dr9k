#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let secrets_file = include_bytes!("../conf.properties");
    let secrets = java_properties::read(secrets_file.as_slice()).unwrap();
    println!("{secrets:?}");
}
