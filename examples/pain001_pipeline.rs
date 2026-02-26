//! PAIN.001 (ISO 20022 Customer Credit Transfer Initiation) pipeline on KpType.
//!
//! See: [GS Transaction Banking PAIN.001](https://developer.gs.com/docs/services/transaction-banking/pain001details/)
//!
//! This example:
//! - Defines PAIN.001 structs (GrpHdr, PmtInf, CdtTrfTxInf) with keypaths
//! - Uses **map**, **filter**, **PKp** (PartialKeyPath), **AKp** (AnyKeyPath) to build a validation pipeline
//! - Runs parallel (GPU-ready) validation via key_paths_iter (par_map, par_filter, par_all)

use key_paths_iter::query_par::ParallelCollectionKeyPath;
use rayon::prelude::*;
use rust_key_paths::{Kp, KpType, PKp, AKp};

// ══════════════════════════════════════════════════════════════════════════
// 1. PAIN.001 STRUCTS (simplified ISO 20022 pain.001.001.07)
// ══════════════════════════════════════════════════════════════════════════

/// Group Header (GrpHdr) – message-level info
#[derive(Clone, Debug, Default)]
pub struct GroupHeader {
    /// MsgId – unique message identification
    pub message_id: String,
    /// CreDtTm – creation date time
    pub creation_date_time: String,
    /// NbOfTxs – number of transactions (optional, can be derived)
    pub number_of_transactions: u32,
    /// InitgPty – initiating party
    pub initiating_party_id: String,
}

/// Payment Information (PmtInf) – one per batch of transfers with same debtor/account
#[derive(Clone, Debug, Default)]
pub struct PaymentInformation {
    /// PmtInfId – unique payment information identifier
    pub pmt_inf_id: String,
    /// DbtrAcct – debtor account id
    pub debtor_account_id: String,
    /// PmtMtd – payment method (e.g. TRF)
    pub payment_method: String,
    /// CdtTrfTxInf – credit transfer transaction info (multiple)
    pub credit_transfer_tx_infos: Vec<CreditTransferTxInfo>,
}

/// Credit Transfer Transaction Information (CdtTrfTxInf)
#[derive(Clone, Debug, Default)]
pub struct CreditTransferTxInfo {
    /// PmtId.InstrId – instruction identifier
    pub instruction_id: String,
    /// Amt – instructed amount
    pub amount: f64,
    /// Ccy – currency (3 letters)
    pub currency: String,
    /// CdtrAcct – creditor account id
    pub creditor_account_id: String,
    /// Cdtr – creditor name
    pub creditor_name: String,
    /// RmtInf – remittance information (optional)
    pub remittance_info: Option<String>,
}

/// Root PAIN.001 message (CustomerCreditTransferInitiation)
#[derive(Clone, Debug, Default)]
pub struct Pain001 {
    /// GrpHdr – group header (once)
    pub group_header: GroupHeader,
    /// PmtInf – payment information (one or more)
    pub payment_informations: Vec<PaymentInformation>,
}

// ══════════════════════════════════════════════════════════════════════════
// 2. KEYPATHS (KpType) for PAIN.001
// ══════════════════════════════════════════════════════════════════════════

fn pain_group_header() -> KpType<'static, Pain001, GroupHeader> {
    Kp::new(
        |p: &Pain001| Some(&p.group_header),
        |p: &mut Pain001| Some(&mut p.group_header),
    )
}

fn pain_message_id() -> KpType<'static, Pain001, String> {
    Kp::new(
        |p: &Pain001| Some(&p.group_header.message_id),
        |p: &mut Pain001| Some(&mut p.group_header.message_id),
    )
}

fn pain_payment_informations() -> KpType<'static, Pain001, Vec<PaymentInformation>> {
    Kp::new(
        |p: &Pain001| Some(&p.payment_informations),
        |p: &mut Pain001| Some(&mut p.payment_informations),
    )
}

/// Keypath: PaymentInformation → Vec<CreditTransferTxInfo> (for nested pipeline or GPU buffer extraction)
#[allow(dead_code)]
fn pmt_inf_credit_transfer_tx_infos() -> KpType<'static, PaymentInformation, Vec<CreditTransferTxInfo>> {
    Kp::new(
        |p: &PaymentInformation| Some(&p.credit_transfer_tx_infos),
        |p: &mut PaymentInformation| Some(&mut p.credit_transfer_tx_infos),
    )
}

