use std::{
	collections::HashMap,
	env, fs,
	io::{self, BufWriter, Write},
	process,
};

use sqlparser::{
	ast::{Expr, SetExpr, Statement, UnaryOperator, Value},
	dialect,
	parser::Parser,
	tokenizer::Token,
};

const DEFAULT_CAPACITY: usize = 128 * 1024;

fn main() -> io::Result<()> {
	let Some(path) = env::args().nth(1) else {
		println!("Usage: wca-developer-to-sql <path-to-sql-file>");
		process::exit(1);
	};
	let file = fs::File::open(path)?;
	let dialect = dialect::MySqlDialect {};
	let mut parser = Parser::new(&dialect, file);
	let mut files = HashMap::new();
	if fs::read_dir("tables").is_err() {
		fs::create_dir("tables")?;
	}
	loop {
		while let Token::SemiColon = parser.peek_token().token {
			parser.next_token();
		}
		match parser.parse_statement() {
			Ok(query) => {
				if let Statement::CreateTable(create_table) = query {
					let name = &create_table.name.0[0].value;
					files.insert(
						name.to_owned(),
						BufWriter::with_capacity(
							DEFAULT_CAPACITY,
							fs::File::create(format!("tables/{}.tsv", name))?,
						),
					);
					let file = files.get_mut(name).unwrap();
					for col in &create_table.columns {
						write!(file, "{}\t", col.name.value)?;
					}
					writeln!(file)?;
				} else if let Statement::Insert(insert) = query {
					let name = &insert.table_name.0[0].value;
					let file = files.get_mut(name).unwrap();
					if let SetExpr::Values(values) = *insert.source.unwrap().body {
						for row in &values.rows {
							for col in row {
								string_of_col(col, file)?;
							}
							writeln!(file)?;
						}
					}
				}
			}
			Err(e) => {
				if e.to_string().contains("EOF") {
					break;
				} else {
					panic!("{:?}", e);
				}
			}
		}
	}

	Ok(())
}

fn write_escaped(bytes: &[u8], file: &mut impl Write) -> io::Result<()> {
	for &x in bytes {
		match x {
			b'\t' => file.write_all(b"\\t")?,
			b'\n' => file.write_all(b"\\n")?,
			b'\r' => file.write_all(b"\\r")?,
			_ => file.write_all(&[x])?,
		}
	}
	file.write_all(&[b'\t'])
}

fn string_of_col<'a>(expr: &'a Expr, file: &mut impl Write) -> io::Result<()> {
	match expr {
		Expr::Value(value) => match value {
			Value::Number(s, _) => write_escaped(s.as_bytes(), file),
			Value::SingleQuotedString(s) => write_escaped(s.as_bytes(), file),
			Value::Null => write_escaped(b"null", file),
			_ => unreachable!(),
		},
		Expr::UnaryOp {
			op: UnaryOperator::Minus,
			expr: v,
		} => {
			if let Expr::Value(Value::Number(s, _)) = v.as_ref() {
				write_escaped(b"-", file)?;
				write_escaped(s.as_bytes(), file)
			} else {
				unreachable!()
			}
		}
		_ => unreachable!(),
	}
}
