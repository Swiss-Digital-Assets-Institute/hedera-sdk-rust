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

use std::collections::HashMap;

use hedera_proto::services;
use time::OffsetDateTime;

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    AssessedCustomFee,
    ContractFunctionResult,
    EvmAddress,
    FromProtobuf,
    Hbar,
    PublicKey,
    ScheduleId,
    Tinybar,
    TokenAssociation,
    TokenId,
    TokenNftTransfer,
    TransactionId,
    TransactionReceipt,
    Transfer,
};

/// The complete record for a transaction on Hedera that has reached consensus.
/// Response from [`TransactionRecordQuery`][crate::TransactionRecordQuery].
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    /// The status (reach consensus, or failed, or is unknown) and the ID of
    /// any new account/file/instance created.
    pub receipt: TransactionReceipt,

    /// The hash of the Transaction that executed (not the hash of any Transaction that failed for
    /// having a duplicate TransactionID).
    pub transaction_hash: Vec<u8>,

    /// The consensus timestamp.
    pub consensus_timestamp: OffsetDateTime,

    /// Record of the value returned by the smart contract function or constructor.
    pub contract_function_result: Option<ContractFunctionResult>,

    /// All hbar transfers as a result of this transaction, such as fees, or
    /// transfers performed by the transaction, or by a smart contract it calls,
    /// or by the creation of threshold records that it triggers.
    pub transfers: Vec<Transfer>,

    /// All fungible token transfers as a result of this transaction.
    pub token_transfers: HashMap<TokenId, HashMap<AccountId, i64>>,

    /// All NFT Token transfers as a result of this transaction.
    pub token_nft_transfers: HashMap<TokenId, Vec<TokenNftTransfer>>,

    /// The ID of the transaction this record represents.
    pub transaction_id: TransactionId,

    /// The memo that was submitted as part of the transaction.
    pub transaction_memo: String,

    /// The actual transaction fee charged.
    pub transaction_fee: Hbar,

    /// Reference to the scheduled transaction ID that this transaction record represents.
    pub schedule_ref: Option<ScheduleId>,

    /// All custom fees that were assessed during a [`TransferTransaction`](crate::TransferTransaction), and must be paid if the
    /// transaction status resolved to SUCCESS.
    pub assessed_custom_fees: Vec<AssessedCustomFee>,

    /// All token associations implicitly created while handling this transaction
    pub automatic_token_associations: Vec<TokenAssociation>,

    /// In the record of an internal transaction, the consensus timestamp of the user
    /// transaction that spawned it.
    pub parent_consensus_timestamp: Option<OffsetDateTime>,

    /// In the record of an internal CryptoCreate transaction triggered by a user
    /// transaction with a (previously unused) alias, the new account's alias.
    pub alias_key: Option<PublicKey>,

    /// The records of processing all child transaction spawned by the transaction with the given
    /// top-level id, in consensus order. Always empty if the top-level status is UNKNOWN.
    pub children: Vec<Self>,

    /// The records of processing all consensus transaction with the same id as the distinguished
    /// record above, in chronological order.
    pub duplicates: Vec<Self>,

    /// The keccak256 hash of the ethereumData. This field will only be populated for
    /// `EthereumTransaction`.
    #[cfg_attr(feature = "ffi", serde(with = "serde_with::As::<serde_with::base64::Base64>"))]
    pub ethereum_hash: Vec<u8>,

    /// In the record of a PRNG transaction with no output range, a pseudorandom 384-bit string.
    #[cfg_attr(
        feature = "ffi",
        serde(with = "serde_with::As::<Option<serde_with::base64::Base64>>")
    )]
    pub prng_bytes: Option<Vec<u8>>,

    /// In the record of a PRNG transaction with an output range, the output of a PRNG
    /// whose input was a 384-bit string.
    pub prng_number: Option<u32>,

    /// The last 20 bytes of the keccak-256 hash of a ECDSA_SECP256K1 primitive key.
    pub evm_address: Option<EvmAddress>,
}
// TODO: paid_staking_rewards

impl TransactionRecord {
    /// Create a new `TransactionRecord` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::<services::TransactionRecord>::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }

