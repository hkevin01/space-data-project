#!/bin/bash

# Fix all the string repeat syntax issues in simulation files

# Fix main.rs
sed -i 's/println!("=" \.repeat(\([0-9]*\)));/println!("{}", "=".repeat(\1));/g' rust-workspace/simulation/src/main.rs
sed -i 's/println!("-"\.repeat(\([0-9]*\)));/println!("{}", "-".repeat(\1));/g' rust-workspace/simulation/src/main.rs

# Fix interactive.rs
sed -i 's/println!("=" \.repeat(\([0-9]*\)));/println!("{}", "=".repeat(\1));/g' rust-workspace/simulation/src/interactive.rs
sed -i 's/println!("-"\.repeat(\([0-9]*\)));/println!("{}", "-".repeat(\1));/g' rust-workspace/simulation/src/interactive.rs

echo "Fixed string repeat syntax in simulation files"
