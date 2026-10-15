import CRhwpNative
import Foundation

public enum RhwpNativeHostError: Error {
    case failed(code: String, message: String)
    case invalidResponse
    case invalidBufferLength
}

/// Handle은 생성한 OS 스레드에서 사용하고 명시적으로 닫는다.
/// 반환된 픽셀 버퍼는 문서와 독립적으로 sessionBufferRelease로 해제한다.
public extension Rhwp {
    static func sessionCapabilities() throws -> [String: Any] {
        try consumeHostResult(rhwp_session_capabilities())
    }

    static func sessionOpen(data: Data = Data(), password: Data = Data()) throws -> [String: Any] {
        try data.withUnsafeBytes { bytes in
            try password.withUnsafeBytes { secret in
                try consumeHostResult(rhwp_session_open(
                    bytes.baseAddress?.assumingMemoryBound(to: UInt8.self), bytes.count,
                    secret.baseAddress?.assumingMemoryBound(to: UInt8.self), secret.count
                ))
            }
        }
    }

    static func sessionRequest(handle: UInt64, request: [String: Any]) throws -> [String: Any] {
        let data = try JSONSerialization.data(withJSONObject: request)
        return try data.withUnsafeBytes { bytes in
            try consumeHostResult(rhwp_session_request(
                handle, bytes.baseAddress?.assumingMemoryBound(to: UInt8.self), bytes.count
            ))
        }
    }

    static func hostRequest(request: [String: Any], data: Data = Data()) throws -> [String: Any] {
        let json = try JSONSerialization.data(withJSONObject: request)
        return try json.withUnsafeBytes { bytes in
            try data.withUnsafeBytes { payload in
                try consumeHostResult(rhwp_native_host_request(
                    bytes.baseAddress?.assumingMemoryBound(to: UInt8.self), bytes.count,
                    payload.baseAddress?.assumingMemoryBound(to: UInt8.self), payload.count
                ))
            }
        }
    }

    static func sessionClose(handle: UInt64) throws {
        _ = try consumeHostResult(rhwp_session_close(handle))
    }

    static func sessionBufferCopy(buffer: UInt64, byteLength: Int) throws -> Data {
        guard byteLength > 0, byteLength <= 128 * 1024 * 1024 else {
            throw RhwpNativeHostError.invalidBufferLength
        }
        var data = Data(count: byteLength)
        try data.withUnsafeMutableBytes { bytes in
            _ = try consumeHostResult(rhwp_session_buffer_copy(
                buffer, bytes.baseAddress?.assumingMemoryBound(to: UInt8.self), bytes.count
            ))
        }
        return data
    }

    static func sessionBufferRelease(buffer: UInt64) throws {
        _ = try consumeHostResult(rhwp_session_buffer_release(buffer))
    }

    private static func consumeHostResult(_ pointer: UnsafeMutablePointer<CChar>?) throws -> [String: Any] {
        guard let pointer else { throw RhwpError.nativeReturnedNull }
        defer { rhwp_string_free(pointer) }
        guard let text = String(validatingCString: pointer),
              let result = try JSONSerialization.jsonObject(with: Data(text.utf8)) as? [String: Any],
              let ok = result["ok"] as? Bool else {
            throw RhwpNativeHostError.invalidResponse
        }
        guard ok else {
            throw RhwpNativeHostError.failed(
                code: result["code"] as? String ?? "unknown",
                message: result["error"] as? String ?? "Native host request failed"
            )
        }
        return result
    }
}