// ══════════════════════════════════════════════════════════════════════════
// 3. PIPELINE: map, filter, par_* on KpType
// ══════════════════════════════════════════════════════════════════════════

/// All credit transfer tx infos from the whole message (flat)
fn all_credit_transfers(pain: &Pain001) -> Vec<CreditTransferTxInfo> {
    pain_payment_informations().par_flat_map(pain, |pmt| pmt.credit_transfer_tx_infos.clone())
}

/// Parallel validation: every payment info has at least one tx and valid id
fn validate_payment_infos(pain: &Pain001) -> bool {
    let kp = pain_payment_informations();
    kp.par_all(pain, |pmt| {
        !pmt.pmt_inf_id.is_empty() && !pmt.credit_transfer_tx_infos.is_empty()
    })
}

/// Parallel validation: every credit transfer has amount > 0 and non-empty currency
fn validate_credit_transfers(pain: &Pain001) -> bool {
    let kp = pain_payment_informations();
    kp.par_all(pain, |pmt| {
        pmt.credit_transfer_tx_infos
            .iter()
            .all(|tx| tx.amount > 0.0 && tx.currency.len() == 3)
    })
}

/// Count transactions with amount above threshold (par_count_by style via par_flat_map + len)
fn count_transfers_above(pain: &Pain001, min_amount: f64) -> usize {
    let kp = pain_payment_informations();
    kp.par_flat_map(pain, |pmt| {
        pmt.credit_transfer_tx_infos
            .iter()
            .filter(|tx| tx.amount >= min_amount)
            .cloned()
            .collect::<Vec<_>>()
    })
    .len()
}

/// Sum of all instructed amounts (parallel fold)
fn total_instructed_amount(pain: &Pain001) -> f64 {
    let kp = pain_payment_informations();
    kp.par_fold(
        pain,
        &(|| 0.0_f64),
        |acc, pmt| {
            acc + pmt
                .credit_transfer_tx_infos
                .iter()
                .map(|tx| tx.amount)
                .sum::<f64>()
        },
        |a, b| a + b,
    )
}

/// GPU-ready validation: extract all amounts into a flat buffer and validate in parallel (par_all).
/// Same pattern as scale_par: data can be sent to GPU for bulk checks if needed.
fn validate_all_amounts_positive(pain: &Pain001) -> bool {
    let kp = pain_payment_informations();
    kp.par_flat_map(pain, |pmt| {
        pmt.credit_transfer_tx_infos.iter().map(|tx| tx.amount).collect::<Vec<_>>()
    })
    .par_iter()
    .all(|&a| a > 0.0)
}

// ══════════════════════════════════════════════════════════════════════════
// 4. PKp (PartialKeyPath): filter + map on message_id
// ══════════════════════════════════════════════════════════════════════════

