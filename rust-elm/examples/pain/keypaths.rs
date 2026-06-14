//! Composed [`KpType`] paths (fn pointers) — reusable with [`super::validation::validate_at`].

use rust_key_paths::{Kp, KpType};

use super::model::{CreditTransferTxInfo, Pain001, PaymentInformation};

pub fn pain_message_id() -> KpType<'static, Pain001, String> {
    Kp::new(
        |p: &Pain001| Some(&p.group_header.message_id),
        |p: &mut Pain001| Some(&mut p.group_header.message_id),
    )
}

pub fn pain_creation_date_time() -> KpType<'static, Pain001, String> {
    Kp::new(
        |p: &Pain001| Some(&p.group_header.creation_date_time),
        |p: &mut Pain001| Some(&mut p.group_header.creation_date_time),
    )
}

pub fn pain_initiating_party_id() -> KpType<'static, Pain001, String> {
    Kp::new(
        |p: &Pain001| Some(&p.group_header.initiating_party_id),
        |p: &mut Pain001| Some(&mut p.group_header.initiating_party_id),
    )
}

pub fn pmt_inf_id() -> KpType<'static, PaymentInformation, String> {
    Kp::new(
        |p: &PaymentInformation| Some(&p.pmt_inf_id),
        |p: &mut PaymentInformation| Some(&mut p.pmt_inf_id),
    )
}

pub fn pmt_payment_method() -> KpType<'static, PaymentInformation, String> {
    Kp::new(
        |p: &PaymentInformation| Some(&p.payment_method),
        |p: &mut PaymentInformation| Some(&mut p.payment_method),
    )
}

pub fn pmt_requested_execution_date() -> KpType<'static, PaymentInformation, String> {
    Kp::new(
        |p: &PaymentInformation| Some(&p.requested_execution_date),
        |p: &mut PaymentInformation| Some(&mut p.requested_execution_date),
    )
}

pub fn pmt_debtor_name() -> KpType<'static, PaymentInformation, String> {
    Kp::new(
        |p: &PaymentInformation| Some(&p.debtor_name),
        |p: &mut PaymentInformation| Some(&mut p.debtor_name),
    )
}

pub fn pmt_debtor_account_id() -> KpType<'static, PaymentInformation, String> {
    Kp::new(
        |p: &PaymentInformation| Some(&p.debtor_account_id),
        |p: &mut PaymentInformation| Some(&mut p.debtor_account_id),
    )
}

pub fn tx_instruction_id() -> KpType<'static, CreditTransferTxInfo, String> {
    Kp::new(
        |t: &CreditTransferTxInfo| Some(&t.instruction_id),
        |t: &mut CreditTransferTxInfo| Some(&mut t.instruction_id),
    )
}

pub fn tx_end_to_end_id() -> KpType<'static, CreditTransferTxInfo, String> {
    Kp::new(
        |t: &CreditTransferTxInfo| Some(&t.end_to_end_id),
        |t: &mut CreditTransferTxInfo| Some(&mut t.end_to_end_id),
    )
}

pub fn tx_amount() -> KpType<'static, CreditTransferTxInfo, f64> {
    Kp::new(
        |t: &CreditTransferTxInfo| Some(&t.amount),
        |t: &mut CreditTransferTxInfo| Some(&mut t.amount),
    )
}

pub fn tx_currency() -> KpType<'static, CreditTransferTxInfo, String> {
    Kp::new(
        |t: &CreditTransferTxInfo| Some(&t.currency),
        |t: &mut CreditTransferTxInfo| Some(&mut t.currency),
    )
}

pub fn tx_creditor_name() -> KpType<'static, CreditTransferTxInfo, String> {
    Kp::new(
        |t: &CreditTransferTxInfo| Some(&t.creditor_name),
        |t: &mut CreditTransferTxInfo| Some(&mut t.creditor_name),
    )
}

pub fn tx_creditor_account_id() -> KpType<'static, CreditTransferTxInfo, String> {
    Kp::new(
        |t: &CreditTransferTxInfo| Some(&t.creditor_account_id),
        |t: &mut CreditTransferTxInfo| Some(&mut t.creditor_account_id),
    )
}
