use async_trait::async_trait;
use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use services::crypto_get_account_balance_query::BalanceSource;
use tonic::transport::Channel;

use crate::query::{Query, QueryExecute, ToQueryProtobuf};
use crate::{AccountBalance, AccountIdOrAlias, ContractIdOrEvmAddress, ToProtobuf};

/// Get the balance of a cryptocurrency account.
///
/// This returns only the balance, so it is a smaller reply
/// than [`AccountInfoQuery`][crate::AccountInfoQuery], which returns the balance plus
/// additional information.
///
pub type AccountBalanceQuery = Query<AccountBalanceQueryData>;

#[derive(Default, Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct AccountBalanceQueryData {
    source: Option<AccountBalanceSource>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
enum AccountBalanceSource {
    Account(AccountIdOrAlias),
    Contract(ContractIdOrEvmAddress),
}

impl AccountBalanceQuery {
    /// Sets the account ID for which information is requested.
    ///
    /// This is mutually exclusive with [`contract_id`](#method.contract_id).
    ///
    pub fn account_id(&mut self, id: impl Into<AccountIdOrAlias>) -> &mut Self {
        self.data.source = Some(AccountBalanceSource::Account(id.into()));
        self
    }

    /// Sets the contract ID for which information is requested.
    ///
    /// This is mutually exclusive with [`account_id`](#method.account_id).
    ///
    pub fn contract_id(&mut self, id: ContractIdOrEvmAddress) -> &mut Self {
        self.data.source = Some(AccountBalanceSource::Contract(id.into()));
        self
    }
}

impl ToQueryProtobuf for AccountBalanceQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let source = self.source.as_ref().map(|source| match source {
            AccountBalanceSource::Account(id) => BalanceSource::AccountId(id.to_protobuf()),
            AccountBalanceSource::Contract(id) => BalanceSource::ContractId(id.to_protobuf()),
        });

        services::Query {
            query: Some(services::query::Query::CryptogetAccountBalance(
                services::CryptoGetAccountBalanceQuery {
                    balance_source: source,
                    header: Some(header),
                },
            )),
        }
    }
}

#[async_trait]
impl QueryExecute for AccountBalanceQuery {
    type Response = AccountBalance;

    fn is_payment_required() -> bool {
        false
    }

    async fn execute(
        channel: Channel,
        request: services::Query,
    ) -> Result<tonic::Response<services::Response>, tonic::Status> {
        CryptoServiceClient::new(channel).crypto_get_balance(request).await
    }
}