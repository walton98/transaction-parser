use std::{collections::HashMap, fs::File};

use transaction_manager::{transaction_parser::ClientId, writer::AccountSummary};

fn parse_output(output: impl std::io::Read) -> HashMap<ClientId, AccountSummary> {
    csv::Reader::from_reader(output)
        .deserialize()
        .map(|client| {
            let client: AccountSummary = client.unwrap();
            (client.client, client)
        })
        .collect()
}

fn compare_to_expected(input_filename: &str, output_filename: &str) {
    let mut buf = Vec::<u8>::new();
    transaction_manager::run(input_filename, &mut buf).unwrap();
    let data = parse_output(&buf[..]);
    let expected_data = parse_output(File::open(output_filename).unwrap());
    assert_eq!(data, expected_data);
}

#[test]
fn test_parse() {
    compare_to_expected("test_data/input.csv", "test_data/output.csv");
}
