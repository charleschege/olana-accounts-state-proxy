use std::{
    borrow::Cow,
    fmt,
    time::{Duration, Instant},
};
use tabled::Tabled;

#[derive(Tabled)]
pub struct PostgresTimer<'a> {
    test: Cow<'a, str>,
    postgres: Cow<'a, str>,
    rpc_response: Cow<'a, str>,
}

impl<'a> PostgresTimer<'a> {
    pub fn new() -> Self {
        Self {
            test: Cow::default(),
            postgres: Cow::default(),
            rpc_response: Cow::default(),
        }
    }

    pub fn add_test_name(mut self, test: &'a str) -> Self {
        self.test = Cow::Borrowed(test);

        self
    }

    pub fn postgres_exec_time(mut self, postgres: &'a str) -> Self {
        self.postgres = Cow::Borrowed(postgres);

        self
    }

    pub fn rpc_exec_time(mut self, rpc_response: &'a str) -> Self {
        self.rpc_response = Cow::Borrowed(rpc_response);

        self
    }

    pub fn build(self) -> Self {
        self
    }

    pub fn with_timer(&self, function_to_time: fn()) -> Duration {
        let now = Instant::now();

        function_to_time();

        now.elapsed()
    }
}

impl<'a> Default for PostgresTimer<'a> {
    fn default() -> Self {
        PostgresTimer::new()
    }
}
