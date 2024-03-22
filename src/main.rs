use std::fs::{self, File}; // 这里self表示“use std::fs”，即将“use std::fs”和“use std::fs::File”合并写为“use std::fs::{self, File};”
use std::io::{self, Read, Write, BufWriter, BufRead, BufReader};
use std::path::{Path, PathBuf};

use argh::FromArgs;
use diff::{lines, Result::{Left, Right, Both}}; // 基于LCS算法计算slice和string的差异
use flate2::read::GzDecoder;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

//----------------------------------------------------------------------------------------------------------------
#[derive(FromArgs)]
/// highlight diff
struct Paras {
	/// old file
	#[argh(option, short = 'a')]
	old: Option<String>,

	/// new file
	#[argh(option, short = 'b')]
	new: Option<String>,

	/// result type, support: terminal(default), txt, html
	#[argh(option, short = 'r')]
	res: Option<String>,

	/// highlight color for left, both, right, support: Black, Blue, Green, Red, Cyan, Magenta, Yellow, White, rgb,x,x,x, default: Red:White:Green
	#[argh(option, short = 'c')]
	color: Option<String>,

	/// line number width, default: 5
	#[argh(option, short = 'w')]
	width: Option<usize>,

	/// batch number, default: 0 (all result)
	#[argh(option, short = 't')]
	batch: Option<usize>,

	/// only show diff result
	#[argh(switch, short = 'd')]
	diff: bool,

	/// outname, default: diff_result
	#[argh(option, short = 'n')]
	outname: Option<String>,

	/// outpath, default: ./
	#[argh(option, short = 'o')]
	outpath: Option<String>,
}

//----------------------------------------------------------------------------------------------------------------
// 存储解析后的命令行参数
#[derive(Debug)]
struct ParsedParas {
	old:     String,             // 旧文件，对应left
	new:     String,             // 新文件，对应right
	res:     Restype,            // 如何展示结果，支持：terminal(默认，在终端打印显示)、txt(保存为txt)、html(保存为html)
	color:   [Option<Color>; 3], // 冒号“:”间隔的3个颜色，指定left颜色、both颜色、right颜色，默认Red,White,Green，支持Black、Blue、Green、Red、Cyan、Magenta、Yellow、White、Rgb(x,x,x)，注意指定rgb颜色时使用“rgb,x,x,x”
	width:   usize,              // 行号宽度，默认5
	batch:   usize,              // 在终端打印时，每次打印多少行结果，0表示一次性全部输出，默认0
	all:     bool,               // 是否输出所有结果，对应!diff，即对-d参数取反
	outname: String,             // 输出文件名，默认diff_result
	outpath: PathBuf,            // 结果输出路径，默认./
}

//----------------------------------------------------------------------------------------------------------------
#[derive(Debug)]
enum Restype {
	/// print in terminal
	Terminal,
	/// save as txt
	Txt,
	/// save as html
	Html,
}

