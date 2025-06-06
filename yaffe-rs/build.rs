//Linux builds require open-ssl, mesa-dev, x11-devw libasound2-dev

fn main() {   
    //Link x11 libs                                                                  
    #[cfg(target_os="linux")] 
    for lib in &["X11", "xcb", "Xau", "Xdmcp"] {                                
        println!("cargo:rustc-link-lib=static={}", lib);                        
    }          
    
    use std::io::Write;
    
    //Write version.txt for updating
    const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    let mut file = std::fs::OpenOptions::new().create(true).write(true).open("./version.txt").unwrap();
    file.write_all(CARGO_PKG_VERSION.as_bytes()).unwrap();
}