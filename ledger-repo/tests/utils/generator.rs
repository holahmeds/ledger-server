use chrono::NaiveDate;
use fake::faker::lorem::en::{Sentence, Words};
use fake::faker::name::en::Name;
use fake::{Fake, Faker};
use ledger_repo::transaction_repo::NewTransaction;
use ledger_repo::transaction_template_repo::NewTransactionTemplate;
use rand::seq::SliceRandom;
use rust_decimal::Decimal;
use std::collections::HashSet;

trait Generator<T> {
    fn gen(&mut self) -> T;
}

struct Predefined<T> {
    values: Vec<T>,
    current_pos: usize,
}

impl<T> Predefined<T> {
    fn boxed(values: Vec<T>) -> Box<Predefined<T>> {
        Box::new(Predefined {
            values,
            current_pos: 0,
        })
    }
}

impl<T: Clone> Generator<T> for Predefined<T> {
    fn gen(&mut self) -> T {
        let v = self.values[self.current_pos].clone();
        self.current_pos += 1;
        v
    }
}

struct RandomSample<T> {
    values: Vec<T>,
}

impl<T> RandomSample<T> {
    fn boxed(values: Vec<T>) -> Box<RandomSample<T>> {
        Box::new(RandomSample { values })
    }
}

impl<T: Clone> Generator<T> for RandomSample<T> {
    fn gen(&mut self) -> T {
        self.values.choose(&mut rand::thread_rng()).unwrap().clone()
    }
}

struct FakeGenerator<F: Fake> {
    fake: F,
}

impl<F: Fake> FakeGenerator<F> {
    fn boxed(fake: F) -> Box<FakeGenerator<F>> {
        Box::new(FakeGenerator { fake })
    }
}

impl<T: fake::Dummy<F>, F> Generator<T> for FakeGenerator<F> {
    fn gen(&mut self) -> T {
        self.fake.fake()
    }
}

struct FakeAmount;

impl Generator<Decimal> for FakeAmount {
    fn gen(&mut self) -> Decimal {
        Decimal::from(Faker.fake::<i32>())
    }
}

impl Generator<Option<Decimal>> for FakeAmount {
    fn gen(&mut self) -> Option<Decimal> {
        Some(Decimal::from(Faker.fake::<i32>()))
    }
}

struct FakeTags;

impl Generator<HashSet<String>> for FakeTags {
    fn gen(&mut self) -> HashSet<String> {
        let tags: Vec<String> = Words(1..3).fake();
        HashSet::from_iter(tags)
    }
}

#[allow(dead_code)]
pub struct NewTransactionGenerator {
    cat_gen: Box<dyn Generator<String>>,
    tran_gen: Box<dyn Generator<Option<String>>>,
    note_gen: Box<dyn Generator<Option<String>>>,
    date_gen: Box<dyn Generator<NaiveDate>>,
    amnt_gen: Box<dyn Generator<Decimal>>,
    tag_gen: Box<dyn Generator<HashSet<String>>>,
}

#[allow(dead_code)]
impl NewTransactionGenerator {
    pub fn with_categories(mut self, categories: Vec<&str>) -> NewTransactionGenerator {
        let categories: Vec<String> = categories.into_iter().map(|s| s.to_string()).collect();
        self.cat_gen = Predefined::boxed(categories);
        self
    }

    pub fn with_transactees(mut self, transactees: Vec<&str>) -> NewTransactionGenerator {
        let transactees = transactees
            .into_iter()
            .map(|t| Some(t.to_string()))
            .collect();
        self.tran_gen = Predefined::boxed(transactees);
        self
    }

    pub fn with_dates(mut self, dates: Vec<NaiveDate>) -> NewTransactionGenerator {
        self.date_gen = Predefined::boxed(dates);
        self
    }

    pub fn with_amounts(mut self, amounts: Vec<Decimal>) -> NewTransactionGenerator {
        self.amnt_gen = Predefined::boxed(amounts);
        self
    }

    pub fn with_tags(mut self, tags: Vec<HashSet<String>>) -> NewTransactionGenerator {
        self.tag_gen = Predefined::boxed(tags);
        self
    }

    pub fn generate(&mut self) -> NewTransaction {
        let new_transaction = NewTransaction::new(
            self.cat_gen.gen(),
            self.tran_gen.gen(),
            self.note_gen.gen(),
            self.date_gen.gen(),
            self.amnt_gen.gen(),
            self.tag_gen.gen(),
        );
        new_transaction
    }

    pub fn generate_many(&mut self, count: usize) -> Vec<NewTransaction> {
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(self.generate())
        }
        vec
    }
}

impl Default for NewTransactionGenerator {
    fn default() -> Self {
        NewTransactionGenerator {
            cat_gen: RandomSample::boxed(vec![
                "Misc".to_string(),
                "Groceries".to_string(),
                "Eating Out".to_string(),
                "Transportation".to_string(),
            ]),
            tran_gen: FakeGenerator::boxed(Name()),
            note_gen: FakeGenerator::boxed(Sentence(5..10)),
            date_gen: FakeGenerator::boxed(Faker),
            amnt_gen: Box::new(FakeAmount),
            tag_gen: Box::new(FakeTags),
        }
    }
}

#[allow(dead_code)]
pub struct NewTemplateGenerator {
    cat_gen: Box<dyn Generator<Option<String>>>,
    tran_gen: Box<dyn Generator<Option<String>>>,
    note_gen: Box<dyn Generator<Option<String>>>,
    amnt_gen: Box<dyn Generator<Option<Decimal>>>,
    tag_gen: Box<dyn Generator<HashSet<String>>>,
}

#[allow(dead_code)]
impl NewTemplateGenerator {
    pub fn generate(&mut self) -> NewTransactionTemplate {
        NewTransactionTemplate::new(
            self.cat_gen.gen(),
            self.tran_gen.gen(),
            self.amnt_gen.gen(),
            self.note_gen.gen(),
            self.tag_gen.gen(),
        )
    }
}

impl Default for NewTemplateGenerator {
    fn default() -> Self {
        NewTemplateGenerator {
            cat_gen: RandomSample::boxed(vec![
                None,
                Some("Misc".to_string()),
                Some("Groceries".to_string()),
                Some("Eating Out".to_string()),
                Some("Transportation".to_string()),
            ]),
            tran_gen: FakeGenerator::boxed(Name()),
            note_gen: FakeGenerator::boxed(Sentence(5..10)),
            amnt_gen: Box::new(FakeAmount),
            tag_gen: Box::new(FakeTags),
        }
    }
}
