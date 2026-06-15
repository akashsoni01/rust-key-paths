//! Composed keypaths — reusable with [`super::validation::Validator::field`].

use key_paths_core::KpTrait;
use rust_key_paths::Kp;

use super::model::{CreditTransferTxInfo, Pain001, PaymentInformation};

type PainStringKp = Kp<
    Pain001,
    String,
    &'static Pain001,
    &'static String,
    &'static mut Pain001,
    &'static mut String,
    for<'b> fn(&'b Pain001) -> Option<&'b String>,
    for<'b> fn(&'b mut Pain001) -> Option<&'b mut String>,
>;

type PmtStringKp = Kp<
    PaymentInformation,
    String,
    &'static PaymentInformation,
    &'static String,
    &'static mut PaymentInformation,
    &'static mut String,
    for<'b> fn(&'b PaymentInformation) -> Option<&'b String>,
    for<'b> fn(&'b mut PaymentInformation) -> Option<&'b mut String>,
>;

type TxStringKp = Kp<
    CreditTransferTxInfo,
    String,
    &'static CreditTransferTxInfo,
    &'static String,
    &'static mut CreditTransferTxInfo,
    &'static mut String,
    for<'b> fn(&'b CreditTransferTxInfo) -> Option<&'b String>,
    for<'b> fn(&'b mut CreditTransferTxInfo) -> Option<&'b mut String>,
>;

type TxAmountKp = Kp<
    CreditTransferTxInfo,
    f64,
    &'static CreditTransferTxInfo,
    &'static f64,
    &'static mut CreditTransferTxInfo,
    &'static mut f64,
    for<'b> fn(&'b CreditTransferTxInfo) -> Option<&'b f64>,
    for<'b> fn(&'b mut CreditTransferTxInfo) -> Option<&'b mut f64>,
>;

pub fn pain_message_id() -> PainStringKp {
    fn get(p: &Pain001) -> Option<&String> {
        Some(&p.group_header.message_id)
    }
    fn set(p: &mut Pain001) -> Option<&mut String> {
        Some(&mut p.group_header.message_id)
    }
    Kp::new(get, set)
}

pub fn pain_creation_date_time() -> PainStringKp {
    fn get(p: &Pain001) -> Option<&String> {
        Some(&p.group_header.creation_date_time)
    }
    fn set(p: &mut Pain001) -> Option<&mut String> {
        Some(&mut p.group_header.creation_date_time)
    }
    Kp::new(get, set)
}

pub fn pain_initiating_party_id() -> PainStringKp {
    fn get(p: &Pain001) -> Option<&String> {
        Some(&p.group_header.initiating_party_id)
    }
    fn set(p: &mut Pain001) -> Option<&mut String> {
        Some(&mut p.group_header.initiating_party_id)
    }
    Kp::new(get, set)
}

pub fn pmt_inf_id() -> PmtStringKp {
    fn get(p: &PaymentInformation) -> Option<&String> {
        Some(&p.pmt_inf_id)
    }
    fn set(p: &mut PaymentInformation) -> Option<&mut String> {
        Some(&mut p.pmt_inf_id)
    }
    Kp::new(get, set)
}

pub fn pmt_payment_method() -> PmtStringKp {
    fn get(p: &PaymentInformation) -> Option<&String> {
        Some(&p.payment_method)
    }
    fn set(p: &mut PaymentInformation) -> Option<&mut String> {
        Some(&mut p.payment_method)
    }
    Kp::new(get, set)
}

pub fn pmt_requested_execution_date() -> PmtStringKp {
    fn get(p: &PaymentInformation) -> Option<&String> {
        Some(&p.requested_execution_date)
    }
    fn set(p: &mut PaymentInformation) -> Option<&mut String> {
        Some(&mut p.requested_execution_date)
    }
    Kp::new(get, set)
}

pub fn pmt_debtor_name() -> PmtStringKp {
    fn get(p: &PaymentInformation) -> Option<&String> {
        Some(&p.debtor_name)
    }
    fn set(p: &mut PaymentInformation) -> Option<&mut String> {
        Some(&mut p.debtor_name)
    }
    Kp::new(get, set)
}

pub fn pmt_debtor_account_id() -> PmtStringKp {
    fn get(p: &PaymentInformation) -> Option<&String> {
        Some(&p.debtor_account_id)
    }
    fn set(p: &mut PaymentInformation) -> Option<&mut String> {
        Some(&mut p.debtor_account_id)
    }
    Kp::new(get, set)
}

pub fn tx_instruction_id() -> TxStringKp {
    fn get(t: &CreditTransferTxInfo) -> Option<&String> {
        Some(&t.instruction_id)
    }
    fn set(t: &mut CreditTransferTxInfo) -> Option<&mut String> {
        Some(&mut t.instruction_id)
    }
    Kp::new(get, set)
}

pub fn tx_end_to_end_id() -> TxStringKp {
    fn get(t: &CreditTransferTxInfo) -> Option<&String> {
        Some(&t.end_to_end_id)
    }
    fn set(t: &mut CreditTransferTxInfo) -> Option<&mut String> {
        Some(&mut t.end_to_end_id)
    }
    Kp::new(get, set)
}

pub fn tx_amount() -> TxAmountKp {
    fn get(t: &CreditTransferTxInfo) -> Option<&f64> {
        Some(&t.amount)
    }
    fn set(t: &mut CreditTransferTxInfo) -> Option<&mut f64> {
        Some(&mut t.amount)
    }
    Kp::new(get, set)
}

pub fn tx_currency() -> TxStringKp {
    fn get(t: &CreditTransferTxInfo) -> Option<&String> {
        Some(&t.currency)
    }
    fn set(t: &mut CreditTransferTxInfo) -> Option<&mut String> {
        Some(&mut t.currency)
    }
    Kp::new(get, set)
}

pub fn tx_creditor_name() -> TxStringKp {
    fn get(t: &CreditTransferTxInfo) -> Option<&String> {
        Some(&t.creditor_name)
    }
    fn set(t: &mut CreditTransferTxInfo) -> Option<&mut String> {
        Some(&mut t.creditor_name)
    }
    Kp::new(get, set)
}

pub fn tx_creditor_account_id() -> TxStringKp {
    fn get(t: &CreditTransferTxInfo) -> Option<&String> {
        Some(&t.creditor_account_id)
    }
    fn set(t: &mut CreditTransferTxInfo) -> Option<&mut String> {
        Some(&mut t.creditor_account_id)
    }
    Kp::new(get, set)
}

// Static-dispatch smoke: generic over any reference-shaped `KpTrait`.
#[allow(dead_code)]
fn accepts_state_kp<K>(kp: K) -> K
where
    K: KpTrait<
        Pain001,
        String,
        &'static Pain001,
        &'static String,
        &'static mut Pain001,
        &'static mut String,
    >,
{
    kp
}

#[allow(dead_code)]
fn _static_dispatch_smoke() {
    let _ = accepts_state_kp(pain_message_id());
}
