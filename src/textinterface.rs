use std::rc::Rc;
use std::cell::RefCell;
use chrono::naive::NaiveDate;
use chrono::prelude::*;
use std::io::{self, prelude::*, Stderr, Stdin, Stdout};
use std::str::FromStr;
use std::process;
use std::fmt;
use std::error::Error;

use crate::department::Department;
use crate::personnel::{Person, Name};
use crate::data_handling::ProgramData;

pub type Result<T> = std::result::Result<T, TextInterfaceError>;

struct Command {
    keyword: String,
    short_desc: String,
    long_desc: String,
    operation: fn(&mut TextInterface, std::str::SplitWhitespace) -> Result<()>,
}

pub struct TextInterface {
    io: TextIO,
    data: ProgramData,
    commands: Vec<Command>,
}

impl TextInterface {
    pub fn init() -> Self {
        let mut commands: Vec<Command> = Vec::new();

        commands.push(Command {
            keyword: String::from("help"),
            short_desc: String::from("Print this list.  Use \"help [COMMAND]\" for details on a command."),
            long_desc: String::from("Coming soon!"),
            operation: TextInterface::help,
        });

        commands.push(Command {
            keyword: String::from("new"),
            short_desc: String::from("Add a new employee or department entry."),
            long_desc: String::from("NEW [EMPLOYEE|DEPARTMENT]\n\n\
Ex:  NEW EMPLOYEE\n     NEW DEPARTMENT"),
            operation: TextInterface::new,
        });

        commands.push(Command {
            keyword: String::from("quit"),
            short_desc: String::from("Exit the program."),
            long_desc: String::from("QUIT\n\n\
            Exit out of this program when you no longer want to use the program.  Why\n\
            you would want to do that could be for one or more of several reasons. A\n\
            few possibilities include:\n\
            - Tired of using the program.\n\
            - Program doesn't work.\n\
            - Don't know how to use program.\n\
            - All my friends use the other program.\n\
            - Don't like the guy who wrote the program.\n\
            - Not into hip hop.\n\
            - Time to go to bed.\n\
            - Time to go to work.\n\
            - Break time.\n\
            - Need to use restroom.\n\
            - Erection lasting longer than four hours."),
            operation: TextInterface::quit,
        });

        commands.push(Command {
            keyword: String::from("list"),
            short_desc: String::from("Print a list of departments or employees"),
            long_desc: String::from("LIST [DEPARTMENTS|EMPLOYEES]\n\n\
            Prints a list of departments or employees, in alphamabetical order."),
            operation: TextInterface::list,
        });

        TextInterface {
            io: TextIO {
                stdin: io::stdin(),
                stdout: io::stdout(),
                stderr: io::stderr(),
            },
            data: ProgramData::init(),
            commands,
        }
    }

    /// NYI
    pub fn run(&mut self) -> Result<()> {
        let mut io_buff = String::new();
        loop {
            self.io.stdin.read_line(&mut io_buff)?;

            let mut command = io_buff.split_whitespace();

            match command.next() {
                Some(word) => {
                    let comm = word.to_lowercase();

                    let mut op: Option<fn(&mut TextInterface, std::str::SplitWhitespace) -> Result<()>> = None;
                    for item in &self.commands {
                        if item.keyword == comm {
                            op = Some(item.operation);
                        }
                    }

                    if op.is_some() {
                        (op.unwrap())(self, command)?;
                    } else {
                        println!("Type HELP for a list of commands.");
                    }
                },
                None => println!("Type HELP for a list of commands."),
            };

            io_buff.clear();
        }
    }



    fn help(&mut self, mut args: std::str::SplitWhitespace) -> Result<()> {
        match args.next() {
            None => {
                println!("Type HELP [COMMAND] for more information.");
                println!();

                for comm in &self.commands {
                    println!("{}:  {}", comm.keyword, comm.short_desc);
                }
            },
            Some(arg) => {
                for comm in &self.commands {
                    if comm.keyword == arg {
                        self.io.stdout.write(comm.long_desc.as_bytes())?;
                        self.io.stdout.write(&[b'\n'])?;
                        self.io.stdout.flush()?;
                        return Ok(());
                    }
                }
                println!("Command not found: {}", arg);
            },
        };
        Ok(())
    }

    fn quit(&mut self, mut args: std::str::SplitWhitespace) -> Result<()> {
        println!("\nSo long, sucker!");
        process::exit(0);
    }

    fn list(&mut self, mut args: std::str::SplitWhitespace) -> Result<()> {
        match args.next() {
            Some(what) => {
                let what = what.to_lowercase();

                if what == "employees" {
                    if let Err(e) = self.list_employees(args) {
                        eprintln!("Error printing list: {}", e);
                    }
                }
            },
            None => Self::short_help(),
        };

        Ok(())
    }

    fn list_employees(&mut self, mut args: std::str::SplitWhitespace) -> Result<()> {
        match args.next() {
            Some(whatnow) => {

            },
            None => {
                let all_sorted = self.sort_employees();

                for (alias, name) in all_sorted {
                    println!("\"{}\": {}", alias, name);
                }
            },
        }
        Ok(())
    }

