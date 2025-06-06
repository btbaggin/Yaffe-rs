// This needs to match yaffe
const UPDATE_FILE_PATH: &str = "./yaffe-rs.update";

fn main() {
    println!("Applying patch file");
    if std::path::Path::new(UPDATE_FILE_PATH).exists() {
        let args: Vec<String> = std::env::args().collect();

        if let Err(e) = std::fs::rename(UPDATE_FILE_PATH, args[1].clone()) {
            println!("{:?}", e);
        }

        if let Err(e) = std::process::Command::new(args[1].clone()).spawn() {
            println!("{:?}", e);
        }
    }
}
