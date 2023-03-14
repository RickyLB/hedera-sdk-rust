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

use fraction::Fraction;
use hedera_proto::services;

use crate::{
    AccountId,
    FromProtobuf,
    Hbar,
    ToProtobuf,
    TokenId,
};

#[cfg(test)]
mod tests;

/// Any `CustomFee`.
///
/// See the documentation for [`CustomFee`] and [`AnyCustomFeeData`].
pub type AnyCustomFee = CustomFee<Fee>;

/// A `FixedCustomFee`.
///
/// See the documentation for [`CustomFee`] and [`FixedFeeData`].
pub type FixedFee = CustomFee<FixedFeeData>;

/// A fractional `CustomFee`.
///
/// See the documentation for [`CustomFee`] and [`FractionalFeeData`].
pub type FractionalFee = CustomFee<FractionalFeeData>;

/// A royalty `CustomFee`.
///
/// See the documentation for [`CustomFee`] and [`RoyaltyFeeData`].
pub type RoyaltyFee = CustomFee<RoyaltyFeeData>;

/// A transfer fee to assess during a `CryptoTransfer` that transfers units of the token to which the
/// fee is attached. A custom fee may be either fixed or fractional, and must specify a fee collector
/// account to receive the assessed fees. Only positive fees may be assessed.
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase"))]
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CustomFee<Fee> {
    /// The fee to be charged
    #[cfg_attr(feature = "ffi", serde(flatten))]
    pub fee: Fee,

    /// The account to receive the custom fee.
    #[cfg_attr(feature = "ffi", serde(default, skip_serializing_if = "Option::is_none"))]
    pub fee_collector_account_id: Option<AccountId>,

    pub all_collectors_are_exempt: bool,
}

impl AnyCustomFee {
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::CustomFee> for AnyCustomFee {
    fn from_protobuf(pb: services::CustomFee) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let fee_collector_account_id = Option::from_protobuf(pb.fee_collector_account_id)?;
        let fee = pb_getf!(pb, fee)?;

        let fee: Fee = match fee {
            services::custom_fee::Fee::FixedFee(pb) => FixedFeeData::from_protobuf(pb)?.into(),
            services::custom_fee::Fee::FractionalFee(pb) => {
                FractionalFeeData::from_protobuf(pb)?.into()
            }

            services::custom_fee::Fee::RoyaltyFee(pb) => RoyaltyFeeData::from_protobuf(pb)?.into(),
        };

        Ok(Self {
            fee,
            fee_collector_account_id,
            all_collectors_are_exempt: pb.all_collectors_are_exempt,
        })
    }
}

impl ToProtobuf for AnyCustomFee {
    type Protobuf = services::CustomFee;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::CustomFee {
            fee_collector_account_id: self.fee_collector_account_id.to_protobuf(),
            fee: Some(self.fee.to_protobuf()),
            all_collectors_are_exempt: self.all_collectors_are_exempt,
        }
    }
}

impl From<FixedFee> for AnyCustomFee {
    fn from(v: FixedFee) -> Self {
        Self {
            fee: v.fee.into(),
            fee_collector_account_id: v.fee_collector_account_id,
            all_collectors_are_exempt: v.all_collectors_are_exempt,
        }
    }
}

impl From<FractionalFee> for AnyCustomFee {
    fn from(v: FractionalFee) -> Self {
        Self {
            fee: v.fee.into(),
            fee_collector_account_id: v.fee_collector_account_id,
            all_collectors_are_exempt: v.all_collectors_are_exempt,
        }
    }
}

impl From<RoyaltyFee> for AnyCustomFee {
    fn from(v: RoyaltyFee) -> Self {
        Self {
            fee: v.fee.into(),
            fee_collector_account_id: v.fee_collector_account_id,
            all_collectors_are_exempt: v.all_collectors_are_exempt,
        }
    }
}

/// Represents the possible fee types.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(tag = "$type", rename_all = "camelCase"))]
pub enum Fee {
    Fixed(FixedFeeData),
    Fractional(FractionalFeeData),
    Royalty(RoyaltyFeeData),
}