    fn sort_employees(&self) -> Vec<(String, Name)> {
        let mut list: Vec<(String, Name)> = Vec::new();

        for employee in self.data.employee_list() {
            let name = employee.pointer().borrow().name().clone();

            let search_result = list.binary_search_by(|(_, entry)| (*entry).cmp(&name));

            if let Err(index) = search_result {
                list.insert(index, (employee.alias().clone(), name));
            }
        }

        list
    }

    fn new(&mut self, mut args: std::str::SplitWhitespace) -> Result<()> {
        match args.next() {
            Some(thing) => {
                let thing = thing.to_lowercase();

                if thing == "employee" {
                    if self.data.dept_list().len() == 0 {
                        println!("Cannot add employee: No departments found.");
                    } else {
                        if let Err(e) = self.add_employee() {
                            eprintln!("Could not add employee: {}", e);
                        }
                    }
                } else if thing == "department" {
                    if let Err(e) = self.add_department() {
                        eprintln!("Could not add department: {}", e);
                    }
                } else {
                    Self::short_help();
                }
            },
            None => { Self::short_help(); },
        };
        Ok(())
    }

    fn add_employee(&mut self) -> Result<()> {
        if self.data.dept_list().len() == 0 {
            return Err(TextInterfaceError::NoDepartment);
        }

        let mut alias: Option<String> = None;
        let mut name_last: Option<String> = None;
        let mut name_first: Option<String> = None;
        let mut name_mid: Option<String> = None;
        let mut doh: Option<NaiveDate> = None;
        let mut department: Option<Rc<RefCell<Department>>> = None;

        let none = String::from("None");
        let today = Local::today().naive_local();

        let mut io_buffer = String::new();

        loop {
            println!("1: Alias*:       {}", match &alias {
                Some(name) => &name,
                None => &none,
            });
            println!("2: First Name*:  {}", match &name_first {
                Some(name) => &name,
                None => &none,
            });
            println!("3: Middle Name:  {}", match &name_mid {
                Some(name) => &name,
                None => &none,
            });
            println!("4: Last Name*:   {}", match &name_last {
                Some(name) => &name,
                None => &none,
            });
            println!("5: Date of Hire: {}", match &doh {
                Some(date) => format_date_us(date),
                None => format_date_us(&today),
            });
            println!("6: Department*:  {}", match &department {
                Some(dept) => dept.borrow().name().clone(),
                None => (&none).clone(),
            });

            println!();

            println!("Enter a line number to modify, or \"commit\" to finish.");
            self.io.stdout.write(b"?> ")?;
            self.io.stdout.flush()?;

            io_buffer.clear();

            self.io.stdin.read_line(&mut io_buffer)?;

            let mut option = 0u32;

//            let mut get_string = |prnt: &str| {
//                self.io.stdout.write(format!("Enter {}: ", prnt).as_bytes());
//                self.io.stdout.flush();
//                io_buffer.clear();
//                self.io.stdin.read_line(&mut io_buffer);
//                io_buffer.clone()
//            };

            if io_buffer.trim() == "commit" {
                if name_first.is_none() || name_last.is_none() || department.is_none() || alias.is_none() {
                    println!("Required fields missing");
                    continue;
                } else {
                    let mut person = Person::builder();

                    let first_name_clone = name_first.clone().unwrap();
                    let last_name_clone = name_last.clone().unwrap();
                    let dept_clone = Rc::clone(&department.clone().unwrap());

                    person.first_name(&first_name_clone)
                        .last_name(&last_name_clone)
                        .department(dept_clone);

                    if let Some(date) = doh {
                        person.date_of_hire(date);
                    } else {
                        person.date_of_hire(today);
                    }

                    if let Some(name) = &name_mid {
                        person.middle_name(name);
                    }

                    let person = person.build();
                    let mut person_final: Person;
                    if let Ok(val) = person {
                        person_final = val;
                    } else {
                        panic!("What went wrong??? (textinterface.rs, add_person(), trouble finalizing");
                    }

                    let p_alias = alias.clone().unwrap_or(person_final.first_name().clone());

                    if self.data.add_person(&p_alias, person_final).is_err() {
                        eprintln!("Error on add.  Review fields and try again.");
                        continue;
                    }

                    return Ok(());
                }
            } else {
                option = match u32::from_str(&io_buffer.trim()) {
                    Err(_) => {
                        println!("Invalid input");
                        continue;
                    },
                    Ok(num) => num,
                };

                match option {
                    1 => {
                        alias = Some(get_string("alias", &mut self.io));
                    }
                    2 => {
                        name_first = Some(get_string("first name", &mut self.io));
                    },
                    3 => {
                        let entry = get_string("middle name", &mut self.io);
                        if entry.len() > 0 {
                            name_mid = Some(entry);
                        } else {
                            name_mid = None;
                        }
                    },
                    4 => {
                        name_last = Some(get_string("last name", &mut self.io));
                    },
                    5 => {
                        let doh_string = get_string("date of hire(MM/DD/YYYY)", &mut self.io);
                        if doh_string.len() == 0 {
                            doh = None;
                            continue;
                        }

                        match parse_date_us(&doh_string) {
                            Ok(date) => doh = Some(date),
                            Err(_) => {
                                println!("Invalid date format");
                                continue;
                            },
                        };
                    },
                    6 => {
                        let dept_string = get_string("initial department", &mut self.io);

                        for value in self.data.dept_list() {
                            if value.alias() == &dept_string {
                                department = Some(value.clone_pointer());
                                break;
                            }
                        }
                    },
                    _ => {
                        println!("Invalid input");
                        continue;
                    },
                };
            }
        }
        Ok(())
    }

