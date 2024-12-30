pub mod test_user;

use fake::faker::lorem::en::Words;
use fake::{Fake, Faker};
use rand::seq::SliceRandom;
use rust_decimal::Decimal;
use std::collections::HashSet;

pub trait Generator<T> {
    fn gen(&mut self) -> T;
}

pub struct Predefined<T> {
    values: Vec<T>,
    current_pos: usize,
}

impl<T> Predefined<T> {
    pub fn boxed(values: Vec<T>) -> Box<Predefined<T>> {
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

pub struct RandomSample<T> {
    values: Vec<T>,
}

impl<T> RandomSample<T> {
    pub fn boxed(values: Vec<T>) -> Box<RandomSample<T>> {
        Box::new(RandomSample { values })
    }
}

impl<T: Clone> Generator<T> for RandomSample<T> {
    fn gen(&mut self) -> T {
        self.values.choose(&mut rand::thread_rng()).unwrap().clone()
    }
}

pub struct FakeGenerator<F: Fake> {
    fake: F,
}

impl<F: Fake> FakeGenerator<F> {
    pub fn boxed(fake: F) -> Box<FakeGenerator<F>> {
        Box::new(FakeGenerator { fake })
    }
}

impl<T: fake::Dummy<F>, F> Generator<T> for FakeGenerator<F> {
    fn gen(&mut self) -> T {
        self.fake.fake()
    }
}

pub struct FakeAmount;

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

pub struct FakeTags;

impl Generator<HashSet<String>> for FakeTags {
    fn gen(&mut self) -> HashSet<String> {
        let tags: Vec<String> = Words(1..3).fake();
        HashSet::from_iter(tags)
    }
}
