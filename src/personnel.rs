use std::fmt;
use chrono::naive::NaiveDate;
use std::cmp::Ordering;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;
use std::ops::Deref;

use crate::department::Department;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Name {
    pub last: String,
    pub middle: Option<String>,
    pub first: String,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.middle {
            Some(mid) => write!(f, "{}, {} {}", self.last, self.first, mid),
            None => write!(f, "{}, {}", self.last, self.first),
        }
    }
}

#[derive(Debug)]
pub struct Person {
    name: Name,
    date_of_hire: NaiveDate,
    department: Rc<RefCell<Department>>,
    dept_history: Vec<DeptEntry>,
}

impl PartialEq for Person {
    fn eq(&self, other: &Self) -> bool {
        if &self.name != &other.name {
            return false;
        }

        if &self.date_of_hire != &other.date_of_hire {
            return false;
        }

        if !self.department.borrow().eq(other.department.borrow().deref()) {
            return false;
        }

        if &self.dept_history != &other.dept_history {
            return false;
        }

        true
    }
}

impl Eq for Person {}

impl Person {
    pub fn builder() -> PersonBuilder {
        PersonBuilder::new()
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn name_mut(&mut self) -> &mut Name {
        &mut self.name
    }

    pub fn first_name(&self) -> &String {
        &self.name.first
    }

    pub fn last_name(&self) -> &String {
        &self.name.last
    }

    pub fn middle_name(&self) -> Option<&String> {
        if let Some(mid) = &self.name.middle {
            return Some(mid);
        } else {
            return None;
        }
    }

    pub fn date_of_hire(&self) -> NaiveDate {
        self.date_of_hire
    }

    pub fn department(&self) -> Rc<RefCell<Department>> {
        Rc::clone(&self.department)
    }

    pub fn department_history(&self) -> &Vec<DeptEntry> {
        &self.dept_history
    }

    pub fn department_history_mut(&mut self) -> &mut Vec<DeptEntry> {
        &mut self.dept_history
    }

    /// transfer an employee from their current department to another. Returns empty Ok(()) on
    /// success, or Err(personnel::PersonError) on failure.  Fails if self is not found listed in
    /// their current department, which would be indicative of an error in this API or mishandling
    /// of an employee Vec.  Can also fail if self is found listed in the department they are being
    /// transferred to.  Neither condition should happen, and will lead to database corruption.
    pub fn transfer(&mut self, department: Rc<RefCell<Department>>, date: NaiveDate) -> Result<(), PersonError> {
        // Naturally return Err if trying to transfer to the department self is already a member of
        if self.department.borrow().eq(&department.borrow().deref()) { // This error is non-critical
            return Err(PersonError::Transfer(TransferErr::AlreadyInDept));
        }

        // Set up an entry for self.dept_history
        let entry = DeptEntry {
            date,
            department,
        };

        // Remove self from old department, store the result
        let result = self.department.borrow_mut().remove_employee(&self);

        // Get the Rc for self from the previous result, panic! on Err
        let self_ref = result.expect(
            &format!("Error: {}", PersonError::Transfer(TransferErr::NotListedInDept))
        );

        // Add the Rc to the new department, panic! on Err
        entry.department.borrow_mut().add_employee(Rc::clone(&self_ref))
            .expect(&format!("Error: {}", PersonError::Transfer(TransferErr::AlreadyInDept)) );

        // Update self's department Rc
        self.department = Rc::clone(&entry.department);

        // Add the previous department to self.dept_history
        self.dept_history.push(entry);

        // Success!
        Ok(())
    }
}

#[derive(Debug)]
pub enum PersonError {
    Transfer(TransferErr),
}

impl fmt::Display for PersonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PersonError::Transfer(e) => write!(f, "Transfer failed: {}", e)
        }
    }
}

impl Error for PersonError {}

impl From<TransferErr> for PersonError {
    fn from(error: TransferErr) -> Self {
        PersonError::Transfer(error)
    }
}

#[derive(Debug)]
pub enum TransferErr {
    NotListedInDept,
    AlreadyInDept,
}

impl fmt::Display for TransferErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransferErr::NotListedInDept => write!(f, "Person not listed in department"),
            TransferErr::AlreadyInDept => write!(f, "Invalid transfer to same Dept"),
        }
    }
}

impl Error for TransferErr {}

impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, DOH: {}, {}", self.name, self.date_of_hire, self.department.borrow().name())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DeptEntry {
    department: Rc<RefCell<Department>>,
    date: NaiveDate,
}

impl fmt::Display for DeptEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.department.borrow().name(), self.date.format("%m/%d/%Y"))
    }
}

impl PartialOrd for Person {
    fn partial_cmp(&self, other: &Person) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Person {
    fn cmp(&self, other: &Person) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Name) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Name) -> Ordering {
        if self.last != other.last {
            self.last.cmp(&other.last)
        } else {
            self.first.cmp(&other.first)
        }
    }
}

#[derive(Debug)]
pub struct PersonBuilder {
    name_first: Option<String>,
    name_last: Option<String>,
    name_mid: Option<String>,
    doh: Option<NaiveDate>,
    dept: Option<Rc<RefCell<Department>>>,
}

impl PersonBuilder {
    fn new() -> Self {
        PersonBuilder {
            name_first: None,
            name_last: None,
            name_mid: None,
            doh: None,
            dept: None,
        }
    }

    pub fn first_name(&mut self, first_name: &str) -> &mut Self {
        self.name_first = Some(String::from(first_name));
        self
    }

    pub fn middle_name(&mut self, middle_name: &str) -> &mut Self {
        self.name_mid = Some(String::from(middle_name));
        self
    }

    pub fn last_name(&mut self, last_name: &str) -> &mut Self {
        self.name_last = Some(String::from(last_name));
        self
    }

    pub fn date_of_hire(&mut self, date_of_hire: NaiveDate) -> &mut Self {
        self.doh = Some(date_of_hire);
        self
    }

    pub fn department(&mut self, department: Rc<RefCell<Department>>) -> &mut Self {
        self.dept = Some(department);
        self
    }

    /// Construct an instance of Person from the given values.  Returns Ok(Person) on success, or
    /// Err(Self) on failure.  Function consumes self.
    pub fn build(self) -> Result<Person, Self> {
        if self.name_first.is_none() || self.name_last.is_none() || self.doh.is_none() || self.dept.is_none() {
            return Err(self);
        }

        let name = Name {
            last: self.name_last.unwrap(),
            middle: self.name_mid,
            first: self.name_first.unwrap(),
        };

        let department_ref = self.dept.unwrap();
        let doh = self.doh.unwrap();

        let dept_entry = DeptEntry {
            date: doh,
            department: Rc::clone(&department_ref),
        };

        Ok(Person {
            name,
            date_of_hire: doh,
            department: Rc::clone(&department_ref),
            dept_history: vec![dept_entry],
        })
    }
}