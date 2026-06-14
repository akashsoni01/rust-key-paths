//! Reusable field validators driven by keypaths.
//!
//! Each rule is a pure function; [`validate_at`] binds a keypath + ISO path label to rules.

use rust_key_paths::KpType;

use super::keypaths::{
    pain_creation_date_time, pain_initiating_party_id, pain_message_id, pmt_debtor_account_id,
    pmt_debtor_name, pmt_inf_id, pmt_payment_method, pmt_requested_execution_date,
    tx_amount, tx_creditor_account_id, tx_creditor_name, tx_currency, tx_end_to_end_id,
    tx_instruction_id,
};
use super::model::{CreditTransferTxInfo, Pain001, PaymentInformation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldError {
    pub path: String,
    pub message: &'static str,
}

pub type Rule<V> = fn(&V) -> Option<&'static str>;

/// Run all `rules` against a direct value reference.
pub fn validate_value<V>(iso_path: &str, value: &V, rules: &[Rule<V>]) -> Vec<FieldError> {
    let mut errors = Vec::new();
    for rule in rules {
        if let Some(message) = rule(value) {
            errors.push(FieldError {
                path: iso_path.to_string(),
                message,
            });
        }
    }
    errors
}

/// Run all `rules` against the value at `kp`; tag errors with `iso_path`.
pub fn validate_at<Root, V>(
    root: &Root,
    iso_path: &str,
    kp: KpType<'static, Root, V>,
    rules: &[Rule<V>],
) -> Vec<FieldError> {
    match kp.get(root) {
        Some(value) => validate_value(iso_path, value, rules),
        None => vec![FieldError {
            path: iso_path.to_string(),
            message: "missing",
        }],
    }
}

pub mod rules {
    use super::Rule;

    pub fn required_string(s: &String) -> Option<&'static str> {
        if s.is_empty() {
            Some("must not be empty")
        } else {
            None
        }
    }

    pub fn max_len_35(s: &String) -> Option<&'static str> {
        if s.len() > 35 {
            Some("max 35 characters")
        } else {
            None
        }
    }

    pub fn iso_date(s: &String) -> Option<&'static str> {
        if s.len() == 10 && s.as_bytes().get(4) == Some(&b'-') && s.as_bytes().get(7) == Some(&b'-') {
            None
        } else {
            Some("expected YYYY-MM-DD")
        }
    }

    pub fn iso8601_datetime(s: &String) -> Option<&'static str> {
        if s.contains('T') && s.len() >= 19 {
            None
        } else {
            Some("expected ISO 8601 datetime (CreDtTm)")
        }
    }

    pub fn payment_method_trf(s: &String) -> Option<&'static str> {
        if s == "TRF" {
            None
        } else {
            Some("PmtMtd must be TRF for credit transfer")
        }
    }

    pub fn iso_currency(s: &String) -> Option<&'static str> {
        if s.len() == 3 && s.chars().all(|c| c.is_ascii_uppercase()) {
            None
        } else {
            Some("currency must be 3-letter ISO 4217")
        }
    }

    pub fn positive_amount(n: &f64) -> Option<&'static str> {
        if *n > 0.0 {
            None
        } else {
            Some("amount must be > 0")
        }
    }

    pub fn optional_bic(s: &Option<String>) -> Option<&'static str> {
        match s {
            None => None,
            Some(bic) if bic.len() == 8 || bic.len() == 11 => None,
            Some(_) => Some("BICFI must be 8 or 11 characters"),
        }
    }

    pub fn optional_remittance(s: &Option<String>) -> Option<&'static str> {
        match s {
            Some(info) if info.len() > 140 => Some("remittance max 140 characters"),
            _ => None,
        }
    }

    pub fn min_name_len(s: &String) -> Option<&'static str> {
        if s.len() >= 2 {
            None
        } else {
            Some("name must be at least 2 characters")
        }
    }

    pub const MSG_ID: &[Rule<String>] = &[required_string, max_len_35];
    pub const CRE_DT_TM: &[Rule<String>] = &[required_string, iso8601_datetime];
    pub const PARTY_ID: &[Rule<String>] = &[required_string];
    pub const PMT_INF_ID: &[Rule<String>] = &[required_string, max_len_35];
    pub const PMT_MTD: &[Rule<String>] = &[required_string, payment_method_trf];
    pub const REQD_EXCTN_DT: &[Rule<String>] = &[required_string, iso_date];
    pub const DEBTOR_NAME: &[Rule<String>] = &[required_string, min_name_len];
    pub const ACCOUNT_ID: &[Rule<String>] = &[required_string];
    pub const INSTR_ID: &[Rule<String>] = &[required_string];
    pub const E2E_ID: &[Rule<String>] = &[required_string];
    pub const CREDITOR_NAME: &[Rule<String>] = &[required_string, min_name_len];
    pub const CURRENCY: &[Rule<String>] = &[required_string, iso_currency];
    pub const AMOUNT: &[Rule<f64>] = &[positive_amount];
}

pub fn validate_group_header(payload: &Pain001) -> Vec<FieldError> {
    let mut errors = Vec::new();
    errors.extend(validate_at(
        payload,
        "GrpHdr/MsgId",
        pain_message_id(),
        rules::MSG_ID,
    ));
    errors.extend(validate_at(
        payload,
        "GrpHdr/CreDtTm",
        pain_creation_date_time(),
        rules::CRE_DT_TM,
    ));
    errors.extend(validate_at(
        payload,
        "GrpHdr/InitgPty/Id",
        pain_initiating_party_id(),
        rules::PARTY_ID,
    ));
    errors
}

