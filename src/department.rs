use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

use crate::personnel::Person;

#[derive(Debug, PartialEq, Eq)]
pub struct Department {
    name: String,
    id: u32,
    employees: Vec<Rc<RefCell<Person>>>,
}

impl fmt::Display for Department {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Dept. #{}: {}, {} employees", self.id, self.name, self.employees.len())
    }
}

impl Department {
    pub fn new(name: &str, id: u32) -> Self {
        Department {
            name: String::from(name),
            id,
            employees: Vec::new(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    /// Remove an employee from this department's list of employees. Returns a Rc smart pointer
    /// to the removed instance of employee on success.  Err(DeptErr) on failure.  This function
    /// depends on the list of employees being sorted, which it should be by default.
    /// NOTE: It is better to use Person::transfer() than to invoke this function directly, as
    /// transfer() does some additional data handling on Person to keep things consistent.
    /// See Person::transfer() source for details, know what you're doing if you decide to ignore
    /// this.
    pub fn remove_employee(&mut self, employee: &Person) -> Result<Rc<RefCell<Person>>, DeptErr> {
        let index = self.employees.binary_search_by(|p| {
            p.borrow().cmp(employee)
        });

        match index {
            Ok(i) => return Ok(self.employees.remove(i)),
            Err(_) => return Err(DeptErr::RemoveEmployee),
        };
    }

    /// Add an employee to this departments list of employees.  Returns Ok(()) on success,
    /// Err(DeptErr) on failure.  This method inserts the employee into a position in the list in
    /// order to maintain sorting.
    /// NOTE: It is better to use Person::transfer() than to invoke this function directly, as
    /// transfer() does some additional data handling on Person to keep things consistent.
    /// See Person::transfer() source for details, know what you're doing if you decide to ignore
    /// this.
    pub fn add_employee(&mut self, employee: Rc<RefCell<Person>>) -> Result<(), DeptErr> {
        let index = self.employees.binary_search_by(|p| {
            p.borrow().cmp(&employee.borrow())
        });

        match index {
            Ok(_) => return Err(DeptErr::AddEmployee),
            Err(i) => {
                self.employees.insert(i, Rc::clone(&employee));
                return Ok(());
            },
        };
    }
}

#[derive(Debug)]
pub enum DeptErr {
    RemoveEmployee,
    AddEmployee,
}

impl fmt::Display for DeptErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeptErr::RemoveEmployee => write!(f, "Employee not found in department"),
            DeptErr::AddEmployee => write!(f, "Employee already listed in department"),
        }
    }
}