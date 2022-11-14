/*
 * ‌
 * Hedera Swift SDK
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

/// Marks a contract as deleted and transfers its remaining hBars, if any, to
/// a designated receiver.
public final class ContractDeleteTransaction: Transaction {
    /// Create a new `ContractDeleteTransaction`.
    public init(
        contractId: ContractId? = nil
    ) {
        self.contractId = contractId
    }

    /// The token to be deleted.
    public var contractId: ContractId?

    /// Sets the token to be deleted.
    @discardableResult
    public func contractId(_ contractId: ContractId) -> Self {
        self.contractId = contractId

        return self
    }

    private enum CodingKeys: String, CodingKey {
        case contractId
    }

    public override func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encodeIfPresent(contractId, forKey: .contractId)

        try super.encode(to: encoder)
    }
}