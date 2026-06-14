//! Reusable, composable validation framework driven by keypaths.
//!
//! The core (`FieldError`, `Rule`, `Validator`, and the `rules` combinators) is
//! **domain-agnostic** — reuse it for any payload. Only the `validate_*` functions and
//! the `Validate` impls at the bottom are PAIN.001-specific.

use std::borrow::Cow;

use rust_key_paths::KpType;

use super::keypaths::{
    pain_creation_date_time, pain_initiating_party_id, pain_message_id, pmt_debtor_account_id,
    pmt_debtor_name, pmt_inf_id, pmt_payment_method, pmt_requested_execution_date, tx_amount,
    tx_creditor_account_id, tx_creditor_name, tx_currency, tx_end_to_end_id, tx_instruction_id,
};
use super::model::{CreditTransferTxInfo, Pain001, PaymentInformation};

/// Error message — borrowed for static rules, owned for parameterized ones.
pub type Message = Cow<'static, str>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldError {
    pub path: String,
    pub message: Message,
}

// ── Rule: a single composable field check ──────────────────────────────────

/// A field rule: returns `Some(message)` on failure, `None` when valid.
///
/// Built from closures so rules can be **parameterized** (`max_len(35)`), **lifted**
/// (`optional(..)`), or **composed** — unlike bare `fn` pointers.
pub struct Rule<V> {
    run: Box<dyn Fn(&V) -> Option<Message>>,
}

impl<V> Rule<V> {
    pub fn new(run: impl Fn(&V) -> Option<Message> + 'static) -> Self {
        Self { run: Box::new(run) }
    }

    pub fn check(&self, value: &V) -> Option<Message> {
        (self.run)(value)
    }

    /// Build a rule from a predicate + static failure message.
    pub fn predicate(test: impl Fn(&V) -> bool + 'static, message: &'static str) -> Self {
        Rule::new(move |v| if test(v) { None } else { Some(Cow::Borrowed(message)) })
    }
}

// ── Validator: fluent accumulator over one root ────────────────────────────

/// Accumulates [`FieldError`]s for a single `root`, prefixing every path.
///
/// Chain `field` / `value` / `ensure` / `each` to **accumulate** all errors, or
/// `require` / `require_value` / `must` to **fail fast** — once a mandatory check
/// fails, every later step is skipped. Finish with `finish` / `finish_result` / `first_error`.
pub struct Validator<'r, R> {
    root: &'r R,
    prefix: String,
    errors: Vec<FieldError>,
    /// Set when a mandatory (`require*` / `must`) check fails; skips all later steps.
    aborted: bool,
}

impl<'r, R> Validator<'r, R> {
    pub fn new(root: &'r R) -> Self {
        Self {
            root,
            prefix: String::new(),
            errors: Vec::new(),
            aborted: false,
        }
    }

    pub fn with_prefix(root: &'r R, prefix: impl Into<String>) -> Self {
        Self {
            root,
            prefix: prefix.into(),
            errors: Vec::new(),
            aborted: false,
        }
    }

    fn full_path(&self, path: &str) -> String {
        if self.prefix.is_empty() {
            path.to_string()
        } else {
            format!("{}/{}", self.prefix, path)
        }
    }

    /// Runs `rules` against `value`. Returns `true` if at least one rule failed.
    fn run_rules<V>(&mut self, path: &str, value: &V, rules: &[Rule<V>]) -> bool {
        let mut failed = false;
        for rule in rules {
            if let Some(message) = rule.check(value) {
                failed = true;
                let full = self.full_path(path);
                self.errors.push(FieldError {
                    path: full,
                    message,
                });
            }
        }
        failed
    }

    /// Validate the value reached by `kp`; records `missing` when the path is absent.
    pub fn field<V>(mut self, path: &str, kp: KpType<'static, R, V>, rules: &[Rule<V>]) -> Self {
        if self.aborted {
            return self;
        }
        match kp.get(self.root) {
            Some(value) => {
                self.run_rules(path, value, rules);
            }
            None => {
                let full = self.full_path(path);
                self.errors.push(FieldError {
                    path: full,
                    message: Cow::Borrowed("missing"),
                });
            }
        }
        self
    }

