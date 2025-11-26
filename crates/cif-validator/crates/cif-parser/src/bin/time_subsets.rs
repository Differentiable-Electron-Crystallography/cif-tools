use std::time::Instant;
fn main() {
    for (name, path) in [
        ("small (1K lines)", "/tmp/small.dic"),
        ("medium (5K lines)", "/tmp/medium.dic"),
        ("large (15K lines)", "/tmp/large.dic"),
    ] {
        let content = std::fs::read_to_string(path).unwrap();
        let start = Instant::now();
        let _ = cif_parser::CifDocument::parse(&content);
        println!("{}: {:?}", name, start.elapsed());
    }
}
