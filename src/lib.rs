//! ## robinhood
//! Library used to access the Client API with Rust
//!
//! ## Installation
//!
//! ```toml
//! [dependencies]
//! robinhood = "*"
//! ```
//!
//! ## Example
//!
//! ```rust
//! extern crate robinhood;
//!
//! use robinhood::Client;
//!
//! fn main() {
//!     let rh = Client::new()
//!         .build()
//!         .unwrap();
//!
//!     let instruments = rh.instruments();
//!     println!("{:#?}", instruments);
//!     for instrument in instruments.take(3) {
//!         println!("Instrument: {:#?}", instrument);
//!     }
//! }
//! ```
//!

#[macro_use]
extern crate error_chain;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time;

use std::rc::Rc;
use std::cell::RefCell;

use reqwest::{Client as HTTPClient, Response};
use reqwest::header::{Authorization, Bearer, Headers, UserAgent};

use std::collections::HashMap;

#[macro_use]
pub mod macros;

error_chain! {
    foreign_links {
        Reqwest(reqwest::Error);
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginatedApiResponse<T> {
    previous: Option<String>,
    next: Option<String>,
    pub results: Vec<T>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuthToken {
    backup_code: Option<String>,
    access_token: Option<String>,
    expires_in: Option<u32>,
    token_type: Option<String>, // Bearer
    scope: Option<String>,
    refresh_token: Option<String>,
    // Locally defined
    birth: Option<i64>,
    // MultiFactor
    mfa_code: Option<String>,
    // MultiFactor waiting for code...
    mfa_type:     Option<String>,
    mfa_required: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlainAuthToken {
    token: Option<String>,
    // MultiFactor
    mfa_code: Option<String>,
    // MultiFactor waiting for code...
    mfa_type:     Option<String>,
    mfa_required: Option<bool>,
}

/// A client/app is represented here
pub struct Client {
    /// This is documentation for the `Client` structure.
    /// # Examples
    pub client: HTTPClient,
    authorized: bool,
}

impl Client {
    /// Creates a new client builder
    ///
    /// More words here. Where do they end up?
    ///
    /// # Arguments
    ///
    /// * None
    ///
    /// # Example
    ///
    /// ```
    /// use robinhood::Client;
    /// let person = Client::new().build();
    /// ```
    pub fn new() -> ClientBuilder {
        let mfa_callback = move |mfa_type: String| -> String {
            use std::io::{stdin, stdout, Write};
            let mut s = String::new();
            print!("Please enter MFA code from {}: ", mfa_type);
            let _ = stdout().flush();
            stdin().read_line(&mut s).expect("Oops! Try again: ");
            if let Some('\n') = s.chars().next_back() {
                s.pop();
            }
            if let Some('\r') = s.chars().next_back() {
                s.pop();
            }
            s
        };

        let cell = Rc::new(RefCell::new(mfa_callback));

        ClientBuilder {
            username: None,
            password: None,
            agent: "Robinhood/2622 (Android 6.1;)".to_string(),
            client_string: None,                 // OAuth2
            scope: Some("internal".to_string()), // OAuth2
            mfa_callback: cell,
        }
    }

    pub fn authorized(&self) -> bool {
        self.authorized
    }

    /// Creates a new client builder
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the person
    ///
    /// # Example
    ///
    /// ```
    /// // You can have rust code between fences inside the comments
    /// // If you pass --test to Rustdoc, it will even test it for you!
    /// use robinhood::Client;
    /// let person = Client::new().build();
    /// ```

    pub fn instruments(&self) -> Instruments {
        Instruments::new_with_client(self.client.to_owned())
    }

    pub fn accounts(&self) -> Accounts {
        // if self.authorized {
        Accounts::new_with_client(self.client.to_owned())
        //}
    }

    pub fn orders(&self) -> Orders {
        // if self.authorized {
        Orders::new_with_client(self.client.to_owned())
        //}
    }

    pub fn get(&self, url: &str) -> Response {
        self.client.to_owned().get(url).send().unwrap()
    }
}

pub struct ClientBuilder {
    username: Option<String>,
    password: Option<String>,
    agent: String,
    client_string: Option<String>, // OAuth2
    scope: Option<String>,         /* OAuth2: read, watchlist, investments, trade, balances,
                                    * funding:all:read */
    mfa_callback: Rc<RefCell<FnMut(String) -> String>>,
}

impl ClientBuilder {
    pub fn user_agent(&mut self, agent: &str) -> &mut ClientBuilder {
        self.agent = agent.to_string();
        self
    }

    pub fn oauth_client(&mut self, client_string: &str) -> &mut ClientBuilder {
        self.client_string = Some(client_string.to_owned());
        self
    }

    pub fn oauth_scope(&mut self, scope: &str) -> &mut ClientBuilder {
        self.scope = Some(scope.to_owned());
        self
    }

    // pub fn mfa(&mut self, callback: Box<Fn() -> String + 'static>) -> &mut
    // ClientBuilder {
    pub fn mfa<F: FnMut(String) -> String + 'static>(&mut self, callback: F) -> &mut ClientBuilder {
        let cell = Rc::new(RefCell::new(callback));
        self.mfa_callback = cell;
        self
    }

    pub fn login(&mut self, username: &str, password: &str) -> &mut ClientBuilder {
        self.username = Some(username.to_owned());
        self.password = Some(password.to_owned());
        self
    }

    fn _get_mfa_code(&self, mfa_type: String) -> String {
        let mut closure = self.mfa_callback.as_ref().borrow_mut();
        // Unfortunately, Rust's auto-dereference of pointers is not clever enough
        // here. We thus have to explicitly dereference the smart
        // pointer and obtain a mutable borrow of the target.
        let mfa_code: String = (&mut *closure)(mfa_type);
        mfa_code
    }

    fn _oauth_login(&self, mfa_code: Option<String>) -> Option<OAuthToken> {
        let mut params = HashMap::new();
        params.insert("grant_type", "password");
        params.insert("username", self.username.as_ref().unwrap());
        params.insert("password", self.password.as_ref().unwrap());
        params.insert("scope", self.scope.as_ref().unwrap());
        params.insert("client_id", self.client_string.as_ref().unwrap());

        if mfa_code.is_some() {
            params.insert("mfa_code", mfa_code.as_ref().unwrap());
        }

        // ("backup_code", backup_code.into())
        let client = HTTPClient::new();

        let mut res = client
            .post("https://api.robinhood.com/oauth2/token/")
            .header(UserAgent::new(self.agent.to_owned()))
            .form(&params.to_owned())
            .send()
            .unwrap()
            .json::<OAuthToken>()
            .unwrap();

        if mfa_code.is_none() && res.mfa_required.is_some() && res.mfa_required.unwrap() {
            let mfa = &self._get_mfa_code(res.mfa_type.unwrap());
            return self._oauth_login(Some(mfa.to_string()));
        }
        else {
            res.birth = Some(time::get_time().sec);
        }

        Some(res)
    }

    fn _classic_login(&self, mfa_code: Option<String>) -> Option<PlainAuthToken> {
        let mut params = HashMap::new();
        params.insert("username", self.username.as_ref().unwrap());
        params.insert("password", self.password.as_ref().unwrap());

        if mfa_code.is_some() {
            params.insert("mfa_code", mfa_code.as_ref().unwrap());
        }

        // ("backup_code", backup_code.into())
        let client = HTTPClient::new();

        let res = client
            .post("https://api.robinhood.com/api-token-auth/")
            .header(UserAgent::new(self.agent.to_owned()))
            .form(&params.to_owned())
            .send()
            .unwrap()
            .json::<PlainAuthToken>()
            .unwrap();

        if mfa_code.is_none() && res.mfa_required.is_some() && res.mfa_required.unwrap() {
            let mfa = &self._get_mfa_code(res.mfa_type.unwrap());
            return self._classic_login(Some(mfa.to_string()));
        }

        Some(res)
    }

    pub fn build(&mut self) -> Result<Client> {
        let mut headers = Headers::new();
        headers.set(UserAgent::new(self.agent.to_owned()));
        let mut authorized = false;

        if self.username.is_some() && self.username.is_some() {
            if self.client_string.is_some() {
                // OAuth2

                // let mfa_callback = self.mfa_callback.as_ref();
                let token = self._oauth_login(None);
                if token.is_some() {
                    headers.set(Authorization(Bearer {
                        token: token.unwrap().access_token.to_owned().unwrap(),
                    }));
                }
            }
            else {
                // Old skool
                let token = self._classic_login(None);
                headers.set(Authorization(
                    String::from("Token ") + token.unwrap().token.to_owned().unwrap().as_ref(),
                ));
            }

            authorized = true;
        }

        // println!("{:?}", headers);

        let client = HTTPClient::builder().default_headers(headers).build()?;

        Ok(Client {
            client:     client,
            authorized: authorized,
        })
    }
}

// Conditionally compile the module `test` only when the test-suite is run.
#[cfg(test)]
mod test_client_builder {
    use super::Client;

    #[test]
    fn client_builder() {
        assert!(Client::new().build().is_ok());
    }

    #[test]
    #[should_panic]
    fn client_builder_bad_login() {
        assert!(Client::new().login("username", "password").build().is_ok());
    }
}

iter_builder!(
    Instruments => Instrument as InstrumentData, "https://api.robinhood.com/instruments/" {
    min_tick_size: Option<String> = None,
    #[serde(rename = "type")]
    type_field: String = None,
    splits: String = None,
    margin_initial_ratio: String = None,
    url: String = None,
    quote: String = None,
    tradability: String = None,
    bloomberg_unique: String = None,
    list_date: Option<String> = None,
    name: String = None,
    symbol: String = None,
    fundamentals: String = None,
    state: String = None,
    country: String = None,
    day_trade_ratio: String = None,
    tradeable: bool = None,
    maintenance_ratio: String = None,
    id: String = None,
    market: String = None,
    simple_name: Option<String> = None
});

impl Instruments {
    pub fn search_by_symbol<S>(symbol: S) -> Result<Instrument>
    where S: Into<String> {
        let mut inst = Instruments {
            results: vec![].into_iter(),
            client:  HTTPClient::new(),
            next:    Some(
                format!(
                    "https://api.robinhood.com/instruments/?symbol={}",
                    symbol.into()
                ).to_owned(),
            ),
        };

        inst.nth(0).unwrap()
    }
}

// Conditionally compile the module `test` only when the test-suite is run.
#[cfg(test)]
mod test_instruments {
    use super::Client;

    fn init_client() -> Client {
        Client::new().build().unwrap()
    }

    #[test]
    fn grab_instruments() {
        let rh = init_client();

        let instruments = rh.instruments();
        println!("{:#?}", instruments);

        for instrument in instruments.take(100) {
            println!("Instrument: {:#?}", instrument);
            assert!(instrument.is_ok());
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarginBalances {
    day_trade_buying_power: String,
    start_of_day_overnight_buying_power: String,
    overnight_buying_power_held_for_orders: String,
    cash_held_for_orders: String,
    created_at: String,
    unsettled_debit: String,
    start_of_day_dtbp: String,
    day_trade_buying_power_held_for_orders: String,
    overnight_buying_power: String,
    marked_pattern_day_trader_date: serde_json::Value,
    cash: String,
    unallocated_margin_cash: String,
    updated_at: String,
    cash_available_for_withdrawal: String,
    margin_limit: String,
    outstanding_interest: String,
    uncleared_deposits: String,
    unsettled_funds: String,
    gold_equity_requirement: String,
    day_trade_ratio: String,
    overnight_ratio: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstantEligibility {
    updated_at: serde_json::Value,
    reason: String,
    reinstatement_date: serde_json::Value,
    reversal: serde_json::Value,
    state: String,
}

iter_builder!(
    Accounts => Account as AccountData, "https://api.robinhood.com/accounts/" {
    deactivated: bool = None,
    updated_at: String = None,
    margin_balances: MarginBalances = None,
    portfolio: String = None,
    cash_balances: serde_json::Value = None,
    can_downgrade_to_cash: String = None,
    withdrawal_halted: bool = None,
    cash_available_for_withdrawal: String = None,
    #[serde(rename = "type")]
    type_field: String = None,
    sma: String = None,
    sweep_enabled: bool = None,
    deposit_halted: bool = None,
    buying_power: String = None,
    user: String = None,
    max_ach_early_access_amount: String = None,
    instant_eligibility: InstantEligibility = None,
    cash_held_for_orders: String = None,
    only_position_closing_trades: bool = None,
    url: String = None,
    positions: String = None,
    created_at: String = None,
    cash: String = None,
    sma_held_for_orders: String = None,
    unsettled_debit: String = None,
    account_number: String = None,
    uncleared_deposits: String = None,
    unsettled_funds: String = None
});

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Execution {
    timestamp: String,
    price: String,
    settlement_date: String,
    id: String,
    quantity: String,
}

iter_builder!(
    Orders => Order as OrderData, "https://api.robinhood.com/orders/" {
    updated_at: String = None,
    ref_id: Option<String> = None,
	time_in_force: String = None,
	fees: String = None,
	cancel: serde_json::Value = None,
	id: String = None,
	cumulative_quantity: String = None,
	stop_price: Option<String> = None,
	reject_reason: serde_json::Value = None,
	instrument: String = None,
	state: String = None,
	trigger: String = None,
	override_dtbp_checks: bool = None,
	#[serde(rename = "type")]
    type_field: String = None,
    last_transaction_at: String = None,
    price: Option<String> = None,
    executions: Vec<Execution> = Vec::new(),
    extended_hours: bool = None,
    account: String = None,
    url: String = None,
    created_at: String = None,
    side: String = None,
    override_day_trade_checks: bool = None,
    position: String = None,
    average_price: Option<String> = None,
    quantity: String = None
});

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
