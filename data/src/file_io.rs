use std::fs::File;
use std::io::Read;
pub fn read_json_file_to_str(filename: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(filename).expect("File not found.");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Error reading file contents to string.");
    Ok(data)
}
