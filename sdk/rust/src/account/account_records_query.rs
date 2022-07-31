use async_trait::async_trait;
use hedera_proto::services;
use hedera_proto::services::crypto_service_client::CryptoServiceClient;
use tonic::transport::Channel;

use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    AccountId,
    FromProtobuf,
    Query,
    ToProtobuf,
    TransactionRecord,
};

/// Get all the records for an account for any transfers into it and out of it,
/// that were above the threshold, during the last 25 hours.
pub type AccountRecordsQuery = Query<AccountRecordsQueryData>;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRecordsQueryData {
    account_id: Option<AccountId>,
}

impl From<AccountRecordsQueryData> for AnyQueryData {
    #[inline]
    fn from(data: AccountRecordsQueryData) -> Self {
        Self::AccountRecords(data)
    }
}

impl AccountRecordsQuery {
    /// Sets the account ID for which the records should be retrieved.
    pub fn account_id(&mut self, id: AccountId) -> &mut Self {
        self.data.account_id = Some(id.into());
        self
    }
}

impl ToQueryProtobuf for AccountRecordsQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let account_id = self.account_id.as_ref().map(|id| id.to_protobuf());

        services::Query {
            query: Some(services::query::Query::CryptoGetAccountRecords(
                services::CryptoGetAccountRecordsQuery { account_id, header: Some(header) },
            )),
        }
    }
}

#[async_trait]
impl QueryExecute for AccountRecordsQueryData {
    type Response = Vec<TransactionRecord>;

    async fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> Result<tonic::Response<services::Response>, tonic::Status> {
        CryptoServiceClient::new(channel).get_account_records(request).await
    }
}

impl FromProtobuf<services::response::Response> for Vec<TransactionRecord> {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let response = pb_getv!(pb, CryptoGetAccountRecords, services::response::Response);

        Vec::<TransactionRecord>::from_protobuf(response.records)
    }
}