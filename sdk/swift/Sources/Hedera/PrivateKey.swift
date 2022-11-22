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

import CHedera
import Foundation

private typealias UnsafeFromBytesFunc = @convention(c) (
    UnsafePointer<UInt8>?, Int, UnsafeMutablePointer<OpaquePointer?>?
) -> HederaError

/// A private key on the Hedera network.
public final class PrivateKey: LosslessStringConvertible, ExpressibleByStringLiteral {
    internal let ptr: OpaquePointer

    // sadly, we can't avoid a leaky abstraction here.
    internal static func unsafeFromPtr(_ ptr: OpaquePointer) -> Self {
        Self(ptr)
    }

    private init(_ ptr: OpaquePointer) {
        self.ptr = ptr
    }

    /// Generates a new Ed25519 private key.
    public static func generateEd25519() -> Self {
        self.init(hedera_private_key_generate_ed25519())
    }

    /// Generates a new ECDSA(secp256k1) private key.
    public static func generateEcdsa() -> Self {
        self.init(hedera_private_key_generate_ecdsa())
    }

    /// Gets the ``PublicKey`` which corresponds to this private key.
    public func getPublicKey() -> PublicKey {
        PublicKey.unsafeFromPtr(hedera_private_key_get_public_key(ptr))
    }

    private static func unsafeFromAnyBytes(_ bytes: Data, _ chederaCallback: UnsafeFromBytesFunc) throws -> Self {
        try bytes.withUnsafeTypedBytes { pointer -> Self in
            var key: OpaquePointer?
            let err = chederaCallback(pointer.baseAddress, pointer.count, &key)

            if err != HEDERA_ERROR_OK {
                throw HError(err)!
            }

            return Self(key!)
        }
    }

    public static func fromBytes(_ bytes: Data) throws -> Self {
        try unsafeFromAnyBytes(bytes, hedera_private_key_from_bytes)
    }

    public static func fromBytesEd25519(_ bytes: Data) throws -> Self {
        try unsafeFromAnyBytes(bytes, hedera_private_key_from_bytes_ed25519)
    }

    public static func fromBytesEcdsa(_ bytes: Data) throws -> Self {
        try unsafeFromAnyBytes(bytes, hedera_private_key_from_bytes_ed25519)
    }

    public static func fromBytesDer(_ bytes: Data) throws -> Self {
        try unsafeFromAnyBytes(bytes, hedera_private_key_from_bytes_ed25519)
    }

    public static func fromString(_ description: String) throws -> Self {
        var key: OpaquePointer?
        let err = hedera_private_key_from_string(description, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    public init?(_ description: String) {
        var key: OpaquePointer?
        let err = hedera_private_key_from_string(description, &key)

        if err != HEDERA_ERROR_OK {
            return nil
        }

        ptr = key!
    }

    public required convenience init(stringLiteral value: StringLiteralType) {
        self.init(value)!
    }

    public static func fromStringDer(_ description: String) throws -> Self {
        var key: OpaquePointer?
        let err = hedera_private_key_from_string_der(description, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    public static func fromStringEd25519(_ description: String) throws -> Self {
        var key: OpaquePointer?
        let err = hedera_private_key_from_string_ed25519(description, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    public static func fromStringEcdsa(_ description: String) throws -> Self {
        var key: OpaquePointer?
        let err = hedera_private_key_from_string_ecdsa(description, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    /// Parse a `PrivateKey` from [PEM](https://www.rfc-editor.org/rfc/rfc7468#section-10) encoded bytes.
    public static func fromPem(_ pem: String) throws -> Self {
        var key: OpaquePointer?
        let err = hedera_private_key_from_pem(pem, &key)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(key!)
    }

    public func toBytesDer() -> Data {
        var buf: UnsafeMutablePointer<UInt8>?
        let size = hedera_private_key_to_bytes_der(ptr, &buf)

        return Data(bytesNoCopy: buf!, count: size, deallocator: Data.unsafeCHederaBytesFree)
    }

    public func toBytes() -> Data {
        var buf: UnsafeMutablePointer<UInt8>?
        let size = hedera_private_key_to_bytes(ptr, &buf)

        return Data(bytesNoCopy: buf!, count: size, deallocator: Data.unsafeCHederaBytesFree)
    }

    public func toBytesRaw() -> Data {
        var buf: UnsafeMutablePointer<UInt8>?
        let size = hedera_private_key_to_bytes_raw(ptr, &buf)

        return Data(bytesNoCopy: buf!, count: size, deallocator: Data.unsafeCHederaBytesFree)
    }

    public var description: String {
        let descriptionBytes = hedera_private_key_to_string(ptr)
        return String(hString: descriptionBytes!)!
    }

    public func toString() -> String {
        description
    }

    public func toStringDer() -> String {
        let stringBytes = hedera_private_key_to_string_der(ptr)
        return String(hString: stringBytes!)!
    }

    public func toStringRaw() -> String {
        let stringBytes = hedera_private_key_to_string_raw(ptr)
        return String(hString: stringBytes!)!
    }

    public func toAccountId(shard: UInt64, realm: UInt64) -> AccountId {
        getPublicKey().toAccountId(shard: shard, realm: realm)
    }

    public func isEd25519() -> Bool {
        hedera_private_key_is_ed25519(ptr)
    }

    public func isEcdsa() -> Bool {
        hedera_private_key_is_ecdsa(ptr)
    }

    public func sign(_ message: Data) -> Data {
        message.withUnsafeTypedBytes { pointer in
            var buf: UnsafeMutablePointer<UInt8>?
            let size = hedera_private_key_sign(ptr, pointer.baseAddress, pointer.count, &buf)
            return Data(bytesNoCopy: buf!, count: size, deallocator: Data.unsafeCHederaBytesFree)
        }
    }

    public func isDerivable() -> Bool {
        hedera_private_key_is_derivable(ptr)
    }

    public func derive(_ index: Int32) throws -> Self {
        var derived: OpaquePointer?
        let err = hedera_private_key_derive(ptr, index, &derived)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(derived!)
    }

    public func legacyDerive(_ index: Int64) throws -> Self {
        var derived: OpaquePointer?
        let err = hedera_private_key_legacy_derive(ptr, index, &derived)

        if err != HEDERA_ERROR_OK {
            throw HError(err)!
        }

        return Self(derived!)
    }

    public static func fromMnemonic(_ mnemonic: Mnemonic, _ passphrase: String) -> Self {
        Self(hedera_private_key_from_mnemonic(mnemonic.ptr, passphrase))
    }

    public static func fromMnemonic(_ mnemonic: Mnemonic) -> Self {
        Self.fromMnemonic(mnemonic, "")
    }

    deinit {
        hedera_private_key_free(ptr)
    }
}