    fn from_protobuf(
        record: services::TransactionRecord,
        duplicates: Vec<Self>,
        children: Vec<Self>,
    ) -> crate::Result<Self> {
        use services::transaction_record::Body;
        let receipt = pb_getf!(record, receipt)?;
        let receipt = TransactionReceipt::from_protobuf(receipt)?;

        let consensus_timestamp = pb_getf!(record, consensus_timestamp)?;
        let transaction_id = pb_getf!(record, transaction_id)?;
        let schedule_ref = Option::from_protobuf(record.schedule_ref)?;
        let parent_consensus_timestamp = record.parent_consensus_timestamp.map(Into::into);

        let alias_key = PublicKey::from_alias_bytes(&record.alias)?;

        let automatic_token_associations = Vec::from_protobuf(record.automatic_token_associations)?;

        let contract_function_result = record.body.map(|it| match it {
            Body::ContractCallResult(it) | Body::ContractCreateResult(it) => it,
        });

        let contract_function_result = Option::from_protobuf(contract_function_result)?;

        let transfers = record.transfer_list.map_or_else(Vec::new, |it| it.account_amounts);
        let transfers = Vec::from_protobuf(transfers)?;

        let (token_transfers, token_nft_transfers) = {
            let mut token_transfers = HashMap::with_capacity(record.token_transfer_lists.len());

            let mut token_nft_transfers: HashMap<TokenId, Vec<TokenNftTransfer>> =
                HashMap::with_capacity(record.token_transfer_lists.len());

            for transfer_list in record.token_transfer_lists {
                let token_id = pb_getf!(transfer_list, token)?;
                let token_id = TokenId::from_protobuf(token_id)?;

                // `.insert` would be the most idiomatic way, but this matches behavior with Java.
                let token_transfers = token_transfers
                    .entry(token_id)
                    .or_insert_with(|| HashMap::with_capacity(transfer_list.transfers.len()));

                for it in transfer_list.transfers {
                    let account_id = AccountId::from_protobuf(pb_getf!(it, account_id)?)?;
                    token_transfers.insert(account_id, it.amount);
                }

                let nft_transfers: Result<Vec<_>, _> = transfer_list
                    .nft_transfers
                    .into_iter()
                    .map(|it| TokenNftTransfer::from_protobuf(it, token_id))
                    .collect();
                let nft_transfers = nft_transfers?;

                token_nft_transfers.entry(token_id).or_default().extend_from_slice(&nft_transfers);
            }

            (token_transfers, token_nft_transfers)
        };

        let evm_address = if record.evm_address.is_empty() {
            None
        } else {
            Some(EvmAddress::try_from(record.evm_address)?)
        };

        let (prng_bytes, prng_number) = match record.entropy {
            Some(services::transaction_record::Entropy::PrngBytes(it)) => (Some(it), None),
            Some(services::transaction_record::Entropy::PrngNumber(it)) => (None, Some(it as u32)),
            None => (None, None),
        };

        Ok(Self {
            receipt,
            transaction_hash: record.transaction_hash,
            consensus_timestamp: consensus_timestamp.into(),
            contract_function_result,
            transaction_id: TransactionId::from_protobuf(transaction_id)?,
            transaction_memo: record.memo,
            transaction_fee: Hbar::from_tinybars(record.transaction_fee as Tinybar),
            schedule_ref,
            automatic_token_associations,
            parent_consensus_timestamp,
            duplicates,
            ethereum_hash: record.ethereum_hash,
            children,
            alias_key,
            transfers,
            token_transfers,
            token_nft_transfers,
            assessed_custom_fees: Vec::from_protobuf(record.assessed_custom_fees)?,
            evm_address,
            prng_bytes,
            prng_number,
        })
    }
}

impl FromProtobuf<services::response::Response> for TransactionRecord {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pb = pb_getv!(pb, TransactionGetRecord, services::response::Response);

        let record = pb_getf!(pb, transaction_record)?;

        let duplicates = Vec::from_protobuf(pb.duplicate_transaction_records)?;

        let children = Vec::from_protobuf(pb.child_transaction_records)?;

        Self::from_protobuf(record, duplicates, children)
    }
}

impl FromProtobuf<services::TransactionRecord> for TransactionRecord {
    fn from_protobuf(receipt: services::TransactionRecord) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::from_protobuf(receipt, Vec::new(), Vec::new())
    }
}

impl ToProtobuf for TransactionRecord {
    type Protobuf = services::TransactionRecord;

