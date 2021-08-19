use std::collections::HashMap;
use std::error::Error;

use zbus::dbus_proxy;
use zvariant::Value;

#[dbus_proxy]
trait Notifications {
    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: &[&str],
        hints: HashMap<&str, &Value>,
        expire_timeout: i32,
    ) -> zbus::Result<u32>;
}

#[dbus_proxy(
    interface = "org.zbus.MyGreeter1",
    default_service = "org.zbus.MyGreeter",
    default_path = "/org/zbus/MyGreeter"
)]
trait Greeter {
    fn say_hello(&self, name: &str) -> zbus::Result<String>;
    fn sleep(&self) -> zbus::Result<()>;
}

fn main() -> Result<(), Box<dyn Error>> {
    let connection = zbus::Connection::new_session()?;

    // let proxy = NotificationsProxy::new(&connection)?;
    // let reply = proxy.notify(
    //     "my-app",
    //     0,
    //     "dialog-information",
    //     "A summary",
    //     "Some body",
    //     &[],
    //     HashMap::new(),
    //     5000,
    // )?;
    // dbg!(reply);
    let proxy = GreeterProxy::new(&connection)?;
    let reply = proxy.say_hello("Brilliant")?;
    dbg!(reply);
    Ok(())
}