    /// Validate a directly-borrowed value (e.g. an `Option<_>` field) without a keypath.
    pub fn value<V>(mut self, path: &str, value: &V, rules: &[Rule<V>]) -> Self {
        if self.aborted {
            return self;
        }
        self.run_rules(path, value, rules);
        self
    }

    /// Record an error at `path` unless `condition` holds.
    pub fn ensure(mut self, condition: bool, path: &str, message: impl Into<Message>) -> Self {
        if self.aborted {
            return self;
        }
        if !condition {
            let full = self.full_path(path);
            self.errors.push(FieldError {
                path: full,
                message: message.into(),
            });
        }
        self
    }

    /// **Mandatory** keypath field: on failure (or missing) record the error and **abort** —
    /// every later step is skipped so `finish_result` / `first_error` returns immediately.
    pub fn require<V>(mut self, path: &str, kp: KpType<'static, R, V>, rules: &[Rule<V>]) -> Self {
        if self.aborted {
            return self;
        }
        match kp.get(self.root) {
            Some(value) => {
                if self.run_rules(path, value, rules) {
                    self.aborted = true;
                }
            }
            None => {
                let full = self.full_path(path);
                self.errors.push(FieldError {
                    path: full,
                    message: Cow::Borrowed("missing"),
                });
                self.aborted = true;
            }
        }
        self
    }

    /// **Mandatory** borrowed value: abort on first failure.
    #[allow(dead_code)]
    pub fn require_value<V>(mut self, path: &str, value: &V, rules: &[Rule<V>]) -> Self {
        if self.aborted {
            return self;
        }
        if self.run_rules(path, value, rules) {
            self.aborted = true;
        }
        self
    }

    /// **Mandatory** condition: record the error and abort unless `condition` holds.
    pub fn must(mut self, condition: bool, path: &str, message: impl Into<Message>) -> Self {
        if self.aborted {
            return self;
        }
        if !condition {
            let full = self.full_path(path);
            self.errors.push(FieldError {
                path: full,
                message: message.into(),
            });
            self.aborted = true;
        }
        self
    }

    /// Validate each item of a collection with an indexed, prefixed sub-validator.
    pub fn each<T>(
        mut self,
        path: &str,
        items: &[T],
        validate_item: impl Fn(&T, &str) -> Vec<FieldError>,
    ) -> Self {
        if self.aborted {
            return self;
        }
        for (i, item) in items.iter().enumerate() {
            let prefix = self.full_path(&format!("{path}[{i}]"));
            self.errors.extend(validate_item(item, &prefix));
        }
        self
    }

    /// Merge already-prefixed errors (e.g. from cross-field checks).
    pub fn merge(mut self, errors: impl IntoIterator<Item = FieldError>) -> Self {
        if self.aborted {
            return self;
        }
        self.errors.extend(errors);
        self
    }

    /// All accumulated errors.
    pub fn finish(self) -> Vec<FieldError> {
        self.errors
    }

    /// `Ok(())` when clean, else every accumulated error.
    #[allow(dead_code)]
    pub fn finish_result(self) -> Result<(), Vec<FieldError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    /// The first error (the mandatory one when a `require*` / `must` aborted), if any.
    pub fn first_error(mut self) -> Option<FieldError> {
        if self.errors.is_empty() {
            None
        } else {
            Some(self.errors.swap_remove(0))
        }
    }
}

// ── Reusable rule combinators (domain-agnostic) ────────────────────────────

pub mod rules {
    use super::Rule;
    use std::borrow::Cow;

    pub fn required() -> Rule<String> {
        Rule::new(|s: &String| {
            if s.is_empty() {
                Some(Cow::Borrowed("must not be empty"))
            } else {
                None
            }
        })
    }

    pub fn max_len(n: usize) -> Rule<String> {
        Rule::new(move |s: &String| {
            if s.len() > n {
                Some(Cow::Owned(format!("max {n} characters")))
            } else {
                None
            }
        })
    }

    pub fn min_len(n: usize) -> Rule<String> {
        Rule::new(move |s: &String| {
            if s.len() < n {
                Some(Cow::Owned(format!("must be at least {n} characters")))
            } else {
                None
            }
        })
    }

