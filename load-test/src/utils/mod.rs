use rand::Rng;

pub fn random_client_id() -> i32 {
    rand::rng().random_range(1..=5)
}

pub fn random_transaction_value() -> i32 {
    rand::rng().random_range(1..=10000)
}

pub fn random_description() -> String {
    rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}
