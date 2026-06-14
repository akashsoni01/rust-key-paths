//! Sample PAIN.001 API payloads — valid and intentionally invalid.

use super::model::{
    CreditTransferTxInfo, GroupHeader, Pain001, PaymentInformation,
};

pub fn sample_valid() -> Pain001 {
    Pain001 {
        group_header: GroupHeader {
            message_id: "MSG-2024-001".to_string(),
            creation_date_time: "2024-01-15T10:00:00".to_string(),
            number_of_transactions: 3,
            control_sum: Some(425.75),
            initiating_party_id: "CUST-123".to_string(),
        },
        payment_informations: vec![
            PaymentInformation {
                pmt_inf_id: "PMT-001".to_string(),
                payment_method: "TRF".to_string(),
                requested_execution_date: "2024-01-16".to_string(),
                debtor_name: "Acme Treasury".to_string(),
                debtor_account_id: "GB82WEST12345698765432".to_string(),
                debtor_agent_bic: Some("WESTGB22".to_string()),
                credit_transfer_tx_infos: vec![
                    CreditTransferTxInfo {
                        instruction_id: "INSTR-1".to_string(),
                        end_to_end_id: "E2E-1".to_string(),
                        amount: 100.50,
                        currency: "USD".to_string(),
                        creditor_name: "Acme Corp".to_string(),
                        creditor_account_id: "US64SVBKUS6S3300958879".to_string(),
                        creditor_agent_bic: Some("SVBKUS6S".to_string()),
                        remittance_info: Some("Invoice #1".to_string()),
                    },
                    CreditTransferTxInfo {
                        instruction_id: "INSTR-2".to_string(),
                        end_to_end_id: "E2E-2".to_string(),
                        amount: 250.0,
                        currency: "EUR".to_string(),
                        creditor_name: "Beta Inc".to_string(),
                        creditor_account_id: "DE89370400440532013000".to_string(),
                        creditor_agent_bic: None,
                        remittance_info: None,
                    },
                ],
            },
            PaymentInformation {
                pmt_inf_id: "PMT-002".to_string(),
                payment_method: "TRF".to_string(),
                requested_execution_date: "2024-01-17".to_string(),
                debtor_name: "Acme Treasury".to_string(),
                debtor_account_id: "GB82WEST12345698765432".to_string(),
                debtor_agent_bic: None,
                credit_transfer_tx_infos: vec![CreditTransferTxInfo {
                    instruction_id: "INSTR-3".to_string(),
                    end_to_end_id: "E2E-3".to_string(),
                    amount: 75.25,
                    currency: "GBP".to_string(),
                    creditor_name: "Gamma Ltd".to_string(),
                    creditor_account_id: "GB29NWBK60161331926819".to_string(),
                    creditor_agent_bic: Some("NWBKGB2L".to_string()),
                    remittance_info: Some("Refund".to_string()),
                }],
            },
        ],
    }
}

/// Invalid payload for validation demo — wrong counts, bad currency, empty ids.
pub fn sample_invalid() -> Pain001 {
    let mut pain = sample_valid();
    pain.group_header.message_id = String::new();
    pain.group_header.number_of_transactions = 99;
    pain.group_header.control_sum = Some(1.0);
    pain.payment_informations[0].payment_method = "CHK".to_string();
    pain.payment_informations[0].credit_transfer_tx_infos[0].currency = "US".to_string();
    pain.payment_informations[0].credit_transfer_tx_infos[0].amount = -10.0;
    pain.payment_informations[1].debtor_name = "X".to_string();
    pain
}
