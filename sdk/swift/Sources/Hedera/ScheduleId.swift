/// The unique identifier for a schedule on Hedera.
public class ScheduleId: CustomStringConvertible {
    /// The shard number (non-negative).
    public let shard: UInt64

    /// The realm number (non-negative).
    public let realm: UInt64

    public let num: UInt64

    public init(num: UInt64, shard: UInt64 = 0, realm: UInt64 = 0) {
        self.num = num
        self.shard = shard
        self.realm = realm
    }

    public var description: String {
        "\(shard).\(realm).\(num)"
    }
}