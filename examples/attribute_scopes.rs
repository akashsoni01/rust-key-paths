use key_paths_derive::Kp;

#[derive(Clone, Debug, Kp)]
struct Account {
    // Inherits the struct-level #[Readable] scope; only readable methods are emitted.
    nickname: Option<String>,
    // Field-level attribute overrides the default, enabling writable accessors.
    balance: i64,
    // Failable readable for Option fields (inherits struct-level #[Readable]).
    recovery_token: Option<String>,
}

fn main() {
    let mut account = Account {
        nickname: Some("ace".to_string()),
        balance: 1_000,
        recovery_token: Some("token-123".to_string()),
    };

    let nickname_fr = Account::nickname();
    let balance_w = Account::balance();
    let recovery_token_fr = Account::recovery_token();

    let nickname_value = nickname_fr.get(&account);
    println!("nickname (readable): {:?}", nickname_value);

    if let Some(balance_ref) = balance_w.get_mut(&mut account)
    {
        *balance_ref += 500;
    }
    println!("balance after writable update: {}", account.balance);

    // Note: The new rust-keypaths API doesn't support owned keypaths.
    // For Option fields, use OptionalKeyPath and get() to access the value.
    // If you need an owned value, clone it after getting the reference.
    if let Some(token) = recovery_token_fr.get(&account) {
        let owned_token = token.clone();
        println!("recovery token (owned): {:?}", owned_token);
    }

    // Uncommenting the next line would fail to compile because `nickname` only has readable methods.
    // let _ = Account::nickname();
}
