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

extern crate chrono;

use std::cell::RefCell;
use std::rc::Rc;

use reqwest::header::{Authorization, Bearer, ContentType, Headers, UserAgent};
use reqwest::{Client as HTTPClient, Response};

use std::collections::HashMap;

use std::io::Read;

use chrono::naive::NaiveDate;
use chrono::prelude::*;

#[macro_use]
pub mod macros;

error_chain! {
    foreign_links {
        Reqwest(reqwest::Error);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginatedApiResponse<T> {
    previous: Option<String>,
    next: Option<String>,
    pub results: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuthToken {
    backup_code: Option<String>,
    access_token: Option<String>,
    expires_in: Option<u32>,
    token_type: Option<String>, // Bearer
    scope: Option<String>,
    refresh_token: Option<String>,
    // Locally defined
    birth: Option<DateTime<Utc>>,
    // MultiFactor
    mfa_code: Option<String>,
    // MultiFactor waiting for code...
    mfa_type: Option<String>,
    mfa_required: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlainAuthToken {
    token: Option<String>,
    // MultiFactor
    mfa_code: Option<String>,
    // MultiFactor waiting for code...
    mfa_type: Option<String>,
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
            agent: "Robinhood/2672 (Android 6.1;)".to_string(),
            client_string: None,                 // OAuth2
            scope: Some("internal".to_string()), // OAuth2
            mfa_callback: cell,
        }
    }

    pub fn _get(&self, url: &str) -> String {
        let mut body = String::new();
        let mut res = self._get_res(url);
        //println!("{:?}", res);
        res.read_to_string(&mut body).unwrap();
        body
    }

    pub fn _get_res(&self, url: &str) -> Response {
        let mut req = self.client.get(url);
        req.send().unwrap()
    }

    pub fn _post(&self, url: &str, params: Option<HashMap<&str, &str>>) -> String {
        let mut body = String::new();
        let mut res = self._post_res(url, params);
        println!("{:?}", res);
        res.read_to_string(&mut body).unwrap();
        body
    }

    pub fn _post_res(&self, url: &str, params: Option<HashMap<&str, &str>>) -> Response {
        let mut req = self.client.post(url);

        if params.is_some() {
            req.form(&params.to_owned());
        }

        req.send().unwrap()
    }

    pub fn _patch(&self, url: &str, patch: serde_json::Map<String, serde_json::Value>) -> String {
        let mut body = String::new();
        let mut res = self._patch_res(url, patch);
        println!("{:?}", res);
        res.read_to_string(&mut body).unwrap();
        body
    }

    pub fn _patch_res(
        &self,
        url: &str,
        patch: serde_json::Map<String, serde_json::Value>,
    ) -> Response {
        self.client
            .patch(url)
            .header(ContentType::json())
            .body(serde_json::to_string(&patch).unwrap())
            .send()
            .unwrap()
    }

    /// Checks whether or not the client is authorized with an account.
    ///
    /// # Arguments
    ///
    /// None
    ///
    /// # Example
    ///
    /// ```
    /// // Some methods require an account to access
    /// use robinhood::Client;
    /// let rh = Client::new().build().unwrap();
    /// if rh.authorized() { // False because we didn't even attempt to log in
    ///    let orders = rh.orders();
    /// }
    /// ```
    pub fn authorized(&self) -> bool {
        self.authorized
    }

    /// Unless you're using OAuth2, every client that logs in with your username/password is
    /// given the same token. For security, you can force it to expire with a call to log out.
    ///
    /// Clients logged in with OAuth tokens will not be asked to log back in.
    ///
    /// # Arguments
    ///
    /// None
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use robinhood::Client;
    /// let rh = Client::new().login("username", "password").build().unwrap();
    /// // Do something
    /// rh.logout();
    /// ```
    pub fn logout(&self) -> bool {
        if self.authorized() {
            let mut body = String::new();
            let mut res = self
                .client
                .post("https://api.robinhood.com/api-token-logout/")
                .send()
                .unwrap();

            res.read_to_string(&mut body).unwrap();
            body.is_empty()
        } else {
            false
        }
    }

    /// Creates a recursive iterator for gathering the list of instruments
    ///
    /// # Arguments
    ///
    /// None
    ///
    /// # Example
    ///
    /// ```
    /// use robinhood::Client;
    /// let rh = Client::new().build().unwrap();
    /// let instruments = rh.instruments();
    /// for instrument in instruments.take(3) {
    ///     println!("Instrument: {:#?}", instrument);
    /// }
    /// ```
    pub fn instruments(&self) -> Instruments {
        Instruments::new_with_client(self.client.to_owned())
    }

    pub fn instrument_by_symbol(&self, symbol: &str) -> Result<Instrument> {
        Instruments::search_by_symbol(symbol)
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

    pub fn sell(&self, quantity: u64, instrument: Instrument) -> OrderBuilder {
        let account = self.accounts().nth(0).unwrap().unwrap();
        self.to_owned()
            .sell_with_account(quantity, instrument, account)
    }

    pub fn sell_with_account(
        &self,
        quantity: u64,
        instrument: Instrument,
        account: Account,
    ) -> OrderBuilder {
        OrderBuilder::new(
            self.client.to_owned(),
            "sell",
            quantity,
            instrument,
            account,
        )
        // pub fn new( side: &str, quantity: u64, instrument: Instrument,
        // account: Account ) -> OrderBuilder {
    }

    pub fn buy(&self, quantity: u64, instrument: Instrument) -> OrderBuilder {
        let account = self.accounts().nth(0).unwrap().unwrap();
        self.buy_with_account(quantity, instrument, account)
    }

    pub fn buy_with_account(
        &self,
        quantity: u64,
        instrument: Instrument,
        account: Account,
    ) -> OrderBuilder {
        let order_builder: OrderBuilder =
            OrderBuilder::new(self.client.to_owned(), "buy", quantity, instrument, account);
        order_builder
        // pub fn new( side: &str, quantity: u64, instrument: Instrument,
        // account: Account ) -> OrderBuilder {
    }

    pub fn cancel(&self, order: Order) -> bool {
        if order.can_cancel().is_none() {
            return false;
        }
        let res = self
            .client
            .post(order.can_cancel().unwrap().as_str())
            .send()
            .unwrap();
        res.status().is_success()
    }

    pub fn positions(&self) -> Positions {
        let account = self.accounts().nth(0).unwrap().unwrap();
        Positions::new_with_client(self.client.to_owned())
            .set_next(account.positions())
            .to_owned()
    }

    pub fn positions_with_account(&self, account: Account) -> Positions {
        Positions::new_with_client(self.client.to_owned())
            .set_next(account.positions())
            .to_owned()
    }

    pub fn positions_nonzero(&self) -> Positions {
        Positions::new_with_client(self.client.to_owned())
            .set_next("https://api.robinhood.com/positions/?nonzero=true".to_string())
            .to_owned()
    }

    pub fn positions_nonzero_with_account(&self, account: Account) -> Positions {
        let mut url: String = account.positions();
        url.push_str("?nonzero=true");
        Positions::new_with_client(self.client.to_owned())
            .set_next(url)
            .to_owned()
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
        } else {
            res.birth = Some(Utc::now())
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
                // let mfa_callback = self.mfa_callback.as_ref();
                let token = self._oauth_login(None);
                // println!("OAuth2: {:?}", token);
                if token.is_some() {
                    headers.set(Authorization(Bearer {
                        token: token.unwrap().access_token.to_owned().unwrap(),
                    }));
                    authorized = true;
                }
            } else {
                // Old skool
                let token = self._classic_login(None);
                // println!("Classic: {:?}", token);
                if token.is_some() {
                    headers.set(Authorization(
                        String::from("Token ") + token.unwrap().token.to_owned().unwrap().as_ref(),
                    ));
                    authorized = true;
                }
            }
        }

        // println!("{:?}", headers);

        let client = HTTPClient::builder().default_headers(headers).build()?;

        Ok(Client {
            client: client,
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
    list_date: Option<NaiveDate> = None,
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
    simple_name: Option<String> = None,
    rhs_tradability: String = None,
    tradable_chain_id: Option<String> = None
});

impl Instruments {
    pub fn search_by_symbol<S>(symbol: S) -> Result<Instrument>
    where
        S: Into<String>,
    {
        let mut inst = Instruments {
            results: vec![].into_iter(),
            client: HTTPClient::new(),
            next: Some(
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarginBalances {
    day_trade_buying_power: String,
    start_of_day_overnight_buying_power: String,
    overnight_buying_power_held_for_orders: String,
    cash_held_for_orders: String,
    created_at: DateTime<Utc>,
    unsettled_debit: String,
    start_of_day_dtbp: String,
    day_trade_buying_power_held_for_orders: String,
    overnight_buying_power: String,
    marked_pattern_day_trader_date: Option<NaiveDate>,
    cash: String,
    unallocated_margin_cash: String,
    updated_at: DateTime<Utc>,
    cash_available_for_withdrawal: String,
    margin_limit: String,
    outstanding_interest: String,
    uncleared_deposits: String,
    unsettled_funds: String,
    gold_equity_requirement: String,
    day_trade_ratio: String,
    overnight_ratio: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstantEligibility {
    updated_at: Option<DateTime<Utc>>,
    reason: String,
    reinstatement_date: Option<DateTime<Utc>>,
    reversal: Option<serde_json::Value>,
    state: String,
}

iter_builder!(
    Accounts => Account as AccountData, "https://api.robinhood.com/accounts/" {
    deactivated: bool = None,
    updated_at: DateTime<Utc> = None,
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
    created_at: DateTime<Utc> = None,
    cash: String = None,
    sma_held_for_orders: String = None,
    unsettled_debit: String = None,
    account_number: String = None,
    uncleared_deposits: String = None,
    unsettled_funds: String = None,
    nummus_enabled: Option<bool> = None, // Crypto
    option_level: String = None,
    is_pinnacle_account: bool = None
});

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Execution {
    timestamp: String,
    price: String,
    settlement_date: String,
    id: String,
    quantity: String,
}

iter_builder!(
    Orders => Order as OrderData, "https://api.robinhood.com/orders/" {
    account: String = None,
    average_price: Option<String> = None,
    #[serde(rename = "cancel")]
    can_cancel: Option<String> = None,
    created_at: DateTime<Utc> = None,
    cumulative_quantity: String = None,
    executions: Vec<Execution> = Vec::new(),
    extended_hours: bool = None,
    fees: String = None,
    id: String = None,
    instrument: String = None,
    last_transaction_at: DateTime<Utc> = None,
    override_day_trade_checks: bool = None,
    override_dtbp_checks: bool = None,
    position: String = None,
    price: Option<String> = None,
    quantity: String = None,
    ref_id: Option<String> = None,
    reject_reason: Option<String> = None,
    response_category: Option<String> = None,
    side: String = None,
    state: String = None,
    stop_price: Option<String> = None,
    time_in_force: String = None,
    trigger: String = None,
    #[serde(rename = "type")]
    type_field: String = None,
    updated_at: DateTime<Utc> = None,
    url: String = None
});

iter_builder!(
    Positions => Position as PositionData, "https://api.robinhood.com/accounts/{account_id}/positions/" {
    shares_held_for_stock_grants: String = None,
    account: String = None,
    intraday_quantity: String = None,
    intraday_average_buy_price: String = None,
    url: String = None,
    created_at: DateTime<Utc> = None,
    updated_at: DateTime<Utc> = None,
    shares_held_for_buys: String = None,
    average_buy_price: String = None,
    instrument: String = None,
    shares_held_for_sells: String = None,
    quantity: String = None
});

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

#[derive(Debug, Clone)]
pub struct OrderBuilder {
    client: HTTPClient,
    time_in_force: String,
    stop_price: Option<f64>,
    instrument: Instrument,
    override_dtbp_checks: bool,
    _type: String,
    price: Option<f64>,
    extended_hours: bool,
    account: Account,
    side: String,
    override_day_trade_checks: bool,
    quantity: u64,
}

impl OrderBuilder {
    pub fn new(
        ref mut client: HTTPClient,
        side: &str,
        quantity: u64,
        instrument: Instrument,
        account: Account,
    ) -> OrderBuilder {
        OrderBuilder {
            client: client.to_owned(),

            _type: "market".to_owned(),
            side: side.to_owned(),
            time_in_force: "gfd".to_owned(), //  `gfd`, `gtc`, or `opg`

            price: None,
            stop_price: None,
            quantity: quantity,

            instrument: instrument,
            account: account,

            extended_hours: false,
            override_dtbp_checks: false,
            override_day_trade_checks: false,
        }
    }

    pub fn send(&self) -> Order {
        let mut params = HashMap::new();
        params.insert("account", self.account.url());
        params.insert("instrument", self.instrument.url());
        params.insert("symbol", self.instrument.symbol());
        params.insert("type", self._type.to_owned());
        params.insert("time_in_force", self.time_in_force.to_owned());
        params.insert("trigger", "immediate".to_owned());
        params.insert("quantity", self.quantity.to_string());
        params.insert("side", self.side.to_owned());

        if self.stop_price.is_some() {
            params.insert("stop_price", self.stop_price.unwrap().to_string());
            params.insert("trigger", "stop".to_owned());
        }
        if self._type == "market" && self.price.is_none() {
            // self.price =
            // TODO: Get price from quote endpoint
        }
        if self.price.is_some() {
            params.insert("price", self.price.unwrap().to_string());
        }

        params.insert("override_day_trade_checks", "true".to_string());

        if self._type.eq("limit") && self.stop_price.is_none() {
            // params.insert("extended_hours", "true".to_string());
        }

        let mut req = self.client.post("https://api.robinhood.com/orders/");
        req.form(&params.to_owned());
        let res = req.send().unwrap().json::<OrderData>().unwrap();
        Order { data: res }
    }

    pub fn gfd(&mut self) -> &mut OrderBuilder {
        self.time_in_force = "gfd".to_string();
        self
    }
    pub fn gtc(&mut self) -> &mut OrderBuilder {
        self.time_in_force = "gtc".to_string();
        self
    }
    pub fn opg(&mut self) -> &mut OrderBuilder {
        self.time_in_force = "opg".to_string();
        self
    }

    pub fn stop(&mut self, price: f64) -> &mut OrderBuilder {
        self.stop_price = Some(price);
        self
    }

    pub fn limit(&mut self, price: f64) -> &mut OrderBuilder {
        self.price = Some(price);
        self._type = "limit".to_owned();
        self
    }

    pub fn _price(&mut self, price: f64) -> &mut OrderBuilder {
        // Set collar price on market order
        self.price = Some(price);
        self
    }
    // pub fn oauth_client(&mut self, client_string: &str) -> &mut OrderBuilder {
    // self.client_string = Some(client_string.to_owned());
    // self
    // }
    //
    // pub fn oauth_scope(&mut self, scope: &str) -> &mut OrderBuilder {
    // self.scope = Some(scope.to_owned());
    // self
    // }
    //
    // pub fn mfa(&mut self, callback: Box<Fn() -> String + 'static>) -> &mut
    // OrderBuilder {
    // pub fn mfa<F: FnMut(String) -> String + 'static>(&mut self, callback: F) ->
    // &mut OrderBuilder { let cell = Rc::new(RefCell::new(callback));
    // self.mfa_callback = cell;
    // self
    // }
    //
    // pub fn login(&mut self, username: &str, password: &str) -> &mut OrderBuilder
    // { self.username = Some(username.to_owned());
    // self.password = Some(password.to_owned());
    // self
    // }
    //
    // fn _get_mfa_code(&self, mfa_type: String) -> String {
    // let mut closure = self.mfa_callback.as_ref().borrow_mut();
    // Unfortunately, Rust's auto-dereference of pointers is not clever enough
    // here. We thus have to explicitly dereference the smart
    // pointer and obtain a mutable borrow of the target.
    // let mfa_code: String = (&mut *closure)(mfa_type);
    // mfa_code
    // }
    //
    // fn _oauth_login(&self, mfa_code: Option<String>) -> Option<OAuthToken> {
    // let mut params = HashMap::new();
    // params.insert("grant_type", "password");
    // params.insert("username", self.username.as_ref().unwrap());
    // params.insert("password", self.password.as_ref().unwrap());
    // params.insert("scope", self.scope.as_ref().unwrap());
    // params.insert("client_id", self.client_string.as_ref().unwrap());
    //
    // if mfa_code.is_some() {
    // params.insert("mfa_code", mfa_code.as_ref().unwrap());
    // }
    //
    // ("backup_code", backup_code.into())
    // let client = HTTPClient::new();
    //
    // let mut res = client
    // .post("https://api.robinhood.com/oauth2/token/")
    // .header(UserAgent::new(self.agent.to_owned()))
    // .form(&params.to_owned())
    // .send()
    // .unwrap()
    // .json::<OAuthToken>()
    // .unwrap();
    //
    // if mfa_code.is_none() && res.mfa_required.is_some() &&
    // res.mfa_required.unwrap() { let mfa =
    // &self._get_mfa_code(res.mfa_type.unwrap()); return self.
    // _oauth_login(Some(mfa.to_string())); }
    // else {
    // res.birth = Some(time::get_time().sec);
    // }
    //
    // Some(res)
    // }
    //
    // fn _classic_login(&self, mfa_code: Option<String>) -> Option<PlainAuthToken>
    // { let mut params = HashMap::new();
    // params.insert("username", self.username.as_ref().unwrap());
    // params.insert("password", self.password.as_ref().unwrap());
    //
    // if mfa_code.is_some() {
    // params.insert("mfa_code", mfa_code.as_ref().unwrap());
    // }
    //
    // ("backup_code", backup_code.into())
    // let client = HTTPClient::new();
    //
    // let res = client
    // .post("https://api.robinhood.com/api-token-auth/")
    // .header(UserAgent::new(self.agent.to_owned()))
    // .form(&params.to_owned())
    // .send()
    // .unwrap()
    // .json::<PlainAuthToken>()
    // .unwrap();
    //
    // if mfa_code.is_none() && res.mfa_required.is_some() &&
    // res.mfa_required.unwrap() { let mfa =
    // &self._get_mfa_code(res.mfa_type.unwrap()); return self.
    // _classic_login(Some(mfa.to_string())); }
    //
    // Some(res)
    // }
    //
    // pub fn build(&mut self) -> Result<Client> {
    // let mut headers = Headers::new();
    // headers.set(UserAgent::new(self.agent.to_owned()));
    // let mut authorized = false;
    //
    // if self.username.is_some() && self.username.is_some() {
    // if self.client_string.is_some() {
    // OAuth2
    //
    // let mfa_callback = self.mfa_callback.as_ref();
    // let token = self._oauth_login(None);
    // if token.is_some() {
    // headers.set(Authorization(Bearer {
    // token: token.unwrap().access_token.to_owned().unwrap(),
    // }));
    // }
    // }
    // else {
    // Old skool
    // let token = self._classic_login(None);
    // println!("{:?}", token);
    // headers.set(Authorization(
    // String::from("Token ") + token.unwrap().token.to_owned().unwrap().as_ref(),
    // ));
    // }
    //
    // authorized = true;
    // }
    //
    // println!("{:?}", headers);
    //
    // let client = HTTPClient::builder().default_headers(headers).build()?;
    //
    // Ok(Client {
    // client:     client,
    // authorized: authorized,
    // })
    // }
    //
}

// Conditionally compile the module `test` only when the test-suite is run.
#[cfg(test)]
mod test_order_builder {
    //use super::Order;

    #[test]
    fn order_builder() {
        // assert!(Order::new().build().is_ok());
    }

    #[test]
    //#[should_panic]
    fn client_builder_bad_login() {
        assert!(true);
        //assert!(Order::new().login("username", "password").build().is_ok());
    }
}
