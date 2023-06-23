fn main() {
    let conf_file = include_str!("conf.env");
    conf_file
        .split('\n')
        .map(|part| part.trim())
        .filter(|line| !line.starts_with('#'))
        .for_each(|env| println!("cargo:rustc-env={env}"))
}
