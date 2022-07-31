use async_trait::async_trait;
use hedera_proto::services;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use serde::{
    Deserialize,
    Serialize,
};
use serde_with::skip_serializing_none;
use tonic::transport::Channel;

use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    ContractId,
    FromProtobuf,
    Query,
    ToProtobuf,
};

/// Get the runtime bytecode for a smart contract instance.
pub type ContractBytecodeQuery = Query<ContractBytecodeQueryData>;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractBytecodeQueryData {
    /// The contract for which information is requested.
    contract_id: Option<ContractId>,
}

impl ContractBytecodeQuery {
    /// Sets the contract for which information is requested.
    pub fn contract_id(&mut self, contract_id: ContractId) -> &mut Self {
        self.data.contract_id = Some(contract_id);
        self
    }
}

impl From<ContractBytecodeQueryData> for AnyQueryData {
    #[inline]
    fn from(data: ContractBytecodeQueryData) -> Self {
        Self::ContractBytecode(data)
    }
}

impl ToQueryProtobuf for ContractBytecodeQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let contract_id = self.contract_id.as_ref().map(|id| id.to_protobuf());

        services::Query {
            query: Some(services::query::Query::ContractGetBytecode(
                services::ContractGetBytecodeQuery { contract_id, header: Some(header) },
            )),
        }
    }
}

#[async_trait]
impl QueryExecute for ContractBytecodeQueryData {
    type Response = Vec<u8>;

    async fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> Result<tonic::Response<services::Response>, tonic::Status> {
        SmartContractServiceClient::new(channel).contract_get_bytecode(request).await
    }
}

impl FromProtobuf<services::response::Response> for Vec<u8> {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pb = pb_getv!(pb, ContractGetBytecodeResponse, services::response::Response);
        
        Ok(pb.bytecode)
    }
}
