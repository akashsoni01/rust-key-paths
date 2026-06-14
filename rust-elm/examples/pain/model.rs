//! ISO 20022 **pain.001.001.07** (Customer Credit Transfer Initiation) — simplified API payload.
//!
//! Field names follow GS / ISO element tags. See [`book/pain.md`](../../book/pain.md).

use key_paths_derive::Kp;

/// Root message: `Document/CstmrCdtTrfInitn`
#[derive(Clone, Debug, Default, PartialEq, Kp)]
pub struct Pain001 {
    pub group_header: GroupHeader,
    pub payment_informations: Vec<PaymentInformation>,
}

/// `GrpHdr` — message-level header (once).
#[derive(Clone, Debug, Default, PartialEq, Kp)]
pub struct GroupHeader {
    /// `MsgId` — unique message identification (max 35).
    pub message_id: String,
    /// `CreDtTm` — creation date time (ISO 8601).
    pub creation_date_time: String,
    /// `NbOfTxs` — number of transactions in the message.
    pub number_of_transactions: u32,
    /// `CtrlSum` — total instructed amount (optional control sum).
    pub control_sum: Option<f64>,
    /// `InitgPty/Id` — initiating party identifier.
    pub initiating_party_id: String,
}

/// `PmtInf` — payment information block (one or more).
#[derive(Clone, Debug, Default, PartialEq, Kp)]
pub struct PaymentInformation {
    /// `PmtInfId` — unique payment batch id.
    pub pmt_inf_id: String,
    /// `PmtMtd` — payment method (`TRF`, `CHK`, …).
    pub payment_method: String,
    /// `ReqdExctnDt` — requested execution date (`YYYY-MM-DD`).
    pub requested_execution_date: String,
    /// `Dbtr/Nm` — debtor name.
    pub debtor_name: String,
    /// `DbtrAcct/Id` — debtor account id (IBAN or domestic).
    pub debtor_account_id: String,
    /// `DbtrAgt/BICFI` — debtor agent BIC (8 or 11 chars when present).
    pub debtor_agent_bic: Option<String>,
    /// `CdtTrfTxInf` — credit transfer transactions (one or more).
    pub credit_transfer_tx_infos: Vec<CreditTransferTxInfo>,
}

/// `CdtTrfTxInf` — single credit transfer.
#[derive(Clone, Debug, Default, PartialEq, Kp)]
pub struct CreditTransferTxInfo {
    /// `PmtId/InstrId` — instruction id (unique within batch).
    pub instruction_id: String,
    /// `PmtId/EndToEndId` — end-to-end id.
    pub end_to_end_id: String,
    /// `Amt/InstdAmt` — instructed amount.
    pub amount: f64,
    /// `Amt/InstdAmt/@Ccy` — currency (ISO 4217, 3 letters).
    pub currency: String,
    /// `Cdtr/Nm` — creditor name.
    pub creditor_name: String,
    /// `CdtrAcct/Id` — creditor account id.
    pub creditor_account_id: String,
    /// `CdtrAgt/BICFI` — creditor agent BIC.
    pub creditor_agent_bic: Option<String>,
    /// `RmtInf/Ustrd` — unstructured remittance (optional).
    pub remittance_info: Option<String>,
}