//----------------------------------------------------------------------------------------------------------------
fn main() {
	let parsed_paras = parse_para(); // 解析命令行参数
	let reader_left = my_reader(&parsed_paras.old);
	let reader_right = my_reader(&parsed_paras.new);
	let left = io::read_to_string(reader_left).expect("read -a failed");
	let right = io::read_to_string(reader_right).expect("read -b failed");
	let result = lines(&left, &right); // 对两个文件的行数组进行比较
	let mut left_number: usize = 0; // 记录旧文件的行号
	let mut right_number: usize = 0; // 记录新文件的行号
	let width = parsed_paras.width; // 位数
	match parsed_paras.res {
		Restype::Terminal => { // 终端打印
			let mut n: usize = 0; // 记录打印了多少行
			let mut stdin = io::stdin();
			let mut buf: [u8; 1] = [0];
			for diff in result {
				// “|”前宽度为：width*2+4，例如：width=5，则宽度未5*2+4=14
				// Left显示(默认红色) ：18   -        | xxxxxx
				// Both显示(默认白色) ：18     19     | xxxxxx
				// Right显示(默认绿色)：       19   + | xxxxxx
				match diff {
					Left(l) => {
						left_number += 1;
						let mut stdout = StandardStream::stdout(ColorChoice::Always);
						stdout.set_color(ColorSpec::new().set_fg(parsed_paras.color[0])).expect("error: setting colors in your console failed."); // Cmder显示Red为黄色，使用Rgb(255,0,0)可以显示红色，Windows自带终端没有这个问题
						//stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(255, 0, 0)))).expect("error: setting colors in your console failed.");
						writeln!(&mut stdout, "{:<width$}- {:<width$}  | {}", left_number, "", l).unwrap();
						//println!("{:<width$}- {:<width$}  | {}", left_number, "", l);
					},
					Both(l, _) => {
						left_number += 1;
						right_number += 1;
						if parsed_paras.all { // 指定-d时，不输出没有差异的行
							let mut stdout = StandardStream::stdout(ColorChoice::Always);
							stdout.set_color(ColorSpec::new().set_fg(parsed_paras.color[1])).expect("error: setting colors in your console failed.");
							writeln!(&mut stdout, "{:<width$}  {:<width$}  | {}", left_number, right_number, l).unwrap();
							//println!("{:<width$}  {:<width$}  | {}", left_number, right_number, l);
						}
					},
					Right(r) => {
						right_number += 1;
						let mut stdout = StandardStream::stdout(ColorChoice::Always);
						stdout.set_color(ColorSpec::new().set_fg(parsed_paras.color[2])).expect("error: setting colors in your console failed.");
						writeln!(&mut stdout, "{:<width$}  {:<width$}+ | {}", "", right_number, r).unwrap();
						//println!("{:<width$}  {:<width$}+ | {}", "", right_number, r);
					},
				}
				n += 1;
				if n == parsed_paras.batch { // 等于指定行数则按下回车再继续打印
					// 按下回车继续，参考：https://users.rust-lang.org/t/rusts-equivalent-of-cs-system-pause/4494/3
					//stdin.read(&mut [0u8]).expect("error: failed to read line"); // 如果输入多个字符才回车，则只获取第一个字符，下次循环会直接获取本次输入剩下没获取的字符
					loop {
						stdin.read(&mut buf).expect("error: failed to read line");
						if buf[0] == b'\n' {
							break
						}
					}
					n = 0;
				}
			}
		},
		Restype::Txt => { // 保存为txt
			let mut file_writer = BufWriter::new(File::create(parsed_paras.outpath.join(parsed_paras.outname+".txt").to_str().unwrap()).unwrap()); // 默认buffer大小为8kb
			let mut tmp_content: String;
			for diff in result {
				tmp_content = match diff {
					Left(l) => {
						left_number += 1;
						format!("{:<width$}- {:<width$}  | {}\n", left_number, "", l)
					},
					Both(l, _) => {
						left_number += 1;
						right_number += 1;
						if parsed_paras.all { // 指定-d时，不输出没有差异的行
							format!("{:<width$}  {:<width$}  | {}\n", left_number, right_number, l)
						} else {
							continue
						}
					},
					Right(r) => {
						right_number += 1;
						format!("{:<width$}  {:<width$}+ | {}\n", "", right_number, r)
					},
				};
				file_writer.write(tmp_content.as_bytes()).unwrap(); // 写入文件
			}
		},
		Restype::Html => { // 保存为html，颜色固定：left为#FF6347，both为#696969，right为#3CB371
			let mut file_writer = BufWriter::new(File::create(parsed_paras.outpath.join(parsed_paras.outname+".html").to_str().unwrap()).unwrap()); // 默认buffer大小为8kb
			file_writer.write(b"<!DOCTYPE html>\n<html lang='en'>\n<head>\n    <meta charset='UTF-8'>\n    <meta http-equiv='X-UA-Compatible' content='IE=edge'>\n    <meta name='viewport' content='width=device-width, initial-scale=1.0'>\n    <title>highlight diff</title>\n    <style>\nbody {\n  width: 100%;\n  height: auto;\n  margin: 0px;\n}\n.heading {\n  background-color: #fcd143;\n  margin: 0;\n  padding: 1rem;\n  max-width: 100%;\n  text-align: center;\n  border-bottom-left-radius: 2rem;\n  border-top-right-radius: 2rem;\n  font-family: 'Heebo', sans-serif;\n  margin-block-start: 0rem;\n  margin-block-end: 0rem;\n}\ncode {\n  display: block;\n  text-align: left;\n  white-space: pre-line;\n  position: relative;\n  word-break: normal;\n  word-wrap: normal;\n  line-height: 0.5;\n  background-color: #696969;\n  padding: 15px;\n  margin: 10px;\n  border-radius: 5px;\n  color: #FFFFFF;\n  font-size: 18px;\n}\npre {\n  margin: 0;\n  padding: 0;\n  line-height: 1.2;\n  border: 1px solid rgba(0,0,0,0);\n  border-radius: 5px;\n}\npre:hover {\n  border: 1px solid white;\n}\n.left {\n  background-color: #FF6347;\n}\n.both {\n  background-color: #696969;\n}\n.right {\n  background-color: #3CB371;\n}\n    </style>\n</head>\n<body>\n    <div>\n        <h1 class='heading'>highlight diff</h1>\n    </div>\n    <div>\n        <code>\n").unwrap();
			let mut tmp_content: String;
			for diff in result {
				tmp_content = match diff {
					Left(l) => {
						left_number += 1;
						format!("            <pre class='left'>{:<width$}- {:<width$}  | {}</pre>\n", left_number, "", l.replace("<", "&#60;").replace(">", "&#62;"))
					},
					Both(l, _) => {
						left_number += 1;
						right_number += 1;
						if parsed_paras.all { // 指定-d时，不输出没有差异的行
							format!("            <pre class='both'>{:<width$}  {:<width$}  | {}</pre>\n", left_number, right_number, l.replace("<", "&#60;").replace(">", "&#62;"))
						} else {
							continue
						}
					},
					Right(r) => {
						right_number += 1;
						format!("            <pre class='right'>{:<width$}  {:<width$}+ | {}</pre>\n", "", right_number, r.replace("<", "&#60;").replace(">", "&#62;"))
					},
				};
				file_writer.write(tmp_content.as_bytes()).unwrap(); // 写入文件
			}
			file_writer.write(b"        </code>\n    </div>\n</body>\n</html>\n").unwrap();
		},
	}
}

