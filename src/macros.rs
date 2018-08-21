#[macro_export]
macro_rules! iter_builder {
    ($list_name:ident => $item_name:ident as $data_name:ident, $url:tt {
        $(  $(#[$meta:meta])*
            $attr_name:ident : $attr_type:ty = $attr_default:expr ),*
    })
    => {
        /*#[derive(Debug, Clone)]
        pub struct $item_name {
            pub data: $data_name,
        }*/
/*
        #[derive(Debug)]
        pub struct $data_name {
            $( $attr_name : $attr_type ),*
        }

        #[derive(Debug, Clone)]
        pub struct $_name<'a> {
            pub data: $data_name,
            args: Vec<&'a str>,
            $( $attr_name : Option<$attr_type> ),*
        }

*/
// Could capture more fields here if needed
#[serde(deny_unknown_fields)] // Debugging!
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct $data_name {
    $(  $(#[$meta])* // Can rename fields because rust is retarded
        $attr_name : $attr_type ),*
}

#[derive(Debug, Clone)]
pub struct $item_name {
    pub data: $data_name,
}

impl $item_name {
    /// other operations
    pub fn new(data: $data_name) -> Self {
        $item_name { data: data }
    }

    $(
    pub fn $attr_name(&self) -> $attr_type { // TODO: Renamed fields will need to be handled properly
        //$(#[$meta])*
        self.data.$attr_name.to_owned()
    }
    )*
}

#[derive(Debug, Clone)]
pub struct $list_name {
    pub results: <Vec<$data_name> as IntoIterator>::IntoIter,
    pub next: Option<String>,
    pub client: HTTPClient,
}

impl $list_name {
    /// other operations
    pub fn new_with_client(ref mut client : HTTPClient) -> Self {
        $list_name {
            results: vec![].into_iter(),
            next: Some($url.to_owned()),
            client: client.to_owned(),
        }
    }

    pub fn set_next(&mut self, url: String) -> Self {
        self.next = Some(url.to_owned());
        self.to_owned()
    }

    fn try_next(&mut self) -> Result<Option<$item_name>> {
        // If the previous page has a Instrument that hasn't been looked at.
        if let Some(dep) = self.results.next() {
            return Ok(Some($item_name::new(dep)));
        }

        if self.next.is_none() {
            return Ok(None);
        }
        let url = self.next.clone();

        let response = self.client
            .get(url.as_ref().map(String::as_str).unwrap())
            .send()?
            .json::<PaginatedApiResponse<$data_name>>()?;
        self.results = response.results.into_iter();
        self.next = response.next;
        Ok(Some($item_name::new(self.results.next().unwrap())))
        //Ok(self.results.next())
    }
}

impl Iterator for $list_name {
    type Item = Result<$item_name>;

    fn next(&mut self) -> Option<Self::Item> {
        // Some juggling required here because `try_next` returns a result
        // containing an option, while `next` is supposed to return an option
        // containing a result.
        match self.try_next() {
            Ok(Some(dep)) => Some(Ok(dep)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

}
}
