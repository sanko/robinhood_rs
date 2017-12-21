extern crate robinhood;

use robinhood::Client;

use std::env;

// Log in and grab some of our order history
fn main() {
    let username = &env::var("RHUSER").unwrap();
    let password = &env::var("RHPASSWORD").unwrap();

    let rh = Client::new().login(username, password).build().unwrap();

    let instruments = rh.instruments();
    println!("{:#?}", instruments);
    for instrument in instruments.take(3) {
        println!("Instrument: {:#?}", instrument);
    }
}
