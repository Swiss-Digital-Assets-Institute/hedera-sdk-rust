use async_trait::async_trait;
use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
use tonic::transport::Channel;

use crate::query::{AnyQueryData, Query, QueryExecute, ToQueryProtobuf};
use crate::{FileContentsResponse, FileId, ToProtobuf};

/// Get the contents of a file.
pub type FileContentsQuery = Query<FileContentsQueryData>;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize, Debug)]
pub struct FileContentsQueryData {
    file_id: Option<FileId>,
}

impl From<FileContentsQueryData> for AnyQueryData {
    #[inline]
    fn from(data: FileContentsQueryData) -> Self {
        Self::FileContents(data)
    }
}

impl FileContentsQuery {
    /// Sets the file ID for which contents are requested
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        self.data.file_id = Some(id.into());
        self
    }
}

impl ToQueryProtobuf for FileContentsQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        services::Query {
            query: Some(services::query::Query::FileGetContents(services::FileGetContentsQuery {
                header: Some(header),
                file_id: self.file_id.as_ref().map(FileId::to_protobuf),
            })),
        }
    }
}

#[async_trait]
impl QueryExecute for FileContentsQueryData {
    type Response = FileContentsResponse;

    async fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> Result<tonic::Response<services::Response>, tonic::Status> {
        FileServiceClient::new(channel).get_file_content(request).await
    }
}