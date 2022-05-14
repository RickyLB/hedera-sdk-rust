import CHedera
import Foundation

/// A query that can be executed on the Hedera network.
public class Query<O: Decodable>: Encodable {
    internal init() {}

    public func execute(_ client: Client) async throws -> O {
        // encode self as a JSON request to pass to Rust
        let requestBytes = try JSONEncoder().encode(self)
        let request = String(data: requestBytes, encoding: .utf8)!

        // start an unmanaged continuation to bridge a C callback with Swift async
        let responseBytes: Data = await withUnmanagedContinuation { continuation in
            // invoke `hedera_execute`, callback will be invoked on request completion
            hedera_execute(client.ptr, request, continuation) { continuation, responsePtr in
                // TODO: handle failures

                // NOTE: we are guaranteed to receive valid UTF-8 on a successful response
                let responseBytes = String(validatingUTF8: responsePtr!)!.data(using: .utf8)!

                // resumes the continuation which bridges us back into Swift async
                resumeUnmanagedContinuation(continuation, with: .success(responseBytes))
            }
        }

        // decode the response as the generic output type of this query types
        let response = try JSONDecoder().decode(O.self, from: responseBytes)

        return response
    }

    public func encode(to encoder: Encoder) throws {
        // TODO: encode payment transaction
        // TODO: var container = encoder.container(keyedBy: CodingKeys.self)
    }
}

//    private enum CodingKeys: String, CodingKey {
//        case payment
//    }