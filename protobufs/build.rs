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

use std::env;
use std::fs::{
    self,
    create_dir_all,
    read_dir,
};
use std::path::Path;

const DERIVE_EQ_HASH: &str = "#[derive(Eq, Hash)]";
const DERIVE_EQ_HASH_COPY: &str = "#[derive(Copy, Eq, Hash)]";
const SERVICES_FOLDER: &str = "./protobufs/services";

fn main() -> anyhow::Result<()> {
    // services is the "base" module for the hedera protobufs
    // in the beginning, there was only services and it was named "protos"

    let services_path = Path::new(SERVICES_FOLDER);

    if !services_path.is_dir() {
        anyhow::bail!("Folder {SERVICES_FOLDER} does not exist; do you need to `git submodule update --init`?");
    }

    let services: Vec<_> = read_dir(services_path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            entry.file_type().ok()?.is_file().then(|| entry.path())
        })
        .collect();

    let mut cfg = tonic_build::configure().build_server(cfg!(feature = "server"));

    // most of the protobufs in "basic types" should be Eq + Hash + Copy
    // any protobufs that would typically be used as parameter, that meet the requirements of those
    // traits
    cfg = cfg
        .type_attribute("proto.ShardID", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.RealmID", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.AccountID", DERIVE_EQ_HASH)
        .type_attribute("proto.AccountID.account", DERIVE_EQ_HASH)
        .type_attribute("proto.FileID", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.ContractID", DERIVE_EQ_HASH)
        .type_attribute("proto.ContractID.contract", DERIVE_EQ_HASH)
        .type_attribute("proto.TransactionID", DERIVE_EQ_HASH)
        .type_attribute("proto.Timestamp", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.NftTransfer", DERIVE_EQ_HASH)
        .type_attribute("proto.Fraction", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.TopicID", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.TokenID", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.ScheduleID", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.FeeComponents", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.Key", DERIVE_EQ_HASH)
        .type_attribute("proto.KeyList", DERIVE_EQ_HASH)
        .type_attribute("proto.ThresholdKey", DERIVE_EQ_HASH)
        .type_attribute("proto.Key.key", DERIVE_EQ_HASH)
        .type_attribute("proto.SignaturePair", DERIVE_EQ_HASH)
        .type_attribute("proto.SignaturePair.signature", DERIVE_EQ_HASH)
        .type_attribute("proto.FeeData", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.TokenBalance", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.TokenAssociation", DERIVE_EQ_HASH)
        .type_attribute("proto.CryptoAllowance", DERIVE_EQ_HASH)
        .type_attribute("proto.TokenAllowance", DERIVE_EQ_HASH)
        .type_attribute("proto.GrantedCryptoAllowance", DERIVE_EQ_HASH)
        .type_attribute("proto.GrantedTokenAllowance", DERIVE_EQ_HASH)
        .type_attribute("proto.Duration", DERIVE_EQ_HASH_COPY)
        .type_attribute("proto.ServiceEndpoint", DERIVE_EQ_HASH);

    // the ResponseCodeEnum should be marked as #[non_exhaustive] so
    // adding variants does not trigger a breaking change
    cfg = cfg.type_attribute("proto.ResponseCodeEnum", "#[non_exhaustive]");

    // the ResponseCodeEnum is not documented in the proto source
    cfg = cfg.type_attribute(
        "proto.ResponseCodeEnum",
        r#"#[doc = "
 Returned in `TransactionReceipt`, `Error::PreCheckStatus`, and `Error::ReceiptStatus`.
 
 The success variant is `Success` which is what a `TransactionReceipt` will contain for a
 successful transaction.
     "]"#,
    );

    cfg.compile(&services, &["./protobufs/services/"])?;

    // NOTE: prost generates rust doc comments and fails to remove the leading * line
    remove_useless_comments(&Path::new(&env::var("OUT_DIR")?).join("proto.rs"))?;

    // mirror
    // NOTE: must be compiled in a separate folder otherwise it will overwrite the previous build

    let mirror_out_dir = Path::new(&env::var("OUT_DIR")?).join("mirror");
    create_dir_all(&mirror_out_dir)?;

    tonic_build::configure()
        .build_server(false)
        .extern_path(".proto.Timestamp", "crate::services::Timestamp")
        .extern_path(".proto.TopicID", "crate::services::TopicId")
        .extern_path(".proto.FileID", "crate::services::FileId")
        .extern_path(".proto.NodeAddress", "crate::services::NodeAddress")
        .extern_path(
            ".proto.ConsensusMessageChunkInfo",
            "crate::services::ConsensusMessageChunkInfo",
        )
        .out_dir(&mirror_out_dir)
        .compile(
            &[
                "./protobufs/mirror/consensus_service.proto",
                "./protobufs/mirror/mirror_network_service.proto",
            ],
            &["./protobufs/mirror/", "./protobufs/services/"],
        )?;

    remove_useless_comments(&mirror_out_dir.join("proto.rs"))?;

    let com_out_dir = Path::new(&env::var("OUT_DIR")?).join("com");
    create_dir_all(&com_out_dir)?;

    tonic_build::configure()
        // .build_server(false)
        .extern_path(
            ".com.hedera.hapi.node.addressbook.NodeCreateTransactionBody",
            "crate::services::NodeCreateTransactionBody",
        )
        .extern_path(
            ".com.hedera.hapi.node.addressbook.NodeUpdateTransactionBody",
            "crate::services::NodeUpdateTransactionBody",
        )
        .extern_path(
            ".com.hedera.hapi.node.addressbook.NodeDeleteTransactionBody",
            "crate::services::NodeDeleteTransactionBody",
        )
        .out_dir(&com_out_dir)
        .compile(
            &[
                "./protobufs/services/address_book_service.proto",
                "./protobufs/services/node_create.proto",
                "./protobufs/services/node_delete.proto",
                "./protobufs/services/node_update.proto",
            ],
            &["./protobufs/com/"],
        )?;

    remove_useless_comments(&com_out_dir.join("proto.rs"))?;

    // streams
    // NOTE: must be compiled in a separate folder otherwise it will overwrite the previous build

    let streams_out_dir = Path::new(&env::var("OUT_DIR")?).join("streams");
    create_dir_all(&streams_out_dir)?;

    // NOTE: **ALL** protobufs defined in basic_types must be specified here
    let cfg = tonic_build::configure();
    let cfg = builder::extern_basic_types(cfg);

    cfg.out_dir(&streams_out_dir).compile(
        &["./protobufs/streams/account_balance_file.proto"],
        &["./protobufs/streams/", "./protobufs/services/"],
    )?;

    // see note wrt services.
    remove_useless_comments(&streams_out_dir.join("proto.rs"))?;

    // sdk
    // NOTE: must be compiled in a separate folder otherwise it will overwrite the previous build
    let sdk_out_dir = Path::new(&env::var("OUT_DIR")?).join("sdk");
    create_dir_all(&sdk_out_dir)?;

    // note:
    // almost everything in services must be specified here.
    let cfg = tonic_build::configure();
    let cfg = builder::extern_basic_types(cfg)
        .services_same("AssessedCustomFee")
        .services_same("ConsensusCreateTopicTransactionBody")
        .services_same("ConsensusDeleteTopicTransactionBody")
        .services_same("ConsensusMessageChunkInfo")
        .services_same("ConsensusSubmitMessageTransactionBody")
        .services_same("ConsensusUpdateTopicTransactionBody")
        .services_same("ContractCallTransactionBody")
        .services_same("ContractCreateTransactionBody")
        .services_same("ContractDeleteTransactionBody")
        .services_same("ContractUpdateTransactionBody")
        .services_same("CryptoAddLiveHashTransactionBody")
        .services_same("CryptoApproveAllowanceTransactionBody")
        .services_same("CryptoCreateTransactionBody")
        .services_same("CryptoDeleteTransactionBody")
        .services_same("CryptoDeleteAllowanceTransactionBody")
        .services_same("CryptoTransferTransactionBody")
        .services_same("CryptoUpdateTransactionBody")
        .services_same("CryptoDeleteLiveHashTransactionBody")
        .services_same("CustomFee")
        .services_same("Duration")
        .services_same("EthereumTransactionBody")
        .services_same("FileAppendTransactionBody")
        .services_same("FileCreateTransactionBody")
        .services_same("FileDeleteTransactionBody")
        .services_same("FileUpdateTransactionBody")
        .services_same("FixedFee")
        .services_same("FractionalFee")
        .services_same("FreezeTransactionBody")
        .services_same("FreezeType")
        .services_same("LiveHash")
        .services_same("NftRemoveAllowance")
        .services_same("NodeStake")
        .services_same("NodeStakeUpdateTransactionBody")
        .services_same("RoyaltyFee")
        .services_same("SchedulableTransactionBody")
        .services_same("ScheduleCreateTransactionBody")
        .services_same("ScheduleDeleteTransactionBody")
        .services_same("ScheduleSignTransactionBody")
        .services_same("SystemDeleteTransactionBody")
        .services_same("SystemUndeleteTransactionBody")
        .services_same("TokenAssociateTransactionBody")
        .services_same("TokenBurnTransactionBody")
        .services_same("TokenCreateTransactionBody")
        .services_same("TokenDeleteTransactionBody")
        .services_same("TokenDissociateTransactionBody")
        .services_same("TokenFeeScheduleUpdateTransactionBody")
        .services_same("TokenFreezeAccountTransactionBody")
        .services_same("TokenGrantKycTransactionBody")
        .services_same("TokenMintTransactionBody")
        .services_same("TokenPauseTransactionBody")
        .services_same("TokenRevokeKycTransactionBody")
        .services_same("TokenUnfreezeAccountTransactionBody")
        .services_same("TokenUnpauseTransactionBody")
        .services_same("TokenUpdateTransactionBody")
        .services_same("TokenUpdateNftsTransactionBody")
        .services_same("TokenWipeAccountTransactionBody")
        .services_same("Transaction")
        .services_same("TransactionBody")
        .services_same("UncheckedSubmitBody")
        .services_same("UtilPrngTransactionBody")
        .services_same("VirtualAddress");

    cfg.out_dir(&sdk_out_dir).compile(
        &["./protobufs/sdk/transaction_list.proto"],
        &["./protobufs/sdk/", "./protobufs/services/"],
    )?;

    // see note wrt services.
    remove_useless_comments(&sdk_out_dir.join("proto.rs"))?;

    // let addressbook_out_dir = Path::new(&env::var("OUT_DIR")?).join("addressbook");
    // create_dir_all(&addressbook_out_dir)?;

    // modify_addressbook_package("./protobufs/services/node_create.proto");

    // tonic_build::configure()
    //     .build_server(false)
    //     // Add any necessary extern_path configurations
    //     .extern_path(".com.hedera.hapi.node.addressbook.NodeCreateTransactionBody", "crate::services::NodeCreateTransactionBody")
    //     .extern_path(".com.hedera.hapi.node.addressbook.NodeDeleteTransactionBody", "crate::services::NodeDeleteTransactionBody")
    //     .extern_path(".com.hedera.hapi.node.addressbook.NodeUpdateTransactionBody", "crate::services::NodeUpdateTransactionBody")
    //     // .extern_path(".node_create.proto", "proto")
    //     // .extern_path(".node_delete.proto", "proto")
    //     // .extern_path(".node_update.proto", "proto")
    //     .out_dir(&addressbook_out_dir)
    //     .compile(
    //         &[
    //             "./protobufs/services/node_create.proto",
    //             "./protobufs/services/node_delete.proto",
    //             "./protobufs/services/node_update.proto",
    //         ],
    //         &["./protobufs/services/"],
    //     )?;

    // remove_useless_comments(&addressbook_out_dir.join("proto.rs"))?;

    Ok(())
}

fn remove_useless_comments(path: &Path) -> anyhow::Result<()> {
    let mut contents = fs::read_to_string(path)?;

    contents = contents.replace("///*\n", "");
    contents = contents.replace("/// *\n", "");
    contents = contents.replace("/// UNDOCUMENTED", "");

    fs::write(path, contents)?;

    Ok(())
}

// fn modify_addressbook_package(path: &String) -> anyhow::Result<()> {
//     contents = contents.replace("package com.hedera.hapi.node.addressbook;", "package proto;");

//     fs::write(path, contents)?;

//     Ok(())
// }

trait BuilderExtensions {
    fn services_path<T: AsRef<str>, U: AsRef<str>>(self, proto_name: T, rust_name: U) -> Self
    where
        Self: Sized;

    fn services_same<T: AsRef<str>>(self, name: T) -> Self
    where
        Self: Sized,
    {
        self.services_path(&name, &name)
    }
}

impl BuilderExtensions for tonic_build::Builder {
    fn services_path<T: AsRef<str>, U: AsRef<str>>(self, proto_name: T, rust_name: U) -> Self {
        let proto_name = proto_name.as_ref();
        let rust_name = rust_name.as_ref();

        self.extern_path(format!(".proto.{proto_name}"), format!("crate::services::{rust_name}"))
    }
}

mod builder {
    use crate::BuilderExtensions;

    pub(super) fn extern_basic_types(builder: tonic_build::Builder) -> tonic_build::Builder {
        builder
            .services_same("Fraction")
            .services_same("Timestamp")
            .services_path("AccountID", "AccountId")
            .services_path("TokenID", "TokenId")
            .services_same("AccountAmount")
            .services_same("CurrentAndNextFeeSchedule")
            .services_same("FeeComponents")
            .services_same("FeeData")
            .services_same("FeeSchedule")
            .services_same("Key")
            .services_path("FileID", "FileId")
            .services_same("KeyList")
            .services_same("NftTransfer")
            .services_same("NodeAddress")
            .services_same("NodeAddressBook")
            .services_path("RealmID", "RealmId")
            .services_path("ScheduleID", "ScheduleId")
            .services_path("SemanticVersion", "SemanticVersion")
            .services_path("ServiceEndpoint", "ServiceEndpoint")
            .services_same("ServicesConfigurationList")
            .services_path("Setting", "Setting")
            .services_path("ShardID", "ShardId")
            .services_path("Signature", "Signature")
            .services_path("SignatureList", "SignatureList")
            .services_path("SignatureMap", "SignatureMap")
            .services_path("SignaturePair", "SignaturePair")
            .services_path("ThresholdKey", "ThresholdKey")
            .services_path("ThresholdSignature", "ThresholdSignature")
            .services_path("TimestampSeconds", "TimestampSeconds")
            .services_path("TokenBalance", "TokenBalance")
            .services_path("TokenBalances", "TokenBalances")
            .services_path("TokenRelationship", "TokenRelationship")
            .services_path("TokenTransferList", "TokenTransferList")
            .services_path("TopicID", "TopicId")
            .services_path("TransactionFeeSchedule", "TransactionFeeSchedule")
            .services_path("TransactionID", "TransactionId")
            .services_path("TransferList", "TransferList")
            .services_path("HederaFunctionality", "HederaFunctionality")
            .services_path("SubType", "SubType")
            .services_path("TokenFreezeStatus", "TokenFreezeStatus")
            .services_path("TokenKycStatus", "TokenKycStatus")
            .services_path("TokenSupplyType", "TokenSupplyType")
            .services_path("TokenType", "TokenType")
            .services_path("GrantedCryptoAllowance", "GrantedCryptoAllowance")
            .services_path("GrantedTokenAllowance", "GrantedTokenAllowance")
            .services_path("CryptoAllowance", "CryptoAllowance")
            .services_path("TokenAllowance", "TokenAllowance")
            .services_path("GrantedNftAllowance", "GrantedNftAllowance")
            .services_path("NftAllowance", "NftAllowance")
            .services_path("TokenPauseStatus", "TokenPauseStatus")
            .services_path("TokenAssociation", "TokenAssociation")
            .services_path("ContractID", "ContractId")
            .services_path("StakingInfo", "StakingInfo")
    }
}
