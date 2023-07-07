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
use services::crypto_service_client::CryptoServiceClient;
use tonic::transport::Channel;

use crate::ledger_id::RefLedgerId;
use crate::protobuf::FromProtobuf;
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::{
    AccountId,
    BoxGrpcFuture,
    Error,
    Hbar,
    NftId,
    ToProtobuf,
    TokenId,
    Transaction,
    ValidateChecksums,
};

/// Creates one or more hbar/token approved allowances **relative to the owner account specified in the allowances of
/// this transaction**.
///
/// Each allowance grants a spender the right to transfer a pre-determined amount of the owner's
/// hbar/token to any other account of the spender's choice. If the owner is not specified in any
/// allowance, the payer of transaction is considered to be the owner for that particular allowance.
///
/// Setting the amount to zero will remove the respective allowance for the spender.
///
pub type AccountAllowanceApproveTransaction = Transaction<AccountAllowanceApproveTransactionData>;

#[derive(Debug, Clone, Default)]
pub struct AccountAllowanceApproveTransactionData {
    /// List of hbar allowances approved by the account owner.
    hbar_allowances: Vec<HbarAllowance>,

    /// List of fungible token allowances approved by the account owner.
    token_allowances: Vec<TokenAllowance>,

    /// List of non-fungible token allowances approved by the account owner.
    nft_allowances: Vec<NftAllowance>,
}

impl AccountAllowanceApproveTransaction {
    /// Approves the hbar allowance.
    pub fn approve_hbar_allowance(
        &mut self,
        owner_account_id: AccountId,
        spender_account_id: AccountId,
        amount: Hbar,
    ) -> &mut Self {
        self.data_mut().hbar_allowances.push(HbarAllowance {
            owner_account_id,
            spender_account_id,
            amount,
        });

        self
    }

    /// Returns the hbar allowances approved by the account owner.
    pub fn hbar_approvals(&self) -> &[HbarAllowance] {
        self.data().hbar_allowances.as_ref()
    }

    /// Approves the token allowance.
    pub fn approve_token_allowance(
        &mut self,
        token_id: TokenId,
        owner_account_id: AccountId,
        spender_account_id: AccountId,
        amount: u64,
    ) -> &mut Self {
        self.data_mut().token_allowances.push(TokenAllowance {
            token_id,
            owner_account_id,
            spender_account_id,
            amount,
        });
        self
    }

    /// Returns the fungible token allowances approved by the account owner
    pub fn token_approvals(&self) -> &[TokenAllowance] {
        self.data().token_allowances.as_ref()
    }

    /// Approve the NFT allowance.
    pub fn approve_token_nft_allowance(
        &mut self,
        nft_id: impl Into<NftId>,
        owner_account_id: AccountId,
        spender_account_id: AccountId,
    ) -> &mut Self {
        let nft_id = nft_id.into();
        let data = self.data_mut();

        if let Some(allowance) = data.nft_allowances.iter_mut().find(|allowance| {
            allowance.token_id == nft_id.token_id
                && allowance.owner_account_id == owner_account_id
                && allowance.spender_account_id == spender_account_id
                && allowance.approved_for_all.is_none()
        }) {
            allowance.serials.push(nft_id.serial as i64);
        } else {
            data.nft_allowances.push(NftAllowance {
                serials: vec![nft_id.serial as i64],
                token_id: nft_id.token_id,
                spender_account_id,
                owner_account_id,
                delegating_spender_account_id: None,
                approved_for_all: None,
            });
        };

        self
    }

    /// Approve the NFT allowance on all serial numbers (present and future).
    pub fn approve_token_nft_allowance_all_serials(
        &mut self,
        token_id: TokenId,
        owner_account_id: AccountId,
        spender_account_id: AccountId,
    ) -> &mut Self {
        self.data_mut().nft_allowances.push(NftAllowance {
            approved_for_all: Some(true),
            delegating_spender_account_id: None,
            spender_account_id,
            owner_account_id,
            token_id,
            serials: Vec::new(),
        });

        self
    }