    pub fn len_in(lengths: &'static [usize]) -> Rule<String> {
        Rule::new(move |s: &String| {
            if lengths.contains(&s.len()) {
                None
            } else {
                Some(Cow::Owned(format!("length must be one of {lengths:?}")))
            }
        })
    }

    pub fn one_of(allowed: &'static [&'static str]) -> Rule<String> {
        Rule::new(move |s: &String| {
            if allowed.iter().any(|a| a == s) {
                None
            } else {
                Some(Cow::Owned(format!("must be one of {allowed:?}")))
            }
        })
    }

    pub fn iso_date() -> Rule<String> {
        Rule::predicate(
            |s: &String| {
                s.len() == 10 && s.as_bytes().get(4) == Some(&b'-') && s.as_bytes().get(7) == Some(&b'-')
            },
            "expected YYYY-MM-DD",
        )
    }

    pub fn iso8601_datetime() -> Rule<String> {
        Rule::predicate(
            |s: &String| s.contains('T') && s.len() >= 19,
            "expected ISO 8601 datetime (CreDtTm)",
        )
    }

    pub fn iso_currency() -> Rule<String> {
        Rule::predicate(
            |s: &String| s.len() == 3 && s.chars().all(|c| c.is_ascii_uppercase()),
            "currency must be 3-letter ISO 4217",
        )
    }

    pub fn positive() -> Rule<f64> {
        Rule::predicate(|n: &f64| *n > 0.0, "amount must be > 0")
    }

    /// Lift a `Rule<V>` to `Rule<Option<V>>` — passes when the value is absent.
    pub fn optional<V: 'static>(inner: Rule<V>) -> Rule<Option<V>> {
        Rule::new(move |o: &Option<V>| match o {
            Some(v) => inner.check(v),
            None => None,
        })
    }
}

// ── Shared PAIN rule sets ──────────────────────────────────────────────────

fn account_id_rules() -> [Rule<String>; 1] {
    [rules::required()]
}

fn name_rules() -> [Rule<String>; 2] {
    [rules::required(), rules::min_len(2)]
}

fn optional_bic_rule() -> Rule<Option<String>> {
    rules::optional(rules::len_in(&[8, 11]))
}

// ── PAIN.001 validation (domain-specific) ──────────────────────────────────

pub fn validate_group_header(payload: &Pain001) -> Vec<FieldError> {
    Validator::new(payload)
        .field(
            "GrpHdr/MsgId",
            pain_message_id(),
            &[rules::required(), rules::max_len(35)],
        )
        .field(
            "GrpHdr/CreDtTm",
            pain_creation_date_time(),
            &[rules::required(), rules::iso8601_datetime()],
        )
        .field(
            "GrpHdr/InitgPty/Id",
            pain_initiating_party_id(),
            &[rules::required()],
        )
        .finish()
}

pub fn validate_payment_information(pmt: &PaymentInformation, prefix: &str) -> Vec<FieldError> {
    Validator::with_prefix(pmt, prefix)
        .field("PmtInfId", pmt_inf_id(), &[rules::required(), rules::max_len(35)])
        .field(
            "PmtMtd",
            pmt_payment_method(),
            &[rules::required(), rules::one_of(&["TRF"])],
        )
        .field(
            "ReqdExctnDt",
            pmt_requested_execution_date(),
            &[rules::required(), rules::iso_date()],
        )
        .field("Dbtr/Nm", pmt_debtor_name(), &name_rules())
        .field("DbtrAcct/Id", pmt_debtor_account_id(), &account_id_rules())
        .value("DbtrAgt/BICFI", &pmt.debtor_agent_bic, &[optional_bic_rule()])
        .ensure(
            !pmt.credit_transfer_tx_infos.is_empty(),
            "CdtTrfTxInf",
            "at least one credit transfer required",
        )
        .each(
            "CdtTrfTxInf",
            &pmt.credit_transfer_tx_infos,
            validate_credit_transfer,
        )
        .finish()
}

