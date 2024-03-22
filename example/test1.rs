use argh::FromArgs;
use flate2::read::GzDecoder;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Restype {
	/// print in terminal
	Terminal,
	/// save as txt
	Txt,
	/// save as html
	Html,
}

fn main() {
	let left = fs::read_to_string(&parsed_paras.old).expect("read -a failed");
	let right = fs::read_to_string(&parsed_paras.new).expect("read -b failed");
	let result = lines(left, right);
	let mut left_number: usize = 0;
	let mut right_number: usize = 0;
	let width = parsed_paras.width;
}
