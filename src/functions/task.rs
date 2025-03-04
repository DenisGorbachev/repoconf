use demand::Confirm;
use std::io;

pub fn task<S: Into<String>>(title: S) -> io::Result<bool> {
    Confirm::new(title).run()
}