    /// Returns the non-fungible token allowances approved by the account owner.
    pub fn token_nft_approvals(&self) -> &[NftAllowance] {
        self.data().nft_allowances.as_ref()
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HbarAllowance {
    /// The account ID of the hbar owner (ie. the grantor of the allowance).
    pub owner_account_id: AccountId,

    /// The account ID of the spender of the hbar allowance.
    pub spender_account_id: AccountId,

    /// The amount of the spender's allowance.
    pub amount: Hbar,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TokenAllowance {
    /// The token that the allowance pertains to.
    pub token_id: TokenId,

    /// The account ID of the token owner (ie. the grantor of the allowance).
    pub owner_account_id: AccountId,

    /// The account ID of the spender of the token allowance.
    pub spender_account_id: AccountId,

    /// The amount of the spender's token allowance.
    pub amount: u64,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct NftAllowance {
    /// The token that the allowance pertains to.
    pub token_id: TokenId,

    /// The account ID of the token owner (ie. the grantor of the allowance).
    pub owner_account_id: AccountId,

    /// The account ID of the spender of the token allowance.
    pub spender_account_id: AccountId,

    /// The list of serial numbers that the spender is permitted to transfer.
    pub serials: Vec<i64>,

    /// If true, the spender has access to all of the owner's NFT units of type tokenId (currently
    /// owned and any in the future).
    pub approved_for_all: Option<bool>,

    /// The account ID of the spender who is granted approvedForAll allowance and granting
    /// approval on an NFT serial to another spender.
    pub delegating_spender_account_id: Option<AccountId>,
}

impl TransactionData for AccountAllowanceApproveTransactionData {}

impl TransactionExecute for AccountAllowanceApproveTransactionData {
    fn execute(
        &self,
        channel: Channel,
        request: services::Transaction,
    ) -> BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { CryptoServiceClient::new(channel).approve_allowances(request).await })
    }
}

impl ValidateChecksums for AccountAllowanceApproveTransactionData {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        for hbar_allowance in &self.hbar_allowances {
            hbar_allowance.owner_account_id.validate_checksums(ledger_id)?;
            hbar_allowance.spender_account_id.validate_checksums(ledger_id)?;
        }

        for token_allowance in &self.token_allowances {
            token_allowance.token_id.validate_checksums(ledger_id)?;
            token_allowance.owner_account_id.validate_checksums(ledger_id)?;
            token_allowance.spender_account_id.validate_checksums(ledger_id)?;
        }

        for nft_allowance in &self.nft_allowances {
            nft_allowance.token_id.validate_checksums(ledger_id)?;
            nft_allowance.spender_account_id.validate_checksums(ledger_id)?;
            nft_allowance.owner_account_id.validate_checksums(ledger_id)?;
            if let Some(delegating_spender) = nft_allowance.delegating_spender_account_id {
                delegating_spender.validate_checksums(ledger_id)?;
            }
        }

        Ok(())
    }
}

impl ToTransactionDataProtobuf for AccountAllowanceApproveTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::CryptoApproveAllowance(self.to_protobuf())
    }
}

impl ToSchedulableTransactionDataProtobuf for AccountAllowanceApproveTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::CryptoApproveAllowance(self.to_protobuf())
    }
}

impl From<AccountAllowanceApproveTransactionData> for AnyTransactionData {
    fn from(transaction: AccountAllowanceApproveTransactionData) -> Self {
        Self::AccountAllowanceApprove(transaction)
    }
}

impl FromProtobuf<services::CryptoApproveAllowanceTransactionBody>
    for AccountAllowanceApproveTransactionData
{
    fn from_protobuf(pb: services::CryptoApproveAllowanceTransactionBody) -> crate::Result<Self> {
        Ok(Self {
            hbar_allowances: Vec::from_protobuf(pb.crypto_allowances)?,
            token_allowances: Vec::from_protobuf(pb.token_allowances)?,
            nft_allowances: Vec::from_protobuf(pb.nft_allowances)?,
        })
    }
}

