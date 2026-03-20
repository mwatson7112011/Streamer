use winres::WindowsResource;

fn main() {
    // Only run this if we are compiling for Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "windows".to_string()) == "windows" {
        let mut res = WindowsResource::new();
        
        // Matches your new filename exactly
        res.set_icon("Streamer.ico"); 
        
        res.set("InternalName", "Streamer.exe");
        res.set("ProductName", "Streamer");
        res.set("FileDescription", "Unified Streaming Hub");
        
        if let Err(e) = res.compile() {
            eprintln!("Error compiling resources: {}", e);
            std::process::exit(1);
        }
    }
}