extern crate robinhood;

use robinhood::Client;

use std::env;

// Use a basic client (without login info) to grab RH's account list
fn main() {
    let username = &env::var("RHUSER").unwrap();
    let password = &env::var("RHPASSWORD").unwrap();

    let rh = Client::new().login(username, password).build().unwrap();

    let accounts = rh.accounts();
    println!("{:#?}", accounts);
    for account in accounts.take(3) {
        println!("Account: {:#?}", account);
    }

    //rh.logout();
}
