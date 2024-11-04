use std::{collections::HashMap, env, fs, io::{Write, self, BufWriter}, process};

use sqlparser::{ast::{Expr, SetExpr, Statement, UnaryOperator, Value}, dialect, parser::Parser, tokenizer::Token};

fn main() -> io::Result<()> {
	let Some(path) = env::args().nth(1) else {
		println!("Usage: wca-developer-to-sql <path-to-sql-file>");
		process::exit(1);
	};
	let file  = fs::File::open(path)?;
	let dialect = dialect::MySqlDialect {};
	let mut parser = Parser::new(&dialect, file);
	let mut files = HashMap::new();
	if std::fs::read_dir("tables").is_err() {
		std::fs::create_dir("tables")?;
	}
	loop {
		while let Token::SemiColon = parser.peek_token().token {
			parser.next_token();
		}
		match parser.parse_statement() {
			Ok(query) => {
				if let Statement::CreateTable(create_table) = query {
					let name = &create_table.name.0[0].value;
					files.insert(name.to_owned(), BufWriter::new(fs::File::create(format!("tables/{}.tsv", name))?));
					let file = files.get_mut(name).unwrap();
					for col in &create_table.columns {
						let _ = write!(file, "{}\t", col.name.value);
					}
					let _ = writeln!(file);
				}
				else if let Statement::Insert(insert) = query {
					let name = &insert.table_name.0[0].value;
					let file = files.get_mut(name).unwrap();
					if let SetExpr::Values(values) = *insert.source.unwrap().body {
						for row in &values.rows {
							for col in row {
								let _ = write!(
									file,
									"{}\t",
									string_of_col(col)
										.replace("\n", "\\n")
										.replace("\t", "\\t")
										.replace("\r", "\\r")
								);
							}
							let _ = writeln!(file);
						}
					}
				}
			}
			Err(e) => {
				if e.to_string().contains("EOF") {
					break;
				}
				else {
					panic!("{:?}", e);
				}
			}
		}
	}

	Ok(())
}

fn string_of_col<'a>(expr: &'a Expr) -> String {
	match expr {
		Expr::Value(value) => match value {
			Value::Number(s, _) => s.to_owned(),
			Value::SingleQuotedString(s) => s.to_owned(),
			Value::Null => "null".to_owned(),
			_ => unreachable!(),
		},
		Expr::UnaryOp {
			op: UnaryOperator::Minus,
			expr: v,
		} => {
			if let Expr::Value(Value::Number(s, _)) = v.as_ref() {
				format!("-{}", s)
			} else {
				unreachable!()
			}
		}
		_ => unreachable!(),
	}
}
