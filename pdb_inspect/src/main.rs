use std::fs::File;
use pdb::{PDB, FallibleIterator};

fn main() {
    let file = File::open("target/release/rust_app_example.pdb").expect("open pdb");
    let mut pdb = PDB::open(file).expect("parse pdb");
    let symbol_table = pdb.global_symbols().expect("global symbols");
    let mut count = 0;
    let mut iter = symbol_table.iter();
    while let Ok(Some(symbol)) = iter.next() {
        use pdb::SymbolData;
        if let Ok(data) = symbol.parse() {
            if let SymbolData::Public(public) = data {
                let name = public.name.to_string();
                if name.contains("main") || name.contains("insert") || name.contains("delete") || name.contains("rotate") {
                    println!("{}", name);
                    count += 1;
                }
            }
        }
    }
    println!("Done, found {} symbols", count);
}
