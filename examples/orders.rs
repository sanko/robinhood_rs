extern crate robinhood;

use robinhood::Client;

use std::env;

// Log in and grab some of our order history
fn main() {
    let username = &env::var("RHUSER").unwrap();
    let password = &env::var("RHPASSWORD").unwrap();

    let rh = Client::new()
        .login(username, password)
        .build().unwrap();

    let orders = rh.orders();
    //println!("{:#?}", orders);
    for order in orders.take(100) {
        println!("Order: {:#?}", order);
        //rh.cancel(order.unwrap());
    }
}