impl ToProtobuf for AccountAllowanceApproveTransactionData {
    type Protobuf = services::CryptoApproveAllowanceTransactionBody;

    fn to_protobuf(&self) -> Self::Protobuf {
        let crypto_allowances = self.hbar_allowances.to_protobuf();

        let token_allowances = self.token_allowances.to_protobuf();

        let nft_allowances = self.nft_allowances.to_protobuf();

        services::CryptoApproveAllowanceTransactionBody {
            crypto_allowances,
            nft_allowances,
            token_allowances,
        }
    }
}

impl FromProtobuf<services::CryptoAllowance> for HbarAllowance {
    fn from_protobuf(pb: services::CryptoAllowance) -> crate::Result<Self> {
        Ok(Self {
            owner_account_id: AccountId::from_protobuf(pb_getf!(pb, owner)?)?,
            spender_account_id: AccountId::from_protobuf(pb_getf!(pb, spender)?)?,
            amount: Hbar::from_tinybars(pb.amount),
        })
    }
}

impl ToProtobuf for HbarAllowance {
    type Protobuf = services::CryptoAllowance;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            owner: Some(self.owner_account_id.to_protobuf()),
            spender: Some(self.spender_account_id.to_protobuf()),
            amount: self.amount.to_tinybars(),
        }
    }
}

impl FromProtobuf<services::TokenAllowance> for TokenAllowance {
    fn from_protobuf(pb: services::TokenAllowance) -> crate::Result<Self> {
        Ok(Self {
            token_id: TokenId::from_protobuf(pb_getf!(pb, token_id)?)?,
            owner_account_id: AccountId::from_protobuf(pb_getf!(pb, owner)?)?,
            spender_account_id: AccountId::from_protobuf(pb_getf!(pb, spender)?)?,
            amount: pb.amount as u64,
        })
    }
}

impl ToProtobuf for TokenAllowance {
    type Protobuf = services::TokenAllowance;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            token_id: Some(self.token_id.to_protobuf()),
            owner: Some(self.owner_account_id.to_protobuf()),
            spender: Some(self.spender_account_id.to_protobuf()),
            amount: self.amount as i64,
        }
    }
}

impl FromProtobuf<services::NftAllowance> for NftAllowance {
    fn from_protobuf(pb: services::NftAllowance) -> crate::Result<Self> {
        Ok(Self {
            token_id: TokenId::from_protobuf(pb_getf!(pb, token_id)?)?,
            owner_account_id: AccountId::from_protobuf(pb_getf!(pb, owner)?)?,
            spender_account_id: AccountId::from_protobuf(pb_getf!(pb, spender)?)?,
            serials: pb.serial_numbers,
            approved_for_all: pb.approved_for_all,
            delegating_spender_account_id: Option::from_protobuf(pb.delegating_spender)?,
        })
    }
}

impl ToProtobuf for NftAllowance {
    type Protobuf = services::NftAllowance;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            token_id: Some(self.token_id.to_protobuf()),
            owner: Some(self.owner_account_id.to_protobuf()),
            spender: Some(self.spender_account_id.to_protobuf()),
            serial_numbers: self.serials.clone(),
            approved_for_all: self.approved_for_all,
            delegating_spender: self.delegating_spender_account_id.to_protobuf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::transaction::test_helpers::{
        transaction_body,
        unused_private_key,
        VALID_START,
    };
    use crate::{
        AccountAllowanceApproveTransaction,
        AccountId,
        AnyTransaction,
        Hbar,
        TokenId,
        TransactionId,
    };

