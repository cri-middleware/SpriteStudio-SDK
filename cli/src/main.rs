use std::env;
use std::path::Path;
use ssconverter3;

fn main() {
    let args: Vec<String> = env::args().collect();
    for arg in args {
        println!("{:?}", arg)
    }
    println!("{}", ssconverter3::add(1, 2));
    let path = Path::new("tests/overall/common.ssce");
    let Ok(ssce) = ssconverter3::ssce(&path) else {
        println!("not found: {}", path.as_os_str().to_string_lossy());
        return;
    };
    println!("{:?}", ssce);
}