pub fn validate_credit_transfer(tx: &CreditTransferTxInfo, prefix: &str) -> Vec<FieldError> {
    Validator::with_prefix(tx, prefix)
        .field("PmtId/InstrId", tx_instruction_id(), &[rules::required()])
        .field("PmtId/EndToEndId", tx_end_to_end_id(), &[rules::required()])
        .field("Amt/InstdAmt", tx_amount(), &[rules::positive()])
        .field(
            "Amt/InstdAmt/@Ccy",
            tx_currency(),
            &[rules::required(), rules::iso_currency()],
        )
        .field("Cdtr/Nm", tx_creditor_name(), &name_rules())
        .field("CdtrAcct/Id", tx_creditor_account_id(), &account_id_rules())
        .value("CdtrAgt/BICFI", &tx.creditor_agent_bic, &[optional_bic_rule()])
        .value(
            "RmtInf/Ustrd",
            &tx.remittance_info,
            &[rules::optional(rules::max_len(140))],
        )
        .finish()
}

/// Cross-field: `GrpHdr/NbOfTxs` must equal total `CdtTrfTxInf` count.
pub fn validate_transaction_count(payload: &Pain001) -> Option<FieldError> {
    let expected = payload
        .payment_informations
        .iter()
        .map(|p| p.credit_transfer_tx_infos.len() as u32)
        .sum::<u32>();
    (payload.group_header.number_of_transactions != expected).then(|| FieldError {
        path: "GrpHdr/NbOfTxs".to_string(),
        message: Cow::Borrowed("must equal total CdtTrfTxInf count"),
    })
}

/// Cross-field: `GrpHdr/CtrlSum` must match sum of instructed amounts when present.
pub fn validate_control_sum(payload: &Pain001) -> Option<FieldError> {
    let control_sum = payload.group_header.control_sum?;
    let total: f64 = payload
        .payment_informations
        .iter()
        .flat_map(|p| p.credit_transfer_tx_infos.iter())
        .map(|tx| tx.amount)
        .sum();
    ((control_sum - total).abs() >= 0.001).then(|| FieldError {
        path: "GrpHdr/CtrlSum".to_string(),
        message: Cow::Borrowed("must equal sum of Amt/InstdAmt"),
    })
}

/// Mandatory pre-check — **fails fast**: returns the first missing/invalid mandatory
/// field and stops, before any full field-by-field accumulation runs.
///
/// These are the structural minimums an ISO 20022 PAIN.001 needs to be processable at all.
pub fn validate_mandatory(payload: &Pain001) -> Result<(), FieldError> {
    match Validator::new(payload)
        .require("GrpHdr/MsgId", pain_message_id(), &[rules::required()])
        .require("GrpHdr/CreDtTm", pain_creation_date_time(), &[rules::required()])
        .require(
            "GrpHdr/InitgPty/Id",
            pain_initiating_party_id(),
            &[rules::required()],
        )
        .must(
            !payload.payment_informations.is_empty(),
            "PmtInf",
            "at least one payment information block required",
        )
        .first_error()
    {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// Full payload validation — reusable from reducer, tests, or API ingress.
///
/// Runs the mandatory pre-check first: if it fails, returns **only** that single error
/// immediately; otherwise accumulates every field error.
pub fn validate_pain001(payload: &Pain001) -> Vec<FieldError> {
    if let Err(err) = validate_mandatory(payload) {
        return vec![err];
    }
    Validator::new(payload)
        .merge(validate_group_header(payload))
        .each(
            "PmtInf",
            &payload.payment_informations,
            validate_payment_information,
        )
        .merge(validate_transaction_count(payload))
        .merge(validate_control_sum(payload))
        .finish()
}

/// Anything that can validate itself.
pub trait Validate {
    /// Accumulate all field errors (mandatory pre-check still short-circuits).
    fn validate(&self) -> Vec<FieldError>;

    /// Fail-fast: return the first mandatory error immediately, or `Ok(())`.
    fn validate_mandatory(&self) -> Result<(), FieldError>;
}

impl Validate for Pain001 {
    fn validate(&self) -> Vec<FieldError> {
        validate_pain001(self)
    }

    fn validate_mandatory(&self) -> Result<(), FieldError> {
        validate_mandatory(self)
    }
}