    fn select_department(&mut self) -> Rc<RefCell<Department>> {
        for (index, department) in self.data.dept_list().iter().enumerate() {
            println!("{}: {}", index + 1, department.alias());
        }

        let mut choice = String::new();
        loop {
            self.io.stdout.write(b"Pick a department: ")
                .expect("IO ERROR");
            self.io.stdout.flush()
                .expect("IO ERROR");

            self.io.stdin.read_line(&mut choice)
                .expect("IO ERROR");

            let dept = self.data.departments().get(&choice);

            if let None = dept {
                choice.clear();
                continue;
            } else {
                return Rc::clone(dept.unwrap());
            }
        }
    }

    fn add_department(&mut self) -> Result<()> {
        let mut department_alias: Option<String> = None;
        let mut department_name: Option<String> = None;

        let none = String::from("none");

        let mut io_buffer = String::new();

        loop {
            io_buffer.clear();

            println!("1: Unique identifier: {}", match &department_alias {
                Some(id) => id,
                None => &none,
            });
            println!("2: Full name:         {}", match &department_name {
                Some(name) => name,
                None => &none,
            });

            println!();

            println!("Enter a line number to modify, or \"commit\" to finish.");
            self.io.stdout.write(b"?> ")?;
            self.io.stdout.flush()?;
            self.io.stdin.read_line(&mut io_buffer)?;

            let input = io_buffer.trim();

            if input == "commit" {
                if department_alias.is_none() || department_name.is_none() {
                    println!("Required fields missing.");
                    continue;
                } else {
                    if self.data.add_dept(&department_alias.clone().unwrap(), &department_name.clone().unwrap())
                        .is_err() {
                        eprintln!("Error adding department, check fields and try again.");
                        continue;
                    } else {
                        break;
                    }
                }
            }

            let option = match u32::from_str(input) {
                Ok(num) => num,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    0
                },
            };

            match option {
                1 => {
                    department_alias = Some(get_string("identifier", &mut self.io));
                },
                2 => {
                    department_name = Some(get_string("department name", &mut self.io));
                },
                _ => {
                    println!("Invalid selection.");
                    continue;
                }
            }
        }
        Ok(())
    }

    fn short_help() {
        println!("Type HELP [COMMAND] for more information.");
        println!();
    }
}

fn get_string(prnt: &str, io: &mut TextIO) -> String {
    let mut io_buffer = String::new();

    io.stdout.write(format!("Enter {}: ", prnt).as_bytes()).expect("IO ERROR");
    io.stdout.flush().expect("IO ERROR");
    io.stdin.read_line(&mut io_buffer).expect("IO ERROR");

    String::from(io_buffer.trim())
}

fn format_date_us(date: &NaiveDate) -> String {
    let date_format = date.format("%m/%d/%Y");
    format!("{}", date_format)
}

fn parse_date_us(date_string: &str) -> Result<NaiveDate> {
    const ERR: TextInterfaceError = TextInterfaceError::InvalidDate;

    let values: Vec<&str> = date_string.split('/').collect();

    //let err_value = TextInterfaceError::InvalidDate;

    if values.len() != 3 {
        return Err(ERR);
    }

    let month = u32::from_str(values[0]).map_err(|_| ERR)?;
    let day = u32::from_str(values[1]).map_err(|_| ERR)?;
    let year = i32::from_str(values[2]).map_err(|_| ERR)?;

    NaiveDate::from_ymd_opt(year, month, day).ok_or(ERR)
}

struct TextIO {
    stdin: Stdin,
    stdout: Stdout,
    stderr: Stderr,
}

#[derive(Debug)]
pub enum TextInterfaceError {
    InvalidDate,
    NoDepartment,
    IOError(io::Error),
}

impl Error for TextInterfaceError {}

impl fmt::Display for TextInterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TextInterfaceError::*;

        match self {
            InvalidDate => write!(f, "Invalid Date"),
            NoDepartment => write!(f, "No Department"),
            IOError(e) => write!(f, "IO Error({})", e),
        }
    }
}

impl From<io::Error> for TextInterfaceError {
    fn from(e: io::Error) -> Self {
        TextInterfaceError::IOError(e)
    }
}