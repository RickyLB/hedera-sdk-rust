use async_trait::async_trait;
use hedera_proto::services;
use hedera_proto::services::smart_contract_service_client::SmartContractServiceClient;
use serde_with::skip_serializing_none;
use tonic::transport::Channel;

use crate::protobuf::ToProtobuf;
use crate::transaction::{
    AnyTransactionData,
    ToTransactionDataProtobuf,
    TransactionExecute,
};
use crate::{
    AccountAddress,
    AccountId,
    ContractId,
    Transaction,
};

/// Marks a contract as deleted and transfers its remaining hBars, if any, to
/// a designated receiver.
///
pub type ContractDeleteTransaction = Transaction<ContractDeleteTransactionData>;

#[skip_serializing_none]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractDeleteTransactionData {
    pub delete_contract_id: Option<ContractId>,

    pub transfer_account_id: Option<AccountAddress>,

    pub transfer_contract_id: Option<ContractId>,
}

impl ContractDeleteTransaction {
    /// Sets the contract ID which should be deleted.
    pub fn delete_contract_id(&mut self, id: ContractId) -> &mut Self {
        self.body.data.delete_contract_id = Some(id.into());
        self
    }

    /// Sets the account ID which will receive all remaining hbars.
    pub fn transfer_account_id(&mut self, id: impl Into<AccountAddress>) -> &mut Self {
        self.body.data.transfer_account_id = Some(id.into());
        self
    }

    /// Sets the contract ID which will receive all remaining hbars.
    pub fn transfer_contract_id(&mut self, id: ContractId) -> &mut Self {
        self.body.data.transfer_contract_id = Some(id);
        self
    }
}

#[async_trait]
impl TransactionExecute for ContractDeleteTransactionData {
    async fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> Result<tonic::Response<services::TransactionResponse>, tonic::Status> {
        SmartContractServiceClient::new(channel).delete_contract(request).await
    }
}

impl ToTransactionDataProtobuf for ContractDeleteTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        _node_account_id: AccountId,
        _transaction_id: &crate::TransactionId,
    ) -> services::transaction_body::Data {
        let delete_contract_id = self.delete_contract_id.as_ref().map(ContractId::to_protobuf);

        let obtainers = match (&self.transfer_account_id, &self.transfer_contract_id) {
            (Some(account_id), None) => {
                Some(services::contract_delete_transaction_body::Obtainers::TransferAccountId(
                    account_id.to_protobuf(),
                ))
            }

            (None, Some(contract_id)) => {
                Some(services::contract_delete_transaction_body::Obtainers::TransferContractId(
                    contract_id.to_protobuf(),
                ))
            }

            _ => None,
        };

        services::transaction_body::Data::ContractDeleteInstance(
            services::ContractDeleteTransactionBody {
                contract_id: delete_contract_id,
                permanent_removal: false,
                obtainers,
            },
        )
    }
}

impl From<ContractDeleteTransactionData> for AnyTransactionData {
    fn from(transaction: ContractDeleteTransactionData) -> Self {
        Self::ContractDelete(transaction)
    }
}
