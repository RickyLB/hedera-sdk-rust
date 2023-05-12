/*
 * ‌
 * Hedera Rust SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

use hedera_proto::services;
use hedera_proto::services::file_service_client::FileServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::query::{
    AnyQueryData,
    QueryExecute,
    ToQueryProtobuf,
};
use crate::{
    BoxGrpcFuture,
    Error,
    FileId,
    FileInfo,
    Query,
    ToProtobuf,
    ValidateChecksums,
};

/// Get all the information about a file.
pub type FileInfoQuery = Query<FileInfoQueryData>;

#[derive(Default, Clone, Debug)]
pub struct FileInfoQueryData {
    file_id: Option<FileId>,
}

impl From<FileInfoQueryData> for AnyQueryData {
    #[inline]
    fn from(data: FileInfoQueryData) -> Self {
        Self::FileInfo(data)
    }
}

impl FileInfoQuery {
    /// Returns the ID of the file for which information is requested.
    #[must_use]
    pub fn get_file_id(&self) -> Option<FileId> {
        self.data.file_id
    }

    /// Sets the ID of the file for which information is requested.
    pub fn file_id(&mut self, id: impl Into<FileId>) -> &mut Self {
        self.data.file_id = Some(id.into());
        self
    }
}

impl ToQueryProtobuf for FileInfoQueryData {
    fn to_query_protobuf(&self, header: services::QueryHeader) -> services::Query {
        let file_id = self.file_id.to_protobuf();

        services::Query {
            query: Some(services::query::Query::FileGetInfo(services::FileGetInfoQuery {
                file_id,
                header: Some(header),
            })),
        }
    }
}

impl QueryExecute for FileInfoQueryData {
    type Response = FileInfo;

    fn execute(
        &self,
        channel: Channel,
        request: services::Query,
    ) -> BoxGrpcFuture<'_, services::Response> {
        Box::pin(async { FileServiceClient::new(channel).get_file_info(request).await })
    }
}

impl ValidateChecksums for FileInfoQueryData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        self.file_id.validate_checksums(ledger_id)
    }
}