impl FromProtobuf<services::custom_fee::Fee> for Fee {
    fn from_protobuf(pb: services::custom_fee::Fee) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use services::custom_fee::Fee as ProtoFee;

        match pb {
            ProtoFee::FixedFee(it) => Ok(FixedFeeData::from_protobuf(it)?.into()),
            ProtoFee::FractionalFee(it) => Ok(FractionalFeeData::from_protobuf(it)?.into()),
            ProtoFee::RoyaltyFee(it) => Ok(RoyaltyFeeData::from_protobuf(it)?.into()),
        }
    }
}

impl ToProtobuf for Fee {
    type Protobuf = services::custom_fee::Fee;

    fn to_protobuf(&self) -> Self::Protobuf {
        use services::custom_fee::Fee as ProtoFee;
        match self {
            Self::Fixed(it) => ProtoFee::FixedFee(it.to_protobuf()),
            Self::Fractional(it) => ProtoFee::FractionalFee(it.to_protobuf()),
            Self::Royalty(it) => ProtoFee::RoyaltyFee(it.to_protobuf()),
        }
    }
}

impl From<FixedFeeData> for Fee {
    fn from(v: FixedFeeData) -> Self {
        Self::Fixed(v)
    }
}

impl From<FractionalFeeData> for Fee {
    fn from(v: FractionalFeeData) -> Self {
        Self::Fractional(v)
    }
}

impl From<RoyaltyFeeData> for Fee {
    fn from(v: RoyaltyFeeData) -> Self {
        Self::Royalty(v)
    }
}

/// A fixed number of units (hbar or token) to assess as a fee during a `CryptoTransfer` that transfers
/// units of the token to which this fixed fee is attached.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase"))]
pub struct FixedFeeData {
    /// The number of units to assess as a fee
    pub amount: i64,

    /// The denomination of the fee; taken as hbar if left unset and, in a TokenCreate, taken as the id
    /// of the newly created token if set to the sentinel value of 0.0.0
    pub denominating_token_id: TokenId,
}

impl FixedFeeData {
    #[must_use]
    pub fn from_hbar(amount: Hbar) -> Self {
        Self {
            amount: amount.to_tinybars(),
            denominating_token_id: TokenId { shard: 0, realm: 0, num: 0, checksum: None },
        }
    }

    #[must_use]
    pub fn get_hbar(&self) -> Option<Hbar> {
        (self.denominating_token_id == TokenId { shard: 0, realm: 0, num: 0, checksum: None })
            .then(|| Hbar::from_tinybars(self.amount))
    }
}

impl FromProtobuf<services::FixedFee> for FixedFeeData {
    fn from_protobuf(pb: services::FixedFee) -> crate::Result<Self> {
        Ok(Self {
            amount: pb.amount,
            denominating_token_id: TokenId::from_protobuf(pb_getf!(pb, denominating_token_id)?)?,
        })
    }
}

impl ToProtobuf for FixedFeeData {
    type Protobuf = services::FixedFee;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            amount: self.amount,
            denominating_token_id: Some(self.denominating_token_id.to_protobuf()),
        }
    }
}

/// A fraction of the transferred units of a token to assess as a fee. The amount assessed will never
/// be less than the given `minimum_amount`, and never greater than the given `maximum_amount`.  The
/// denomination is always units of the token to which this fractional fee is attached.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase"))]
pub struct FractionalFeeData {
    /// The denominator of the fraction of transferred units to assess as a fee
    pub denominator: u64,

    /// The numerator of the fraction of transferred units to assess as a fee
    pub numerator: u64,

    /// The minimum amount to assess
    pub minimum_amount: i64,

    /// The maximum amount to assess (zero implies no maximum)
    pub maximum_amount: i64,

    /// If [`Exclusive`](FeeAssessmentMethod::Exclusive),
    /// assesses the fee to the sender,
    /// so the receiver gets the full amount from the token transfer list,
    /// and the sender is charged an additional fee;
    /// if [`Inclusive`](FeeAssessmentMethod::Inclusive), the receiver does NOT get
    /// the full amount,
    /// but only what is left over after paying the fractional fee.
    pub assessment_method: FeeAssessmentMethod,
}

