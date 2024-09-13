use std::collections::HashMap;

use crate::{
    transaction_parser::{Amount, ClientId, Transaction, TransactionType, TxId},
    writer::AccountSummary,
};

#[derive(Debug, Default)]
struct Account {
    available: Amount,
    held: HashMap<TxId, Amount>,
    // NOTE: I'm assuming only deposits can be charged back
    deposit_amounts: HashMap<TxId, Amount>,
    locked: bool,
}

impl Account {
    pub fn held_amount(&self) -> Amount {
        self.held.values().sum()
    }

    pub fn total(&self) -> Amount {
        self.available + self.held_amount()
    }

    fn deposit(&mut self, amount: Amount, tx: TxId) {
        self.available += amount;
        self.deposit_amounts.insert(tx, amount);
    }

    fn withdraw(&mut self, amount: Amount) {
        if amount <= self.available {
            self.available -= amount
        }
    }

    fn dispute(&mut self, tx: TxId) {
        if let Some(amount) = self.deposit_amounts.get(&tx) {
            self.available -= amount;
            self.held.insert(tx, *amount);
        }
    }

    fn resolve(&mut self, tx: TxId) {
        if let Some(amount) = self.held.remove(&tx) {
            self.available += amount;
        }
    }

    fn chargeback(&mut self, tx: TxId) {
        if self.held.remove(&tx).is_some() {
            self.locked = true;
        }
    }
}

#[derive(Default)]
pub struct AccountManager {
    accounts: HashMap<ClientId, Account>,
}

impl AccountManager {
    pub fn process_row(&mut self, row: Transaction) {
        let account = self.get_account(row.client);
        if !account.locked {
            match row.r#type {
                TransactionType::Deposit => {
                    account.deposit(row.amount.expect("Deposits require an amount"), row.tx)
                }
                TransactionType::Withdrawal => {
                    account.withdraw(row.amount.expect("Withdrawals require an amount"))
                }
                TransactionType::Dispute => account.dispute(row.tx),
                TransactionType::Resolve => account.resolve(row.tx),
                TransactionType::Chargeback => account.chargeback(row.tx),
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = AccountSummary> + '_ {
        self.accounts
            .iter()
            .map(|(&client, account)| AccountSummary {
                client,
                available: account.available,
                held: account.held_amount(),
                total: account.total(),
                locked: account.locked,
            })
    }

    fn get_account(&mut self, client: ClientId) -> &mut Account {
        self.accounts.entry(client).or_default()
    }
}

#[cfg(test)]
mod tests {
    use TransactionType::*;

    use super::*;

    fn create_processor() -> AccountManager {
        AccountManager::default()
    }

    fn compare(
        processor: &AccountManager,
        expected_data: &[(ClientId, Amount, Amount, bool)],
    ) -> bool {
        let data: HashMap<ClientId, AccountSummary> = processor
            .iter()
            .map(|summary| (summary.client, summary))
            .collect();
        let expected_data = expected_data
            .iter()
            .map(|&(client, available, held, locked)| {
                let summary = AccountSummary {
                    client,
                    available,
                    held,
                    total: available + held,
                    locked,
                };
                (client, summary)
            })
            .collect();
        data == expected_data
    }

    fn apply_actions(
        processor: &mut AccountManager,
        actions: Vec<(TransactionType, ClientId, TxId, Option<Amount>)>,
    ) {
        for (r#type, client, tx, amount) in actions {
            processor.process_row(Transaction {
                r#type,
                client,
                tx,
                amount,
            })
        }
    }

    #[test]
    fn test_deposit() {
        let mut processor = create_processor();

        let actions = vec![(Deposit, 1, 1, Some(1.0)), (Deposit, 1, 1, Some(2.0))];
        apply_actions(&mut processor, actions);

        assert!(compare(&processor, &[(1, 3.0, 0.0, false)]));
    }

    #[test]
    fn test_withdrawal() {
        let mut processor = create_processor();

        let actions = vec![
            (Deposit, 1, 1, Some(3.0)),
            (Withdrawal, 1, 2, Some(1.0)),
            (Withdrawal, 1, 3, Some(3.0)), // rejected
        ];
        apply_actions(&mut processor, actions);

        assert!(compare(&processor, &[(1, 2.0, 0.0, false)]));
    }

    #[test]
    fn test_dispute() {
        let mut processor = create_processor();

        let actions = vec![
            (Deposit, 1, 1, Some(3.0)),
            (Dispute, 1, 1, None),
            (Dispute, 1, 2, None), // rejected
        ];
        apply_actions(&mut processor, actions);

        assert!(compare(&processor, &[(1, 0.0, 3.0, false)]));
    }

    #[test]
    fn test_resolve() {
        let mut processor = create_processor();

        let actions = vec![
            (Deposit, 1, 1, Some(3.0)),
            (Dispute, 1, 1, None),
            (Resolve, 1, 1, None),
            (Resolve, 1, 2, None), // rejected
        ];
        apply_actions(&mut processor, actions);

        assert!(compare(&processor, &[(1, 3.0, 0.0, false)]));
    }

    #[test]
    fn test_chargeback() {
        let mut processor = create_processor();

        let actions = vec![
            (Deposit, 1, 1, Some(3.0)),
            (Dispute, 1, 1, None),
            (Chargeback, 1, 1, None),
        ];
        apply_actions(&mut processor, actions);

        assert!(compare(&processor, &[(1, 0.0, 0.0, true)]));
    }
}
