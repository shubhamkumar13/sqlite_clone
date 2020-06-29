// use termion::input::TermRead;
// use termion::event::Key;
// use termion::clear;
use std::{process, fmt};
use serde::ser::{Serialize, Serializer, SerializeStruct};
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use std::mem::{size_of, size_of_val};

enum MetaCommand {
    MetaCommandSuccess,
    MetaCommandUnrecognizedCommand,
}

enum PrepareResult {
    PrepareSuccess,
    PrepareSyntaxError,
    PrepareUnreconizedStatement,
}

enum StatementType {
    StatementInsert,
    StatementSelect,
}

#[derive(Clone, Debug)]
struct Row {
    id : Option<u64>,
    uname : Option<String>,
    email : Option<String>,
}

impl Serialize for Row {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
            let mut state = serializer.serialize_struct("Row", 3)?;
            state.serialize_field("id", &self.id)?;
            state.serialize_field("uname", &self.uname)?;
            state.serialize_field("email", &self.email)?;
            state.end()
        }
}

impl Row {
    fn new(id: u64, uname: String, email: String) -> Self {
        Row{id:Some(id), uname:Some(uname), email:Some(email)}
    }

    fn is_empty(&self) -> bool {
        match (self.id, self.uname, self.email) {
            (None, None, None) => true,
            (_,_,_) => false,
        }
    }

    fn plus(&self, a : u64) -> u64 {
        let x = size_of_val(&self);
        (x as u64) + a
    }
}

impl<'de> Deserialize<'de> for Row {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field { Id, Uname, Email };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`id or `uname` or `email`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "Id" => Ok(Field::Id),
                            "Uname" => Ok(Field::Uname),
                            "Email" => Ok(Field::Email),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct RowVisitor;

        impl<'de> Visitor<'de> for RowVisitor {
            type Value = Row;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Row")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Row, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let id = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let uname = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let email = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(Row::new(id, uname, email))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Row, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut uname = None;
                let mut email = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Uname => {
                            if uname.is_some() {
                                return Err(de::Error::duplicate_field("uname"));
                            }
                            uname = Some(map.next_value()?);
                        }
                        Field::Email => {
                            if email.is_some() {
                                return Err(de::Error::duplicate_field("email"));
                            }
                            email = Some(map.next_value()?);
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let uname = uname.ok_or_else(|| de::Error::missing_field("uname"))?;
                let email = email.ok_or_else(|| de::Error::missing_field("email"))?;
                Ok(Row::new(id, uname, email))
            }
        }

        const FIELDS: &'static [&'static str] = &["id", "uname", "email"];
        deserializer.deserialize_struct("Row", FIELDS, RowVisitor)
    }
}


struct Statement {
    stype: Option<StatementType>,
    row_to_insert : Option<Row>,
}

struct Table {
    num_rows : Option<u64>,
    pages : Option<Vec<*const Row>>,
}

impl Table {
    fn new(&self) -> 
}

fn print_row(row : Row) -> () {
    println!("({:?}, {:?}, {:?})", row.id, row.uname, row.email);
}

const ROW_SIZE : usize = size_of::<Row>();
const PAGE_SIZE : usize = 4096;
const TABLE_MAX_PAGES : usize = 100;
const ROWS_PER_PAGE : usize = PAGE_SIZE/ROW_SIZE;
const TABLE_MAX_ROWS : usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

fn row_slot(table : Table, row_num: u64) -> u64 {
    let page_num: u64 = row_num / (ROWS_PER_PAGE as u64);
    let page : *const Row = table.pages.into_iter()
                    .clone()
                    .collect::<Vec<_>>()
                    .remove(page_num as usize);
    let row_offset : u64 = row_num % (ROWS_PER_PAGE as u64);
    let byte_offset : u64 = row_offset * (ROW_SIZE as u64);
    unsafe {
        page.read().plus(byte_offset)
    }
}

fn main() {
    let mut rl = rustyline::Editor::<()>::new();
    loop {
        let readline = rl.readline("sqlite> ");

        match readline {
            Ok(line) => {
                match line.clone().remove(0) {
                    '.' => {
                            match do_meta_command(line.clone()) {
                                MetaCommand::MetaCommandSuccess => {
                                    println!("{}", line);
                                    continue;
                                },
                                MetaCommand::MetaCommandUnrecognizedCommand => {
                                    println!("Unrecognized Command {}", line);
                                    continue;
                                },
                            }
                        },
                    _ => {
                        let mut statement = Statement {
                            stype : None,
                            row_to_insert : None,
                        };
                        match prepare_statement(line.clone(), &mut statement) {
                            PrepareResult::PrepareSuccess => {},
                            PrepareResult::PrepareSyntaxError => {
                                println!("Syntax error. Could not parse statement.");
                            }
                            PrepareResult::PrepareUnreconizedStatement => {
                                println!("Unrecognized keyword at start of {}", line.clone());
                            },
                        }

                        execute_statement(&mut statement);
                        println!("Executed");
                    },
                }
            },
            Err(_) => println!("Something went wrong, and by something I mean the input"),
        }
    }
}

fn do_meta_command(line: String) -> MetaCommand {
    match line.clone().as_str() {
        ".exit" | ".quit" => process::exit(0),
        _ => MetaCommand::MetaCommandUnrecognizedCommand,
    }
}

fn prepare_statement(line: String, statement : &mut Statement) -> PrepareResult {
    let line_t = line.clone();
    let tokens = line_t.split_whitespace().collect::<Vec<&str>>();

    match tokens.first() {
        Some(x) => {
            match *x {
                "insert" => {
                    statement.stype = Some(StatementType::StatementInsert); 
                    let mut tokens_clone = tokens.iter();

                    match tokens_clone.clone().size_hint() {
                        (x,_) => {
                                if x < 3 {
                                PrepareResult::PrepareSyntaxError
                                } else {
                                    let _ = tokens_clone.next();
                                    let id = tokens_clone.nth(1).unwrap().parse::<u64>().unwrap();
                                    let uname = (*tokens_clone.nth(2).unwrap()).to_string();
                                    let email = (*tokens_clone.nth(3).unwrap()).to_string();
                                    statement.row_to_insert = Some(Row::new(id, uname, email));
                                    PrepareResult::PrepareSuccess

                                }
                            },
                        _ => {
                            PrepareResult::PrepareUnreconizedStatement
                        },
                    }

                },
                "select" => {
                    statement.stype = Some(StatementType::StatementSelect);
                    PrepareResult::PrepareSuccess
                }
                _ => {
                    PrepareResult::PrepareUnreconizedStatement
                },
            }
        },
        None => {
            println!("Something went wrong, and by something I mean the line contains the command");
            PrepareResult::PrepareUnreconizedStatement
        }
    }
}

fn execute_statement(statement: &mut Statement) -> () {
    match statement.stype {
        Some(StatementType::StatementInsert) => {
            println!("This is where we would do an insert");
        }
        Some(StatementType::StatementSelect) => {
            println!("This is where we would do a select");
        }
        _ => {}
    }
}