impl FromProtobuf<services::FractionalFee> for FractionalFeeData {
    fn from_protobuf(pb: services::FractionalFee) -> crate::Result<Self> {
        let amount = pb.fractional_amount.map(Fraction::from).unwrap_or_default();
        Ok(Self {
            denominator: *amount.denom().unwrap(),
            numerator: *amount.numer().unwrap(),
            assessment_method: match pb.net_of_transfers {
                true => FeeAssessmentMethod::Exclusive,
                false => FeeAssessmentMethod::Inclusive,
            },
            maximum_amount: pb.maximum_amount,
            minimum_amount: pb.minimum_amount,
        })
    }
}

impl ToProtobuf for FractionalFeeData {
    type Protobuf = services::FractionalFee;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            fractional_amount: Some(Fraction::new(self.numerator, self.denominator).into()),
            minimum_amount: self.minimum_amount,
            maximum_amount: self.maximum_amount,
            net_of_transfers: matches!(self.assessment_method, FeeAssessmentMethod::Exclusive),
        }
    }
}

/// A fee to assess during a `CryptoTransfer` that changes ownership of an NFT. Defines the fraction of
/// the fungible value exchanged for an NFT that the ledger should collect as a royalty. ("Fungible
/// value" includes both ℏ and units of fungible HTS tokens.) When the NFT sender does not receive
/// any fungible value, the ledger will assess the fallback fee, if present, to the new NFT owner.
/// Royalty fees can only be added to tokens of type type `NON_FUNGIBLE_UNIQUE`.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase"))]
pub struct RoyaltyFeeData {
    /// The denominator of the fraction of fungible value exchanged for an NFT to collect as royalty
    pub denominator: u64,

    /// The numerator of the fraction of fungible value exchanged for an NFT to collect as royalty
    pub numerator: u64,

    /// If present, the fixed fee to assess to the NFT receiver when no fungible value is exchanged
    /// with the sender
    #[cfg_attr(feature = "ffi", serde(default, skip_serializing_if = "Option::is_none"))]
    pub fallback_fee: Option<FixedFeeData>,
}

impl FromProtobuf<services::RoyaltyFee> for RoyaltyFeeData {
    fn from_protobuf(pb: services::RoyaltyFee) -> crate::Result<Self> {
        let amount = pb.exchange_value_fraction.unwrap_or_default();

        Ok(Self {
            denominator: amount.denominator as u64,
            numerator: amount.numerator as u64,
            fallback_fee: Option::from_protobuf(pb.fallback_fee)?,
        })
    }
}

impl ToProtobuf for RoyaltyFeeData {
    type Protobuf = services::RoyaltyFee;

    fn to_protobuf(&self) -> Self::Protobuf {
        Self::Protobuf {
            fallback_fee: self.fallback_fee.to_protobuf(),
            exchange_value_fraction: Some(services::Fraction {
                numerator: self.numerator as i64,
                denominator: self.denominator as i64,
            }),
        }
    }
}

/// Enum for the fee assessment method.
///
/// The terminology here (exclusive vs inclusive) is borrowed from tax assessment.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "ffi", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ffi", serde(rename_all = "camelCase"))]
pub enum FeeAssessmentMethod {
    /// The recipient recieves the transfer amount, minus the fee.
    ///
    /// If Alice is paying Bob, and an `Inclusive` fractional fee is collected to be sent to Charlie,
    /// the amount Alice declares she will pay in the transfer transaction *includes* the fee amount.
    Inclusive,

    /// The recipient recieves the whole transfer amount, and an extra fee is charged to the sender.
    ///
    /// If Alice is paying Bob, and an `Exclusive` fractional fee is collected to be sent to Charlie,
    /// the amount Alice declares she will pay in the transfer transaction *does not include* the fee amount.
    Exclusive,
}