    fn to_protobuf(&self) -> Self::Protobuf {
        let entropy = self
            .prng_number
            .map(|it| services::transaction_record::Entropy::PrngNumber(it as i32))
            .or_else(|| {
                self.prng_bytes.clone().map(services::transaction_record::Entropy::PrngBytes)
            });

        let token_transfer_lists = self
            .token_transfers
            .iter()
            .map(|it| services::TokenTransferList {
                token: Some(it.0.to_protobuf()),
                transfers: it
                    .1
                    .iter()
                    .map(|it| services::AccountAmount {
                        account_id: Some(it.0.to_protobuf()),
                        amount: *it.1,
                        is_approval: false,
                    })
                    .collect(),
                nft_transfers: Vec::new(),
                expected_decimals: None,
            })
            .collect();

        services::TransactionRecord {
            receipt: Some(self.receipt.to_protobuf()),
            transaction_hash: self.transaction_hash.clone(),
            consensus_timestamp: Some(self.consensus_timestamp.to_protobuf()),
            transaction_id: Some(self.transaction_id.to_protobuf()),
            memo: self.transaction_memo.clone(),
            transaction_fee: self.transaction_fee.to_tinybars() as u64,
            transfer_list: Some(services::TransferList {
                account_amounts: self.transfers.iter().map(|it| it.to_protobuf()).collect(),
            }),
            token_transfer_lists: token_transfer_lists,
            schedule_ref: self.schedule_ref.to_protobuf(),
            assessed_custom_fees: self.assessed_custom_fees.to_protobuf(),
            automatic_token_associations: self.automatic_token_associations.to_protobuf(),
            parent_consensus_timestamp: self.parent_consensus_timestamp.to_protobuf(),
            alias: self.alias_key.as_ref().map(ToProtobuf::to_bytes).unwrap_or_default(),
            ethereum_hash: self.ethereum_hash.clone(),
            // TODO:
            paid_staking_rewards: Vec::new(),
            evm_address: self
                .evm_address
                .as_ref()
                .map_or_else(Vec::default, |it| it.to_bytes().to_vec()),
            body: self
                .contract_function_result
                .as_ref()
                .map(|it| services::transaction_record::Body::ContractCallResult(it.to_protobuf())),
            entropy,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use expect_test::expect;

    use crate::protobuf::ToProtobuf;
    use crate::transaction::test_helpers::{
        TEST_TX_ID,
        VALID_START,
    };
    use crate::{
        AccountId,
        AssessedCustomFee,
        ContractFunctionResult,
        ContractId,
        Hbar,
        PrivateKey,
        ScheduleId,
        TokenAssociation,
        TokenId,
        TokenNftTransfer,
        TransactionRecord,
        Transfer,
    };

    fn make_record(prng_bytes: Option<Vec<u8>>, prng_number: Option<u32>) -> TransactionRecord {
        TransactionRecord {
            receipt: crate::transaction_receipt::make_receipt(),
            transaction_hash: b"hello".to_vec(),
            consensus_timestamp: VALID_START,
            contract_function_result: Some(ContractFunctionResult {
                contract_id: ContractId::new(1, 2, 3),
                evm_address: Some(ContractId::new(1, 2, 3)),
                bytes: Vec::new(),
                error_message: None,
                bloom: Vec::new(),
                gas_used: 0,
                gas: 0,
                hbar_amount: 0,
                contract_function_parameters_bytes: Vec::new(),
                sender_account_id: Some(AccountId::new(1, 2, 3)),
                logs: Vec::new(),
                contract_nonces: Vec::new(),
            }),
            transfers: Vec::from([Transfer {
                account_id: AccountId::new(4, 4, 4),
                amount: Hbar::new(5),
            }]),
            token_transfers: HashMap::from([(
                TokenId::new(6, 6, 6),
                HashMap::from([(AccountId::new(1, 1, 1), 4)]),
            )]),
            token_nft_transfers: HashMap::from([(
                TokenId::new(4, 4, 4),
                Vec::from([TokenNftTransfer {
                    token_id: TokenId::new(4, 4, 4),
                    sender: AccountId::new(1, 2, 3),
                    receiver: AccountId::new(3, 2, 1),
                    serial: 4,
                    is_approved: true,
                }]),
            )]),
            transaction_id: TEST_TX_ID,
            transaction_memo: "memo".to_owned(),
            transaction_fee: Hbar::from_tinybars(3000),
            schedule_ref: Some(ScheduleId::new(3, 3, 3)),
            assessed_custom_fees: Vec::from([AssessedCustomFee {
                amount: 4,
                token_id: Some(TokenId::new(4, 5, 6)),
                fee_collector_account_id: Some(AccountId::new(8, 6, 5)),
                payer_account_id_list: Vec::from([AccountId::new(3, 3, 3)]),
            }]),
            automatic_token_associations: Vec::from([TokenAssociation {
                token_id: TokenId::new(5, 4, 3),
                account_id: AccountId::new(8, 7, 6),
            }]),
            parent_consensus_timestamp: Some(VALID_START),
            alias_key: Some(
                PrivateKey::from_str_ecdsa(
                    "8776c6b831a1b61ac10dac0304a2843de4716f54b1919bb91a2685d0fe3f3048",
                )
                .unwrap()
                .public_key()
                .into(),
            ),
            children: Vec::new(),
            duplicates: Vec::new(),
            ethereum_hash: b"Some hash".to_vec(),
            prng_bytes,
            prng_number,
            evm_address: Some(crate::EvmAddress([0; 20])),
        }
    }

    #[test]
    fn serialize() {
        expect![[r#"
            TransactionRecord {
                receipt: Some(
                    TransactionReceipt {
                        status: ScheduleAlreadyDeleted,
                        account_id: Some(
                            AccountId {
                                shard_num: 1,
                                realm_num: 2,
                                account: Some(
                                    AccountNum(
                                        3,
                                    ),
                                ),
                            },
                        ),
                        file_id: Some(
                            FileId {
                                shard_num: 4,
                                realm_num: 5,
                                file_num: 6,
                            },
                        ),
                        contract_id: Some(
                            ContractId {
                                shard_num: 3,
                                realm_num: 2,
                                contract: Some(
                                    ContractNum(
                                        1,
                                    ),
                                ),
                            },
                        ),
                        exchange_rate: None,
                        topic_id: Some(
                            TopicId {
                                shard_num: 9,
                                realm_num: 8,
                                topic_num: 7,
                            },
                        ),
                        topic_sequence_number: 3,
                        topic_running_hash: [
                            104,
                            111,
                            119,
                            32,
                            110,
                            111,
                            119,
                            32,
                            98,
                            114,
                            111,
                            119,
                            110,
                            32,
                            99,
                            111,
                            119,
                        ],
                        topic_running_hash_version: 0,
                        token_id: Some(
                            TokenId {
                                shard_num: 6,
                                realm_num: 5,
                                token_num: 4,
                            },
                        ),
                        new_total_supply: 30,
                        schedule_id: Some(
                            ScheduleId {
                                shard_num: 1,
                                realm_num: 1,
                                schedule_num: 1,
                            },
                        ),
                        scheduled_transaction_id: Some(
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
                        serial_numbers: [
                            1,
                            2,
                            3,
                        ],
                    },
                ),
                transaction_hash: [
                    104,
                    101,
                    108,
                    108,
                    111,
                ],
                consensus_timestamp: Some(
                    Timestamp {
                        seconds: 1554158542,
                        nanos: 0,
                    },
                ),
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
                memo: "memo",
                transaction_fee: 3000,
                transfer_list: Some(
                    TransferList {
                        account_amounts: [
                            AccountAmount {
                                account_id: Some(
                                    AccountId {
                                        shard_num: 4,
                                        realm_num: 4,
                                        account: Some(
                                            AccountNum(
                                                4,
                                            ),
                                        ),
                                    },
                                ),
                                amount: 500000000,
                                is_approval: false,
                            },
                        ],
                    },
                ),
                token_transfer_lists: [
                    TokenTransferList {
                        token: Some(
                            TokenId {
                                shard_num: 6,
                                realm_num: 6,
                                token_num: 6,
                            },
                        ),
                        transfers: [
                            AccountAmount {
                                account_id: Some(
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
                                amount: 4,
                                is_approval: false,
                            },
                        ],
                        nft_transfers: [],
                        expected_decimals: None,
                    },
                ],
                schedule_ref: Some(
                    ScheduleId {
                        shard_num: 3,
                        realm_num: 3,
                        schedule_num: 3,
                    },
                ),
                assessed_custom_fees: [
                    AssessedCustomFee {
                        amount: 4,
                        token_id: Some(
                            TokenId {
                                shard_num: 4,
                                realm_num: 5,
                                token_num: 6,
                            },
                        ),
                        fee_collector_account_id: Some(
                            AccountId {
                                shard_num: 8,
                                realm_num: 6,
                                account: Some(
                                    AccountNum(
                                        5,
                                    ),
                                ),
                            },
                        ),
                        effective_payer_account_id: [
                            AccountId {
                                shard_num: 3,
                                realm_num: 3,
                                account: Some(
                                    AccountNum(
                                        3,
                                    ),
                                ),
                            },
                        ],
                    },
                ],
                automatic_token_associations: [
                    TokenAssociation {
                        token_id: Some(
                            TokenId {
                                shard_num: 5,
                                realm_num: 4,
                                token_num: 3,
                            },
                        ),
                        account_id: Some(
                            AccountId {
                                shard_num: 8,
                                realm_num: 7,
                                account: Some(
                                    AccountNum(
                                        6,
                                    ),
                                ),
                            },
                        ),
                    },
                ],
                parent_consensus_timestamp: Some(
                    Timestamp {
                        seconds: 1554158542,
                        nanos: 0,
                    },
                ),
                alias: [
                    58,
                    33,
                    2,
                    112,
                    58,
                    147,
                    112,
                    176,
                    68,
                    59,
                    230,
                    174,
                    124,
                    80,
                    123,
                    10,
                    236,
                    129,
                    165,
                    94,
                    148,
                    228,
                    168,
                    99,
                    185,
                    101,
                    83,
                    96,
                    189,
                    101,
                    53,
                    140,
                    170,
                    101,
                    136,
                ],
                ethereum_hash: [
                    83,
                    111,
                    109,
                    101,
                    32,
                    104,
                    97,
                    115,
                    104,
                ],
                paid_staking_rewards: [],
                evm_address: [
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                body: Some(
                    ContractCallResult(
                        ContractFunctionResult {
                            contract_id: Some(
                                ContractId {
                                    shard_num: 1,
                                    realm_num: 2,
                                    contract: Some(
                                        ContractNum(
                                            3,
                                        ),
                                    ),
                                },
                            ),
                            contract_call_result: [],
                            error_message: "",
                            bloom: [],
                            gas_used: 0,
                            log_info: [],
                            created_contract_i_ds: [],
                            evm_address: None,
                            gas: 0,
                            amount: 0,
                            function_parameters: [],
                            sender_id: Some(
                                AccountId {
                                    shard_num: 1,
                                    realm_num: 2,
                                    account: Some(
                                        AccountNum(
                                            3,
                                        ),
                                    ),
                                },
                            ),
                            contract_nonces: [],
                        },
                    ),
                ),
                entropy: Some(
                    PrngBytes(
                        [
                            118,
                            101,
                            114,
                            121,
                            32,
                            114,
                            97,
                            110,
                            100,
                            111,
                            109,
                            32,
                            98,
                            121,
                            116,
                            101,
                            115,
                        ],
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(&make_record(Some(b"very random bytes".to_vec()), None).to_protobuf())
    }

    #[test]
    fn to_from_bytes() {
        let a = make_record(Some(b"very random bytes".to_vec()), None);
        let b = TransactionRecord::from_bytes(&a.to_bytes()).unwrap();

        assert_eq!(a.to_protobuf(), b.to_protobuf());
    }

    #[test]
    fn serialize2() {
        expect![[r#"
            TransactionRecord {
                receipt: Some(
                    TransactionReceipt {
                        status: ScheduleAlreadyDeleted,
                        account_id: Some(
                            AccountId {
                                shard_num: 1,
                                realm_num: 2,
                                account: Some(
                                    AccountNum(
                                        3,
                                    ),
                                ),
                            },
                        ),
                        file_id: Some(
                            FileId {
                                shard_num: 4,
                                realm_num: 5,
                                file_num: 6,
                            },
                        ),
                        contract_id: Some(
                            ContractId {
                                shard_num: 3,
                                realm_num: 2,
                                contract: Some(
                                    ContractNum(
                                        1,
                                    ),
                                ),
                            },
                        ),
                        exchange_rate: None,
                        topic_id: Some(
                            TopicId {
                                shard_num: 9,
                                realm_num: 8,
                                topic_num: 7,
                            },
                        ),
                        topic_sequence_number: 3,
                        topic_running_hash: [
                            104,
                            111,
                            119,
                            32,
                            110,
                            111,
                            119,
                            32,
                            98,
                            114,
                            111,
                            119,
                            110,
                            32,
                            99,
                            111,
                            119,
                        ],
                        topic_running_hash_version: 0,
                        token_id: Some(
                            TokenId {
                                shard_num: 6,
                                realm_num: 5,
                                token_num: 4,
                            },
                        ),
                        new_total_supply: 30,
                        schedule_id: Some(
                            ScheduleId {
                                shard_num: 1,
                                realm_num: 1,
                                schedule_num: 1,
                            },
                        ),
                        scheduled_transaction_id: Some(
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
                        serial_numbers: [
                            1,
                            2,
                            3,
                        ],
                    },
                ),
                transaction_hash: [
                    104,
                    101,
                    108,
                    108,
                    111,
                ],
                consensus_timestamp: Some(
                    Timestamp {
                        seconds: 1554158542,
                        nanos: 0,
                    },
                ),
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
                memo: "memo",
                transaction_fee: 3000,
                transfer_list: Some(
                    TransferList {
                        account_amounts: [
                            AccountAmount {
                                account_id: Some(
                                    AccountId {
                                        shard_num: 4,
                                        realm_num: 4,
                                        account: Some(
                                            AccountNum(
                                                4,
                                            ),
                                        ),
                                    },
                                ),
                                amount: 500000000,
                                is_approval: false,
                            },
                        ],
                    },
                ),
                token_transfer_lists: [
                    TokenTransferList {
                        token: Some(
                            TokenId {
                                shard_num: 6,
                                realm_num: 6,
                                token_num: 6,
                            },
                        ),
                        transfers: [
                            AccountAmount {
                                account_id: Some(
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
                                amount: 4,
                                is_approval: false,
                            },
                        ],
                        nft_transfers: [],
                        expected_decimals: None,
                    },
                ],
                schedule_ref: Some(
                    ScheduleId {
                        shard_num: 3,
                        realm_num: 3,
                        schedule_num: 3,
                    },
                ),
                assessed_custom_fees: [
                    AssessedCustomFee {
                        amount: 4,
                        token_id: Some(
                            TokenId {
                                shard_num: 4,
                                realm_num: 5,
                                token_num: 6,
                            },
                        ),
                        fee_collector_account_id: Some(
                            AccountId {
                                shard_num: 8,
                                realm_num: 6,
                                account: Some(
                                    AccountNum(
                                        5,
                                    ),
                                ),
                            },
                        ),
                        effective_payer_account_id: [
                            AccountId {
                                shard_num: 3,
                                realm_num: 3,
                                account: Some(
                                    AccountNum(
                                        3,
                                    ),
                                ),
                            },
                        ],
                    },
                ],
                automatic_token_associations: [
                    TokenAssociation {
                        token_id: Some(
                            TokenId {
                                shard_num: 5,
                                realm_num: 4,
                                token_num: 3,
                            },
                        ),
                        account_id: Some(
                            AccountId {
                                shard_num: 8,
                                realm_num: 7,
                                account: Some(
                                    AccountNum(
                                        6,
                                    ),
                                ),
                            },
                        ),
                    },
                ],
                parent_consensus_timestamp: Some(
                    Timestamp {
                        seconds: 1554158542,
                        nanos: 0,
                    },
                ),
                alias: [
                    58,
                    33,
                    2,
                    112,
                    58,
                    147,
                    112,
                    176,
                    68,
                    59,
                    230,
                    174,
                    124,
                    80,
                    123,
                    10,
                    236,
                    129,
                    165,
                    94,
                    148,
                    228,
                    168,
                    99,
                    185,
                    101,
                    83,
                    96,
                    189,
                    101,
                    53,
                    140,
                    170,
                    101,
                    136,
                ],
                ethereum_hash: [
                    83,
                    111,
                    109,
                    101,
                    32,
                    104,
                    97,
                    115,
                    104,
                ],
                paid_staking_rewards: [],
                evm_address: [
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                body: Some(
                    ContractCallResult(
                        ContractFunctionResult {
                            contract_id: Some(
                                ContractId {
                                    shard_num: 1,
                                    realm_num: 2,
                                    contract: Some(
                                        ContractNum(
                                            3,
                                        ),
                                    ),
                                },
                            ),
                            contract_call_result: [],
                            error_message: "",
                            bloom: [],
                            gas_used: 0,
                            log_info: [],
                            created_contract_i_ds: [],
                            evm_address: None,
                            gas: 0,
                            amount: 0,
                            function_parameters: [],
                            sender_id: Some(
                                AccountId {
                                    shard_num: 1,
                                    realm_num: 2,
                                    account: Some(
                                        AccountNum(
                                            3,
                                        ),
                                    ),
                                },
                            ),
                            contract_nonces: [],
                        },
                    ),
                ),
                entropy: Some(
                    PrngNumber(
                        4,
                    ),
                ),
            }
        "#]]
        .assert_debug_eq(&make_record(None, Some(4)).to_protobuf())
    }

    #[test]
    fn to_from_bytes2() {
        let a = make_record(None, Some(4));
        let b = TransactionRecord::from_bytes(&a.to_bytes()).unwrap();

        assert_eq!(a.to_protobuf(), b.to_protobuf());
    }
}
