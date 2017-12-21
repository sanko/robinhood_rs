extern crate robinhood;

use robinhood::Client;

// Use a basic client (without login info) to grab RH's instrument list
fn main() {
    let rh = Client::new().build().unwrap();

    let instruments = rh.instruments();
    println!("{:#?}", instruments);
    for instrument in instruments.take(3) {
        println!("Instrument: {:#?}", instrument);
    }
}
