/// Wipes the provided amount of tokens from the specified account.
public class TokenWipeTransaction: Transaction {
    /// Create a new `TokenWipeTransaction`.
    public init(
        tokenId: TokenId? = nil,
        amount: UInt64 = 0,
        serialNumbers: [UInt64] = []
    ) {
        self.tokenId = tokenId
        self.amount = amount
        self.serialNumbers = serialNumbers
    }

    /// The token for which to wipe tokens.
    public var tokenId: TokenId?

    /// Sets the token for which to wipe tokens.
    @discardableResult
    public func tokenId(_ tokenId: TokenId) -> Self {
        self.tokenId = tokenId

        return self
    }

    /// The account to be wiped.
    public var accountId: AccountId?

    /// Sets the account to be wiped.
    @discardableResult
    public func accountId(_ accountId: AccountId) -> Self {
        self.accountId = accountId

        return self
    }

    /// The amount of a fungible token to wipe from the specified account.
    public var amount: UInt64

    //// Sets the amount of a fungible token to wipe from the specified account.
    @discardableResult
    public func amount(_ amount: UInt64) -> Self {
        self.amount = amount

        return self
    }

    /// The serial numbers of a non-fungible token to wipe from the specified account.
    public var serialNumbers: [UInt64]

    /// Sets the serial numbers of a non-fungible token to wipe from the specified account.
    @discardableResult
    public func serialNumbers(_ serialNumbers: [UInt64]) -> Self {
        self.serialNumbers = serialNumbers

        return self
    }

    private enum CodingKeys: String, CodingKey {
        case tokenId
        case amount
        case serialNumbers
    }

    public override func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(tokenId, forKey: .tokenId)
        try container.encode(amount, forKey: .amount)
        try container.encode(serialNumbers, forKey: .serialNumbers)

        try super.encode(to: encoder)
    }
}
