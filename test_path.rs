use std::path::Path;

fn main() {
    let file_path = "fixtures/lua/demo.lua";
    if let Some(parent) = Path::new(file_path).parent() {
        println!("Parent: {:?}", parent);
        println!("Display: {}", parent.display());
    }
}
