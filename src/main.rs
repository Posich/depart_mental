// Ex. 3: Using a hash map and vectors, create a text interface to allow a user to add employee
// names to a department in a company.  For example, "Add Sally to Engineering" or "Add Amir to
// Sales." Then let the user retrieve a list of all people in a department or all people in the
// company by department, sorted alphabetically.
use depart_mental::textinterface::TextInterface;


fn main() {
    let mut interface = TextInterface::init();

    interface.run().expect("fart");
}
