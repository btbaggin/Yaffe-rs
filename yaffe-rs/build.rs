#![cfg(target_os="linux")] 

//Linux builds require open-ssl, mesa-dev, x11-dev

fn main() {                                                                     
    for lib in &["X11", "Xau", "xcb", "Xdmcp"] {                                
        println!("cargo:rustc-link-lib=static={}", lib);                        
    }                                                                            
}