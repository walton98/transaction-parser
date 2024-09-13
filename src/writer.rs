use serde::{Deserialize, Serialize};

use crate::transaction_parser::{Amount, ClientId};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AccountSummary {
    pub client: ClientId,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

pub struct Writer<R: std::io::Write> {
    writer: csv::Writer<R>,
}

impl<R: std::io::Write> Writer<R> {
    pub fn new(writer: R) -> Self {
        Self {
            writer: csv::Writer::from_writer(writer),
        }
    }
}

impl<R: std::io::Write> Writer<R> {
    pub fn write(&mut self, account: AccountSummary) -> Result<(), csv::Error> {
        self.writer.serialize(account)
    }
}
