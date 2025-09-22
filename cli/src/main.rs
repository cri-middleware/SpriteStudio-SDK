use std::env;
use std::path::Path;
use ssconverter3;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("usage: ssconverter3 <sspj path>");
        return;
    }
    let path = Path::new(args[1].as_str());
    let Ok(sspj) = ssconverter3::sspj(&path) else {
        println!("not found: {}", path.as_os_str().to_string_lossy());
        return;
    };
    println!("{:?}", sspj);
}
