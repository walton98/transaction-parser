use serde::Deserialize;

pub type ClientId = u16;
pub type TxId = u32;
pub type Amount = f64;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<Amount>,
}

pub struct TransactionParser<R> {
    inner: csv::Reader<R>,
}

impl TransactionParser<std::fs::File> {
    pub fn new(filename: &str) -> Result<Self, csv::Error> {
        Ok(Self {
            inner: csv::ReaderBuilder::new()
                .trim(csv::Trim::All)
                .from_path(filename)?,
        })
    }
}

impl<R: std::io::Read> TransactionParser<R> {
    pub fn iter(&mut self) -> impl Iterator<Item = Result<Transaction, csv::Error>> + '_ {
        self.inner.deserialize()
    }
}
