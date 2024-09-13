pub mod account_manager;
pub mod transaction_parser;
pub mod writer;

use crate::{
    account_manager::AccountManager, transaction_parser::TransactionParser, writer::Writer,
};

pub fn process(
    mut parser: TransactionParser<impl std::io::Read>,
    processor: &mut AccountManager,
) -> Result<(), csv::Error> {
    for row in parser.iter() {
        processor.process_row(row?);
    }
    Ok(())
}

pub fn write(
    mut writer: Writer<impl std::io::Write>,
    processor: &mut AccountManager,
) -> Result<(), csv::Error> {
    for account_summary in processor.iter() {
        writer.write(account_summary)?;
    }
    Ok(())
}

pub fn run(filename: &str, output: impl std::io::Write) -> Result<(), csv::Error> {
    let mut processor = AccountManager::default();
    let parser = TransactionParser::new(filename)?;

    process(parser, &mut processor)?;

    let writer = Writer::new(output);
    write(writer, &mut processor)?;
    Ok(())
}
