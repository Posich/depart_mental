use crate::personnel::{ Person, PersonError };
use crate::department::{ Department, DeptErr };

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell};
use std::fmt;
use std::error::Error;
use std::ops::Deref;

use chrono::naive::NaiveDate;
use chrono::prelude::*;
use std::cmp::Ordering;

pub type Result<T> = std::result::Result<T, DataError>;

/// ProgramData and its related methods represent the main API for managing personnel and department
/// data.
pub struct ProgramData {
    dept_aliases:     Vec<DepartmentAlias>,
    person_aliases:   Vec<PersonAlias>,
    departments:      HashMap<String, Rc<RefCell<Department>>>,
    personnel:        HashMap<String, Rc<RefCell<Person>>>,
    employee_count:   u32,
    department_count: u32,
}

// TODO -- Impl Serde and SQLite functionality to store and retrieve data from filesystem.
impl ProgramData {

    /// initialize an empty container struct for program data.  All fields will be empty.  In
    /// the future, it will be possible to populate this struct with saved data from previous
    /// sessions.
    pub fn init() -> Self {
        ProgramData {
            dept_aliases:     Vec::new(),
            person_aliases:   Vec::new(),
            departments:      HashMap::new(),
            personnel:        HashMap::new(),
            employee_count:   0,
            department_count: 0,
        }
    }

    /// Add a new department and store it in memory.  This method, when supplied with strings
    /// for an alias, and full name of the department, will create the department on its own.
    pub fn add_dept(&mut self, alias: &str, dept_name: &str) -> Result<Rc<RefCell<Department>>> {
        if self.departments.contains_key(alias) {
            return Err(DataError::AddDept);
        }

        self.department_count += 1;

        let department_id = self.department_count;

        let new_department = Rc::new(
            RefCell::new(
                Department::new(dept_name, department_id)
            )
        );

        let dept_alias = DepartmentAlias::new(alias, Rc::clone(&new_department));

        //self.dept_aliases.push(DepartmentAlias::new(alias, Rc::clone(&new_department)));
        let insertion_index = self.dept_aliases.binary_search(&dept_alias);
        if let Err(i) = insertion_index {
            self.dept_aliases.insert(i, dept_alias);
        } else {
            eprintln!("Error adding department, alias already in use");
            return Err(DataError::AddDept);
        }

        self.departments.insert(String::from(alias), Rc::clone(&new_department));

        Ok(new_department)
    }

    /// Add a person to active program data.  Unlike add_department, the Person struct must be
    /// fully initialised and provided as an argument.  This is due to Person being more complicated
    /// thus requiring more parameters than would be convenient to pass to a method.
    /// The personnel module provides a builder for Person to make things a little more readable.
    /// This method takes ownership of the Person data.
    pub fn add_person(&mut self, alias: &str, person: Person) -> Result<Rc<RefCell<Person>>> {
        if self.personnel.contains_key(alias) {
            return Err(DataError::AddPerson);
        }

        // Add person to a new smart pointer
        let person_ref = Rc::new(RefCell::new(person));

        // Add the Rc to the alias list
        self.person_aliases.push(
            PersonAlias::new(
                alias,
                Rc::clone(&person_ref)
            )
        );

        // Add the Rc to the personnel HashMap
        self.personnel.insert(
            String::from(alias),
            Rc::clone(&person_ref)
        );

        // increment employee_count
        self.employee_count += 1;

        // add person to their initial department
        person_ref.borrow()
            .department()
            .borrow_mut()
            .add_employee(Rc::clone(&person_ref))?;

        Ok(person_ref)
    }

    pub fn dept_list(&self) -> &Vec<DepartmentAlias> {
        &self.dept_aliases
    }

    pub fn employee_list(&self) -> &Vec<PersonAlias> {
        &self.person_aliases
    }

    /// Add an existing employee to a Department.  Employee must have already been entered into
    /// ProgramData. There is no need to add a person to their initial department, this is done
    /// automatically upon inserting the Person into ProgramData.
    pub fn add_to_dept(&mut self, person_alias: &str, dept_alias: &str, date: Option<NaiveDate>) -> Result<()> {
        let person = self.personnel.get(person_alias)
            .ok_or(DataError::NoSuchPerson)?;

        let department = self.departments.get(dept_alias)
            .ok_or(DataError::NoSuchDept)?;

        let transfer_date = match date {
            Some(d) => d,
            None => Local::today().naive_local(),
        };

        person.borrow_mut()
            .transfer(Rc::clone(department), transfer_date)?;

        Ok(())
    }

    pub fn departments(&self) -> &HashMap<String, Rc<RefCell<Department>>> {
        &self.departments
    }
}

#[derive(Debug)]
pub enum DataError {
    AddDept,
    AddPerson,
    NoSuchDept,
    NoSuchPerson,
    Person(PersonError),
    Department(DeptErr),
}

impl From<DeptErr> for DataError {
    fn from(error: DeptErr) -> DataError {
        DataError::Department(error)
    }
}

impl From<PersonError> for DataError {
    fn from(error: PersonError) -> DataError {
        DataError::Person(error)
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::DataError::*;

        match self {
            AddDept => write!(f, "Could not create department, alias in use"),
            AddPerson => write!(f, "Could not add Person, alias in use"),
            NoSuchDept => write!(f, "Could not find department matching query"),
            NoSuchPerson => write!(f, "Could not find person matching query"),
            Person(e) => write!(f, "Error on transfer: {}", e),
            Department(e) => write!(f, "Error on add_person: {}", e),
        }
    }
}

impl Error for DataError { }

#[derive(Debug)]
pub struct DepartmentAlias {
    alias: String,
    pointer: Rc<RefCell<Department>>,
}

impl Eq for DepartmentAlias { }

impl PartialEq for DepartmentAlias {
    fn eq(&self, other: &Self) -> bool {
        if self.pointer.borrow().deref() != other.pointer.borrow().deref() {
            return false;
        }

        if self.alias != other.alias {
            return false;
        }

        true
    }
}

impl PartialOrd for DepartmentAlias {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DepartmentAlias {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.alias.cmp(&other.alias)
    }
}

impl DepartmentAlias {
    pub fn new(alias: &str, pointer: Rc<RefCell<Department>>) -> Self {
        DepartmentAlias {
            alias: String::from(alias),
            pointer,
        }
    }

    pub fn clone_pointer(&self) -> Rc<RefCell<Department>> {
        Rc::clone(&self.pointer)
    }

    pub fn borrow_pointer(&self) -> &Rc<RefCell<Department>> {
        &self.pointer
    }

    pub fn alias(&self) -> &String {
        &self.alias
    }
}

impl fmt::Display for DepartmentAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.alias)
    }
}

pub struct PersonAlias {
    alias: String,
    pointer: Rc<RefCell<Person>>,
}

impl PersonAlias {
    pub fn new(alias: &str, pointer: Rc<RefCell<Person>>) -> Self {
        PersonAlias {
            alias: String::from(alias),
            pointer,
        }
    }

    pub fn alias(&self) -> &String {
        &self.alias
    }

    pub fn pointer(&self) -> Rc<RefCell<Person>> {
        Rc::clone(&self.pointer)
    }
}

impl fmt::Display for PersonAlias {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.alias)
    }
}