#!/bin/bash

# Comprehensive fix for simulation module

cd rust-workspace/simulation/src

# Fix all string repeat patterns - more precise regex
sed -i 's|println!("=" \.repeat(\([0-9]*\)));|println!("{}", "=".repeat(\1));|g' *.rs
sed -i 's|println!("-"\.repeat(\([0-9]*\)));|println!("{}", "-".repeat(\1));|g' *.rs

# Fix spaces around operators in println statements
sed -i 's|println!(" =" \.repeat|println!("{}", "=".repeat|g' *.rs
sed -i 's|println!(" -"\.repeat|println!("{}", "-".repeat|g' *.rs

# Fix atmospheric.rs type issues
sed -i 's/let line_width = 0\.1;/let line_width: f64 = 0.1;/' atmospheric.rs

# Fix interactive.rs max issue
sed -i 's|\.max(0\.0)|.max(0.0_f64)|' interactive.rs

echo "Applied comprehensive fixes to simulation files"