//----------------------------------------------------------------------------------------------------------------
// 解析参数
fn parse_para() -> ParsedParas {
	let para: Paras = argh::from_env();
	let out: ParsedParas = ParsedParas{
		old: match &para.old { // 旧文件，对应left
			Some(old) => { // 判断该文件是否存在
				let tmp_file = Path::new(&old);
				if tmp_file.exists() && !tmp_file.is_dir() { // 指定文件存在
					old.to_string()
				} else {
					panic!("error: -a file not exists: {}", old);
				}
			},
			None => panic!("error: must use -a specify old file"), // 不存在则报错
		},
		new: match &para.new { // 新文件，对应right
			Some(new) => { // 判断该文件是否存在
				let tmp_file = Path::new(&new);
				if tmp_file.exists() && !tmp_file.is_dir() { // 指定文件存在
					new.to_string()
				} else {
					panic!("error: -b file not exists: {}", new);
				}
			},
			None => panic!("error: must use -b specify new file"), // 不存在则报错
		},
		res: match para.res { // 如何展示结果，支持：terminal(默认，在终端打印显示)、txt(保存为txt)、html(保存为html)
			Some(t) => {
				match t.as_str() {
					"terminal" => Restype::Terminal, // 终端打印
					"txt" => Restype::Txt, // 保存为txt
					"html" => Restype::Html, // 保存为html
					other => panic!("error: -r only support terminal, txt, html, not {}", other),
				}
			},
			None => Restype::Terminal, // 默认终端打印
		},
		color: match para.color { // 冒号“:”间隔的3个颜色，指定left颜色、both颜色、right颜色，默认Red,White,Green，支持Black、Blue、Green、Red、Cyan、Magenta、Yellow、White、Rgb(x,x,x)，注意指定rgb颜色时使用“rgb,x,x,x”
			Some(c) => {
				let color: Vec<String> = c.split(':').map(|s| s.to_string()).collect();
				if color.len() == 3 {
					let mut tmp_color: [Option<Color>; 3] = [Some(Color::Red), Some(Color::White), Some(Color::Green)];
					for (i, c) in color.iter().enumerate() {
						tmp_color[i] = match color[i].as_str() {
							"Black" => Some(Color::Black),
							"Blue" => Some(Color::Blue),
							"Green" => Some(Color::Green),
							"Red" => Some(Color::Red),
							"Cyan" => Some(Color::Cyan),
							"Magenta" => Some(Color::Magenta),
							"Yellow" => Some(Color::Yellow),
							"White" => Some(Color::White),
							rgb if rgb.starts_with("rgb,")  => { // 匹配“rgb,x,x,x”
								let v: Vec<u8> = rgb.split(",").filter_map(|s| s.parse::<u8>().ok()).collect(); // 提取r、g、b字符串值，并转为u8
								if v.len() == 3 {
									Some(Color::Rgb(v[0], v[1], v[2]))
								} else {
									panic!("error: cannot parse rgb,x,x,x: {}", rgb);
								}
							},
							_ => panic!("error: only support Black, Blue, Green, Red, Cyan, Magenta, Yellow, White, rgb,x,x,x, not this color: {}", c),
						};
					}
					tmp_color
				} else {
					panic!("error: -c {}", c);
				}
			},
			None => [Some(Color::Red), Some(Color::White), Some(Color::Green)]
		},
		width: match para.width { // 行号宽度，默认5
			Some(w) => w,
			None => 5_usize,
		},
		batch: match para.batch { // 在终端打印时，每次打印多少行结果，0表示一次性全部输出，默认0
			Some(t) => {
				if t == 0 {
					usize::MAX
				} else {
					t
				}
			},
			None => usize::MAX,
		},
		all: !para.diff, // 是否输出所有结果，对应!diff，即对-d参数取反
		outname: match &para.outname { // 输出文件名，默认diff_result
			Some(n) => n.to_string(),
			None => "diff_result".to_string(),
		},
		outpath: match &para.outpath { // 结果输出路径，默认./
			Some(outpath) => {
				let tmp_outpath = Path::new(&outpath);
				if !(tmp_outpath.exists() && tmp_outpath.is_dir()) { // exists检查是否存在，is_dir检查是否是路径
					match fs::create_dir_all(&tmp_outpath) { // 不存在则创建
						Ok(_) => (),
						Err(err) => {
							panic!("create output dir error: {}", err);
						},
					}
				}
				tmp_outpath.to_path_buf()
			},
			None => Path::new("./").to_path_buf(),
		},
	};
	out
}

//----------------------------------------------------------------------------------------------------------------
// 读取gz压缩或未压缩文件，这样可以返回reader
// 参考：https://users.rust-lang.org/t/write-to-normal-or-gzip-file-transparently/35561/2
// 参考：https://github.com/rust-lang/flate2-rs/issues/393
fn my_reader(filename: &str) -> Box<dyn BufRead> {
	let path = Path::new(filename);
	let file = match File::open(&path) {
		Ok(file) => file,
		Err(why) => panic!("couldn't open {}: {}", path.display(), why),
	};
	if path.extension().unwrap() == "gz" {
		//Box::new(BufReader::with_capacity(8 * 1024, GzDecoder::new(file)))
		Box::new(BufReader::new(GzDecoder::new(file)))
	} else {
		//Box::new(BufReader::with_capacity(8 * 1024, file))
		Box::new(BufReader::new(file))
	}
}
