use std::{fs::File, io::Read};

use twilight_http::Client;

fn get_command_file() -> std::io::Result<File> {
    crate::file::data_file("commands.properties")
}

pub fn init_commands(client: &Client) {
    let mut buffer = String::new();
    get_command_file()
        .expect("Could not get command file")
        .read_to_string(&mut buffer)
        .expect("Could not read command data");
    println!("{buffer}");
}
