use chrono::NaiveDate;
use fake::faker::lorem::en::{Sentence, Words};
use fake::faker::name::en::Name;
use fake::{Fake, Faker};
use ledger_repo::transaction_repo::NewTransaction;
use rand::seq::SliceRandom;
use rust_decimal::Decimal;
use std::collections::HashSet;
use std::default::Default;

const CATEGORIES: [&str; 4] = ["Misc", "Groceries", "Eating Out", "Transporation"];

struct NewTransactionGenerator {
    cat_gen: Box<dyn FnOnce() -> String>,
    tran_gen: Box<dyn FnOnce() -> Option<String>>,
    note_gen: Box<dyn FnOnce() -> Option<String>>,
    date_gen: Box<dyn FnOnce() -> NaiveDate>,
    amnt_gen: Box<dyn FnOnce() -> Decimal>,
    tag_gen: Box<dyn FnOnce() -> HashSet<String>>,
}

impl NewTransactionGenerator {
    fn generate(self) -> NewTransaction {
        let new_transaction = NewTransaction::new(
            (self.cat_gen)(),
            (self.tran_gen)(),
            (self.note_gen)(),
            (self.date_gen)(),
            (self.amnt_gen)(),
            (self.tag_gen)(),
        );
        new_transaction
    }
}

impl Default for NewTransactionGenerator {
    fn default() -> Self {
        NewTransactionGenerator {
            cat_gen: Box::new(|| {
                CATEGORIES
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .to_string()
            }),
            tran_gen: Box::new(|| Name().fake()),
            note_gen: Box::new(|| Sentence(5..10).fake()),
            date_gen: Box::new(|| Faker.fake()),
            amnt_gen: Box::new(|| Decimal::from(Faker.fake::<i32>())),
            tag_gen: Box::new(|| {
                let tags: Vec<String> = Words(1..3).fake();
                HashSet::from_iter(tags)
            }),
        }
    }
}

pub fn generate_new_transaction() -> NewTransaction {
    let generator = NewTransactionGenerator::default();
    generator.generate()
}

pub fn generate_new_transaction_with_tags(tags: HashSet<String>) -> NewTransaction {
    let generator = NewTransactionGenerator {
        tag_gen: Box::new(|| tags),
        ..Default::default()
    };
    generator.generate()
}

pub fn generate_new_transaction_with_category(category: String) -> NewTransaction {
    let generator = NewTransactionGenerator {
        cat_gen: Box::new(|| category),
        ..Default::default()
    };
    generator.generate()
}

pub fn generate_new_transaction_with_transactee(transactee: String) -> NewTransaction {
    let generator = NewTransactionGenerator {
        tran_gen: Box::new(|| Some(transactee)),
        ..Default::default()
    };
    generator.generate()
}

pub fn generate_new_transaction_with_date(date: NaiveDate) -> NewTransaction {
    let generator = NewTransactionGenerator {
        date_gen: Box::new(move || date),
        ..Default::default()
    };
    generator.generate()
}

pub fn generate_new_transaction_with_amount(amount: Decimal) -> NewTransaction {
    let generator = NewTransactionGenerator {
        amnt_gen: Box::new(move || amount),
        ..Default::default()
    };
    generator.generate()
}

pub fn generate_new_transaction_with_date_and_amount(
    date: NaiveDate,
    amount: Decimal,
) -> NewTransaction {
    let generator = NewTransactionGenerator {
        date_gen: Box::new(move || date),
        amnt_gen: Box::new(move || amount),
        ..Default::default()
    };
    generator.generate()
}

pub fn generate_new_transaction_with_category_and_transactee(
    category: String,
    transactee: String,
) -> NewTransaction {
    let generator = NewTransactionGenerator {
        cat_gen: Box::new(move || category),
        tran_gen: Box::new(move || Some(transactee)),
        ..Default::default()
    };
    generator.generate()
}