pub fn validate_payment_information(pmt: &PaymentInformation, prefix: &str) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let tag = |field_errors: Vec<FieldError>| {
        field_errors
            .into_iter()
            .map(|e| FieldError {
                path: format!("{prefix}/{}", e.path),
                message: e.message,
            })
            .collect::<Vec<_>>()
    };

    errors.extend(tag(validate_at(
        pmt,
        "PmtInfId",
        pmt_inf_id(),
        rules::PMT_INF_ID,
    )));
    errors.extend(tag(validate_at(
        pmt,
        "PmtMtd",
        pmt_payment_method(),
        rules::PMT_MTD,
    )));
    errors.extend(tag(validate_at(
        pmt,
        "ReqdExctnDt",
        pmt_requested_execution_date(),
        rules::REQD_EXCTN_DT,
    )));
    errors.extend(tag(validate_at(
        pmt,
        "Dbtr/Nm",
        pmt_debtor_name(),
        rules::DEBTOR_NAME,
    )));
    errors.extend(tag(validate_at(
        pmt,
        "DbtrAcct/Id",
        pmt_debtor_account_id(),
        rules::ACCOUNT_ID,
    )));
    errors.extend(validate_value(
        &format!("{prefix}/DbtrAgt/BICFI"),
        &pmt.debtor_agent_bic,
        &[rules::optional_bic],
    ));

    if pmt.credit_transfer_tx_infos.is_empty() {
        errors.push(FieldError {
            path: format!("{prefix}/CdtTrfTxInf"),
            message: "at least one credit transfer required",
        });
    }

    errors
}

pub fn validate_credit_transfer(tx: &CreditTransferTxInfo, prefix: &str) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let tag = |field_errors: Vec<FieldError>| {
        field_errors
            .into_iter()
            .map(|e| FieldError {
                path: format!("{prefix}/{}", e.path),
                message: e.message,
            })
            .collect::<Vec<_>>()
    };

    errors.extend(tag(validate_at(
        tx,
        "PmtId/InstrId",
        tx_instruction_id(),
        rules::INSTR_ID,
    )));
    errors.extend(tag(validate_at(
        tx,
        "PmtId/EndToEndId",
        tx_end_to_end_id(),
        rules::E2E_ID,
    )));
    errors.extend(tag(validate_at(tx, "Amt/InstdAmt", tx_amount(), rules::AMOUNT)));
    errors.extend(tag(validate_at(
        tx,
        "Amt/InstdAmt/@Ccy",
        tx_currency(),
        rules::CURRENCY,
    )));
    errors.extend(tag(validate_at(
        tx,
        "Cdtr/Nm",
        tx_creditor_name(),
        rules::CREDITOR_NAME,
    )));
    errors.extend(tag(validate_at(
        tx,
        "CdtrAcct/Id",
        tx_creditor_account_id(),
        rules::ACCOUNT_ID,
    )));
    errors.extend(validate_value(
        &format!("{prefix}/CdtrAgt/BICFI"),
        &tx.creditor_agent_bic,
        &[rules::optional_bic],
    ));
    errors.extend(validate_value(
        &format!("{prefix}/RmtInf/Ustrd"),
        &tx.remittance_info,
        &[rules::optional_remittance],
    ));

    errors
}

/// Cross-field: `GrpHdr/NbOfTxs` must equal total `CdtTrfTxInf` count.
pub fn validate_transaction_count(payload: &Pain001) -> Option<FieldError> {
    let expected = payload
        .payment_informations
        .iter()
        .map(|p| p.credit_transfer_tx_infos.len() as u32)
        .sum::<u32>();
    if payload.group_header.number_of_transactions == expected {
        None
    } else {
        Some(FieldError {
            path: "GrpHdr/NbOfTxs".to_string(),
            message: "must equal total CdtTrfTxInf count",
        })
    }
}

/// Cross-field: `GrpHdr/CtrlSum` must match sum of instructed amounts when present.
pub fn validate_control_sum(payload: &Pain001) -> Option<FieldError> {
    let Some(control_sum) = payload.group_header.control_sum else {
        return None;
    };
    let total: f64 = payload
        .payment_informations
        .iter()
        .flat_map(|p| p.credit_transfer_tx_infos.iter())
        .map(|tx| tx.amount)
        .sum();
    if (control_sum - total).abs() < 0.001 {
        None
    } else {
        Some(FieldError {
            path: "GrpHdr/CtrlSum".to_string(),
            message: "must equal sum of Amt/InstdAmt",
        })
    }
}

/// Full payload validation — reusable from reducer, tests, or API ingress.
pub fn validate_pain001(payload: &Pain001) -> Vec<FieldError> {
    let mut errors = validate_group_header(payload);

    if payload.payment_informations.is_empty() {
        errors.push(FieldError {
            path: "PmtInf".to_string(),
            message: "at least one payment information block required",
        });
    }

    for (i, pmt) in payload.payment_informations.iter().enumerate() {
        let prefix = format!("PmtInf[{i}]");
        errors.extend(validate_payment_information(pmt, &prefix));
        for (j, tx) in pmt.credit_transfer_tx_infos.iter().enumerate() {
            let tx_prefix = format!("{prefix}/CdtTrfTxInf[{j}]");
            errors.extend(validate_credit_transfer(tx, &tx_prefix));
        }
    }

    if let Some(err) = validate_transaction_count(payload) {
        errors.push(err);
    }
    if let Some(err) = validate_control_sum(payload) {
        errors.push(err);
    }

    errors
}
