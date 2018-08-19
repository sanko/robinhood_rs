extern crate robinhood;

use robinhood::Client;

use std::env;

// Log in and grab some of our order history
fn main() {
    let username = &env::var("RHUSER").unwrap();
    let password = &env::var("RHPASSWORD").unwrap();

    let rh = Client::new().login(username, password).build().unwrap();

    let instrument = rh.instrument_by_symbol("MSFT").unwrap();
    let mut market_order = rh.buy(30, instrument);
    market_order.limit(0.010);
    //    pub fn sell(&self, quantity: u64, instrument: Instrument, account: Account) -> OrderBuilder {

        println!("{:?}", market_order);
market_order.send();
//println!("{:?}", market_order.url());


//market_order.stop(7.54);

//println!("{:?}", market_order.url());


    let orders = rh.orders();
    //println!("{:#?}", orders);
    for order in orders.take(10) {
        println!("Order: {:#?}", order);
        //rh.cancel(order.unwrap());
    }
    
}