    fn make_transaction() -> AccountAllowanceApproveTransaction {
        let owner_id: AccountId = "5.6.7".parse().unwrap();
        let mut tx = AccountAllowanceApproveTransaction::new();

        let invalid_token_ids: [TokenId; 4] = [
            "2.2.2".parse().unwrap(),
            "4.4.4".parse().unwrap(),
            "6.6.6".parse().unwrap(),
            "8.8.8".parse().unwrap(),
        ];

        let invalid_account_ids: [AccountId; 5] = [
            "1.1.1".parse().unwrap(),
            "3.3.3".parse().unwrap(),
            "5.5.5".parse().unwrap(),
            "7.7.7".parse().unwrap(),
            "9.9.9".parse().unwrap(),
        ];

        tx.node_account_ids(["0.0.5005".parse().unwrap(), "0.0.5006".parse().unwrap()])
            .transaction_id(TransactionId {
                account_id: "5006".parse().unwrap(),
                valid_start: VALID_START,
                nonce: None,
                scheduled: false,
            })
            .approve_hbar_allowance(owner_id, invalid_account_ids[0], Hbar::new(3))
            .approve_token_allowance(invalid_token_ids[0], owner_id, invalid_account_ids[1], 6)
            .approve_token_nft_allowance(
                invalid_token_ids[1].nft(123),
                owner_id,
                invalid_account_ids[2],
            )
            .approve_token_nft_allowance(
                invalid_token_ids[1].nft(456),
                owner_id,
                invalid_account_ids[2],
            )
            .approve_token_nft_allowance(
                invalid_token_ids[3].nft(456),
                owner_id,
                invalid_account_ids[2],
            )
            .approve_token_nft_allowance(
                invalid_token_ids[1].nft(789),
                owner_id,
                invalid_account_ids[4],
            )
            .approve_token_nft_allowance_all_serials(
                invalid_token_ids[2],
                owner_id,
                invalid_account_ids[3],
            )
            .max_transaction_fee(Hbar::from_tinybars(100_000))
            .freeze()
            .unwrap()
            .sign(unused_private_key());

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        expect![[r#"
            TransactionBody {
                transaction_id: Some(
                    TransactionId {
                        transaction_valid_start: Some(
                            Timestamp {
                                seconds: 1554158542,
                                nanos: 0,
                            },
                        ),
                        account_id: Some(
                            AccountId {
                                shard_num: 0,
                                realm_num: 0,
                                account: Some(
                                    AccountNum(
                                        5006,
                                    ),
                                ),
                            },
                        ),
                        scheduled: false,
                        nonce: 0,
                    },
                ),
                node_account_id: Some(
                    AccountId {
                        shard_num: 0,
                        realm_num: 0,
                        account: Some(
                            AccountNum(
                                5005,
                            ),
                        ),
                    },
                ),
                transaction_fee: 100000,
                transaction_valid_duration: Some(
                    Duration {
                        seconds: 120,
                    },
                ),
                generate_record: false,
                memo: "",
                data: Some(
                    CryptoApproveAllowance(
                        CryptoApproveAllowanceTransactionBody {
                            crypto_allowances: [
                                CryptoAllowance {
                                    owner: Some(
                                        AccountId {
                                            shard_num: 5,
                                            realm_num: 6,
                                            account: Some(
                                                AccountNum(
                                                    7,
                                                ),
                                            ),
                                        },
                                    ),
                                    spender: Some(
                                        AccountId {
                                            shard_num: 1,
                                            realm_num: 1,
                                            account: Some(
                                                AccountNum(
                                                    1,
                                                ),
                                            ),
                                        },
                                    ),
                                    amount: 300000000,
                                },
                            ],
                            nft_allowances: [
                                NftAllowance {
                                    token_id: Some(
                                        TokenId {
                                            shard_num: 4,
                                            realm_num: 4,
                                            token_num: 4,
                                        },
                                    ),
                                    owner: Some(
                                        AccountId {
                                            shard_num: 5,
                                            realm_num: 6,
                                            account: Some(
                                                AccountNum(
                                                    7,
                                                ),
                                            ),
                                        },
                                    ),
                                    spender: Some(
                                        AccountId {
                                            shard_num: 5,
                                            realm_num: 5,
                                            account: Some(
                                                AccountNum(
                                                    5,
                                                ),
                                            ),
                                        },
                                    ),
                                    serial_numbers: [
                                        123,
                                        456,
                                    ],
                                    approved_for_all: None,
                                    delegating_spender: None,
                                },
                                NftAllowance {
                                    token_id: Some(
                                        TokenId {
                                            shard_num: 8,
                                            realm_num: 8,
                                            token_num: 8,
                                        },
                                    ),
                                    owner: Some(
                                        AccountId {
                                            shard_num: 5,
                                            realm_num: 6,
                                            account: Some(
                                                AccountNum(
                                                    7,
                                                ),
                                            ),
                                        },
                                    ),
                                    spender: Some(
                                        AccountId {
                                            shard_num: 5,
                                            realm_num: 5,
                                            account: Some(
                                                AccountNum(
                                                    5,
                                                ),
                                            ),
                                        },
                                    ),
                                    serial_numbers: [
                                        456,
                                    ],
                                    approved_for_all: None,
                                    delegating_spender: None,
                                },
                                NftAllowance {
                                    token_id: Some(
                                        TokenId {
                                            shard_num: 4,
                                            realm_num: 4,
                                            token_num: 4,
                                        },
                                    ),
                                    owner: Some(
                                        AccountId {
                                            shard_num: 5,
                                            realm_num: 6,
                                            account: Some(
                                                AccountNum(
                                                    7,
                                                ),
                                            ),
                                        },
                                    ),
                                    spender: Some(
                                        AccountId {
                                            shard_num: 9,
                                            realm_num: 9,
                                            account: Some(
                                                AccountNum(
                                                    9,
                                                ),
                                            ),
                                        },
                                    ),
                                    serial_numbers: [
                                        789,
                                    ],
                                    approved_for_all: None,
                                    delegating_spender: None,
                                },
                                NftAllowance {
                                    token_id: Some(
                                        TokenId {
                                            shard_num: 6,
                                            realm_num: 6,
                                            token_num: 6,
                                        },
                                    ),
                                    owner: Some(
                                        AccountId {
                                            shard_num: 5,
                                            realm_num: 6,
                                            account: Some(
                                                AccountNum(
                                                    7,
                                                ),
                                            ),
                                        },
                                    ),
                                    spender: Some(
                                        AccountId {
                                            shard_num: 7,
                                            realm_num: 7,
                                            account: Some(
                                                AccountNum(
                                                    7,
                                                ),
                                            ),
                                        },
                                    ),
                                    serial_numbers: [],
                                    approved_for_all: Some(
                                        true,
                                    ),
                                    delegating_spender: None,
                                },
                            ],
                            token_allowances: [
                                TokenAllowance {
                                    token_id: Some(
                                        TokenId {
                                            shard_num: 2,
                                            realm_num: 2,
                                            token_num: 2,
                                        },
                                    ),
                                    owner: Some(
                                        AccountId {
                                            shard_num: 5,
                                            realm_num: 6,
                                            account: Some(
                                                AccountNum(
                                                    7,
                                                ),
                                            ),
                                        },
                                    ),
                                    spender: Some(
                                        AccountId {
                                            shard_num: 3,
                                            realm_num: 3,
                                            account: Some(
                                                AccountNum(
                                                    3,
                                                ),
                                            ),
                                        },
                                    ),
                                    amount: 6,
                                },
                            ],
                        },
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn check_properties() {
        let tx = make_transaction();

        assert!(!tx.hbar_approvals().is_empty());
        assert!(!tx.token_approvals().is_empty());
        assert!(!tx.token_approvals().is_empty());
    }
}
