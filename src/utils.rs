use md5::Context;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn calculate_md5(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut context = Context::new();
    let mut buffer = [0; 4096];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        context.consume(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", context.compute()))
}
