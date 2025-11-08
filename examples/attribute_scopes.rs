use key_paths_core::KeyPaths;
use key_paths_derive::Keypaths;

#[derive(Clone, Debug, Keypaths)]
#[Readable]
struct Account {
    // Inherits the struct-level #[Readable] scope; only readable methods are emitted.
    nickname: Option<String>,
    // Field-level attribute overrides the default, enabling writable accessors.
    #[Writable]
    balance: i64,
    // Owned scope generates owned and failable owned accessors.
    #[Owned]
    recovery_token: Option<String>,
}

fn main() {
    let mut account = Account {
        nickname: Some("ace".to_string()),
        balance: 1_000,
        recovery_token: Some("token-123".to_string()),
    };

    let nickname_fr: KeyPaths<Account, String> = Account::nickname_fr();
    let balance_w: KeyPaths<Account, i64> = Account::balance_w();
    let recovery_token_fo: KeyPaths<Account, String> = Account::recovery_token_fo();

    let nickname_value = nickname_fr.get(&account);
    println!("nickname (readable): {:?}", nickname_value);

    if let Some(balance_ref) = balance_w.get_mut(&mut account) {
        *balance_ref += 500;
    }
    println!("balance after writable update: {}", account.balance);

    let owned_token = recovery_token_fo.get_failable_owned(account.clone());
    println!("recovery token (owned): {:?}", owned_token);

    // Uncommenting the next line would fail to compile because `nickname` only has readable methods.
    // let _ = Account::nickname_w();
}

