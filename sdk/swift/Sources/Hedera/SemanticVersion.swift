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

import Foundation
import CHedera

/// Hedera follows semantic versioning for both the HAPI protobufs and
/// the Services software.
public struct SemanticVersion: Codable, CustomStringConvertible {
    /// Increases with incompatible API changes
    public let major: UInt32

    /// Increases with backwards-compatible new functionality
    public let minor: UInt32

    /// Increases with backwards-compatible bug fixes
    public let patch: UInt32

    public var description: String {
        "\(major).\(minor)\(patch)"
    }

    public func toString() -> String {
        description
    }

    /// A pre-release version MAY be denoted by appending a hyphen and a series of dot separated identifiers (https://semver.org/#spec-item-9);
    /// so given a semver 0.14.0-alpha.1+21AF26D3, this field would contain ‘alpha.1’
    public let prerelease: String

    /// Build metadata MAY be denoted by appending a plus sign and a series of dot separated identifiers
    /// immediately following the patch or pre-release version (https://semver.org/#spec-item-10);
    /// so given a semver 0.14.0-alpha.1+21AF26D3, this field would contain ‘21AF26D3’
    public let build: String

    internal init(unsafeFromCHedera hedera: HederaSemanticVersion) {
        major = hedera.major
        minor = hedera.minor
        patch = hedera.patch
        prerelease = hedera.prerelease == nil ? "" : String(hString: hedera.prerelease!)
        build = hedera.build == nil ? "" : String(hString: hedera.build!)
    }

    internal func unsafeWithCHedera<Result>(_ body: (HederaSemanticVersion) throws -> Result) rethrows -> Result {
        try prerelease.withCString { (pre) in
            try build.withCString { (build) in
                let mutPrerelease = UnsafeMutablePointer(mutating: pre)
                let mutBuild = UnsafeMutablePointer(mutating: build)
                let csemver = HederaSemanticVersion(major: major, minor: minor, patch: patch, prerelease: mutPrerelease, build: mutBuild)

                return try body(csemver)
            }
        }
    }

    public static func fromBytes(_ bytes: Data) throws -> Self {
        try bytes.withUnsafeBytes { (pointer: UnsafeRawBufferPointer) in
            var semver = HederaSemanticVersion()

            let err = hedera_semantic_version_from_bytes(pointer.baseAddress, pointer.count, &semver);

            if err != HEDERA_ERROR_OK {
                throw HError(err)!
            }

            return Self(unsafeFromCHedera: semver)
        }
    }

    public func toBytes() -> Data {
        unsafeWithCHedera { info in
            var buf: UnsafeMutablePointer<UInt8>?
            let size = hedera_semantic_version_to_bytes(info, &buf)

            return Data(bytesNoCopy: buf!, count: size, deallocator: Data.unsafeCHederaBytesFree)
        }
    }
}

