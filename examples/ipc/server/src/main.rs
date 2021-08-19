use std::convert::TryInto;
use std::error::Error;
use zbus::{dbus_interface, fdo};

struct Greeter {
    count: u64,
}

#[dbus_interface(name = "org.zbus.MyGreeter1")]
impl Greeter {
    fn say_hello(&mut self, name: &str) -> String {
        self.count += 1;
        format!("Hello {}! I have been called: {}", name, self.count)
    }
    fn sleep(&mut self) {
        std::process::Command::new("systemctl")
            .arg("suspend")
            .output()
            .unwrap();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let connection = zbus::Connection::new_session()?;
    fdo::DBusProxy::new(&connection)?.request_name(
        "org.zbus.MyGreeter",
        fdo::RequestNameFlags::ReplaceExisting.into(),
    )?;

    let mut object_server = zbus::ObjectServer::new(&connection);
    let mut greeter = Greeter { count: 0 };
    object_server.at(&"/org/zbus/MyGreeter".try_into()?, greeter)?;
    loop {
        if let Err(err) = object_server.try_handle_next() {
            eprintln!("{}", err);
        }
    }
}