fn run_pkp_pipeline(pain: &Pain001) {
    let msg_id_kp = pain_message_id();
    let msg_id_pkp: PKp<Pain001> = PKp::new(msg_id_kp);

    // Filter: only consider non-empty message id
    let valid_msg_pkp = msg_id_pkp.filter::<String, _>(|s| !s.is_empty());
    if valid_msg_pkp.get_as::<String>(pain).is_some() {
        println!("  PKp: message_id passes filter (non-empty)");
    }

    // Map: message_id length
    let len_pkp = msg_id_pkp.map::<String, usize, _>(|s| s.len());
    if let Some(len) = len_pkp.get_as::<usize>(pain) {
        println!("  PKp: message_id length = {}", len);
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 5. AKp (AnyKeyPath): type-erased keypath + filter
// ══════════════════════════════════════════════════════════════════════════

fn run_akp_pipeline(pain: &Pain001) {
    let msg_id_kp = pain_message_id();
    let msg_id_akp = AKp::new(msg_id_kp);

    // Filter: message_id non-empty (Root = Pain001, Value = String)
    let valid_akp = msg_id_akp.filter::<Pain001, String, _>(|s| !s.is_empty());
    // get_as<Root, Value> returns Option<Option<&Value>> when root type matches
    if valid_akp.get_as::<Pain001, String>(pain).and_then(|o| o).is_some() {
        println!("  AKp: message_id passes filter");
    }
}

// ══════════════════════════════════════════════════════════════════════════
// 6. MAIN: build Pain001, run full pipeline, GPU-ready validation
// ══════════════════════════════════════════════════════════════════════════

fn main() {
    println!("=== PAIN.001 pipeline (KpType + map, filter, PKp, AKp + GPU-ready validation) ===\n");

    let pain = Pain001 {
        group_header: GroupHeader {
            message_id: "MSG-2024-001".to_string(),
            creation_date_time: "2024-01-15T10:00:00".to_string(),
            number_of_transactions: 3,
            initiating_party_id: "CUST-123".to_string(),
        },
        payment_informations: vec![
            PaymentInformation {
                pmt_inf_id: "PMT-001".to_string(),
                debtor_account_id: "DEBT-ACC-1".to_string(),
                payment_method: "TRF".to_string(),
                credit_transfer_tx_infos: vec![
                    CreditTransferTxInfo {
                        instruction_id: "INSTR-1".to_string(),
                        amount: 100.50,
                        currency: "USD".to_string(),
                        creditor_account_id: "CRED-1".to_string(),
                        creditor_name: "Acme Corp".to_string(),
                        remittance_info: Some("Invoice #1".to_string()),
                    },
                    CreditTransferTxInfo {
                        instruction_id: "INSTR-2".to_string(),
                        amount: 250.0,
                        currency: "EUR".to_string(),
                        creditor_account_id: "CRED-2".to_string(),
                        creditor_name: "Beta Inc".to_string(),
                        remittance_info: None,
                    },
                ],
            },
            PaymentInformation {
                pmt_inf_id: "PMT-002".to_string(),
                debtor_account_id: "DEBT-ACC-1".to_string(),
                payment_method: "TRF".to_string(),
                credit_transfer_tx_infos: vec![CreditTransferTxInfo {
                    instruction_id: "INSTR-3".to_string(),
                    amount: 75.25,
                    currency: "GBP".to_string(),
                    creditor_account_id: "CRED-3".to_string(),
                    creditor_name: "Gamma Ltd".to_string(),
                    remittance_info: Some("Refund".to_string()),
                }],
            },
        ],
    };

    // 1) KpType get
    let grp = pain_group_header().get(&pain).unwrap();
    println!("1) GroupHeader MsgId: {}", grp.message_id);

    // 2) Parallel validation (GPU-ready: par_all over collections)
    let ok_pmt = validate_payment_infos(&pain);
    let ok_tx = validate_credit_transfers(&pain);
    println!("2) Validation (par_all): payment_infos valid = {}, credit_transfers valid = {}", ok_pmt, ok_tx);

    // 3) par_map / par_flat_map: collect all amounts for downstream or GPU buffer
    let all_tx = all_credit_transfers(&pain);
    println!("3) par_flat_map: total credit transfers = {}", all_tx.len());

    // 4) par_count_by style: transfers above threshold
    let count_above_100 = count_transfers_above(&pain, 100.0);
    println!("4) Count transfers with amount >= 100: {}", count_above_100);

    // 5) par_fold: total instructed amount
    let total = total_instructed_amount(&pain);
    println!("5) par_fold: total instructed amount = {:.2}", total);

    // 5b) GPU-ready validation: flat amount buffer + par_all (can run on GPU via scale_par pattern)
    let amounts_ok = validate_all_amounts_positive(&pain);
    println!("5b) GPU-ready validation (par_flat_map + par_all): all amounts > 0 = {}", amounts_ok);

    // 6) PKp pipeline (filter + map)
    println!("6) PKp (PartialKeyPath):");
    run_pkp_pipeline(&pain);

    // 7) AKp pipeline (type-erased filter)
    println!("7) AKp (AnyKeyPath):");
    run_akp_pipeline(&pain);

    println!("\nDone: PAIN.001 pipeline on KpType (map, filter, PKp, AKp, par_* validation).");
}
