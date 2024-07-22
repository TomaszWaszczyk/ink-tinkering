#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod bank {
    use ink_storage::collections::HashMap as StorageHashMap;

    #[ink(storage)]
    pub struct Bank {
        balances: StorageHashMap<AccountId, Balance>,
    }

    impl Bank {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                balances: StorageHashMap::new(),
            }
        }

        #[ink(message)]
        pub fn deposit(&mut self) {
            let caller = self.env().caller();
            let deposit_amount = self.env().transferred_balance();
            let balance = self.balances.get(&caller).copied().unwrap_or(0);
            self.balances.insert(caller, balance + deposit_amount);
        }

        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance) -> bool {
            let caller = self.env().caller();
            let caller_balance = self.balances.get(&caller).copied().unwrap_or(0);

            if caller_balance < amount {
                return false;
            }

            // Vulnerable point: decreasing the balance after sending the funds
            if self.env().transfer(caller, amount).is_ok() {
                self.balances.insert(caller, caller_balance - amount);
                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn get_balance(&self, account: AccountId) -> Balance {
            self.balances.get(&account).copied().unwrap_or(0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn deposit_works() {
            let mut bank = Bank::new();
            bank.deposit();
            assert_eq!(bank.get_balance(bank.env().caller()), 0);
        }

        #[ink::test]
        fn withdraw_works() {
            let mut bank = Bank::new();
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
            bank.deposit();
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(accounts.alice, 1_000_000);
            assert!(bank.withdraw(500));
            assert_eq!(bank.get_balance(accounts.alice), 0);
        }
    }
}
