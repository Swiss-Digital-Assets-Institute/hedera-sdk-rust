use async_trait::async_trait;
use hedera_proto::services;
use serde::{Deserialize, Deserializer};
use serde_with::skip_serializing_none;
use time::Duration;
use tonic::transport::Channel;
use tonic::{Response, Status};

use crate::account::{
    AccountCreateTransactionData, AccountDeleteTransactionData, AccountUpdateTransactionData
};
use crate::transaction::{ToTransactionDataProtobuf, TransactionBody, TransactionExecute};
use crate::transfer_transaction::TransferTransactionData;
use crate::{AccountId, Transaction, TransactionId};

/// Any possible transaction that may be executed on the Hedera network.
pub type AnyTransaction = Transaction<AnyTransactionData>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnyTransactionData {
    AccountCreate(AccountCreateTransactionData),
    AccountUpdate(AccountUpdateTransactionData),
    AccountDelete(AccountDeleteTransactionData),
    Transfer(TransferTransactionData),
}

impl ToTransactionDataProtobuf for AnyTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        node_account_id: AccountId,
        transaction_id: &TransactionId,
    ) -> services::transaction_body::Data {
        match self {
            Self::Transfer(transaction) => {
                transaction.to_transaction_data_protobuf(node_account_id, transaction_id)
            }

            Self::AccountCreate(transaction) => {
                transaction.to_transaction_data_protobuf(node_account_id, transaction_id)
            }

            Self::AccountUpdate(transaction) => {
                transaction.to_transaction_data_protobuf(node_account_id, transaction_id)
            }

            Self::AccountDelete(transaction) => {
                transaction.to_transaction_data_protobuf(node_account_id, transaction_id)
            }
        }
    }
}

#[async_trait]
impl TransactionExecute for AnyTransactionData {
    fn default_max_transaction_fee(&self) -> u64 {
        match self {
            Self::Transfer(transaction) => transaction.default_max_transaction_fee(),
            Self::AccountCreate(transaction) => transaction.default_max_transaction_fee(),
            Self::AccountUpdate(transaction) => transaction.default_max_transaction_fee(),
            Self::AccountDelete(transaction) => transaction.default_max_transaction_fee(),
        }
    }

    async fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> Result<Response<services::TransactionResponse>, Status> {
        match self {
            Self::AccountCreate(transaction) => transaction.execute(channel, request).await,
            Self::AccountUpdate(transaction) => transaction.execute(channel, request).await,
            Self::AccountDelete(transaction) => transaction.execute(channel, request).await,
            Self::Transfer(transaction) => transaction.execute(channel, request).await,
        }
    }
}

// NOTE: as we cannot derive Deserialize on Query<T> directly as `T` is not Deserialize,
//  we create a proxy type that has the same layout but is only for AnyQueryData and does
//  derive(Deserialize).

#[skip_serializing_none]
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnyTransactionBody<D> {
    #[serde(flatten)]
    data: D,
    #[serde(default)]
    node_account_ids: Option<Vec<AccountId>>,
    #[serde(default, with = "crate::serde::duration_opt")]
    transaction_valid_duration: Option<Duration>,
    #[serde(default)]
    max_transaction_fee: Option<u64>,
    #[serde(default, skip_serializing_if = "crate::serde::skip_if_string_empty")]
    transaction_memo: String,
    #[serde(default)]
    payer_account_id: Option<AccountId>,
    #[serde(default)]
    transaction_id: Option<TransactionId>,
}

impl<D> From<AnyTransactionBody<D>> for Transaction<D>
where
    D: TransactionExecute,
{
    fn from(body: AnyTransactionBody<D>) -> Self {
        Self { body: body.into(), signers: Vec::new() }
    }
}

impl<D> From<TransactionBody<D>> for AnyTransactionBody<D>
where
    D: TransactionExecute,
{
    fn from(body: TransactionBody<D>) -> Self {
        Self {
            data: body.data,
            node_account_ids: body.node_account_ids,
            transaction_valid_duration: body.transaction_valid_duration,
            max_transaction_fee: body.max_transaction_fee,
            transaction_memo: body.transaction_memo,
            payer_account_id: body.payer_account_id,
            transaction_id: body.transaction_id,
        }
    }
}

impl<D> From<AnyTransactionBody<D>> for TransactionBody<D>
where
    D: TransactionExecute,
{
    fn from(body: AnyTransactionBody<D>) -> Self {
        Self {
            data: body.data,
            node_account_ids: body.node_account_ids,
            transaction_valid_duration: body.transaction_valid_duration,
            max_transaction_fee: body.max_transaction_fee,
            transaction_memo: body.transaction_memo,
            payer_account_id: body.payer_account_id,
            transaction_id: body.transaction_id,
        }
    }
}

impl<'de> Deserialize<'de> for AnyTransaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <AnyTransactionBody<AnyTransactionData> as Deserialize>::deserialize(deserializer)
            .map(AnyTransactionBody::into)
    }
}