#!/usr/bin/env swift
/// rhwp_write_text FFI 직접 호출 예제.
///
/// 사용법 (레포 루트에서):
///   cargo build --manifest-path bindings/Native/Cargo.toml --release
///   swift bindings/swift/Examples/write_text_ffi.swift
///
/// rhwp_write_text(outputPath, text) -> JSON 문자열 반환.
///
/// 반환 JSON 형식:
///   {"ok":true,"pageCount":1,"file":".../created.hwp","byteCount":12345}

import Foundation

typealias WriteTextFn = @convention(c) (UnsafePointer<CChar>, UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
typealias StringFreeFn = @convention(c) (UnsafeMutablePointer<CChar>) -> Void

let scriptDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let repoRoot = scriptDir
    .deletingLastPathComponent()
    .deletingLastPathComponent()
    .deletingLastPathComponent()
let libPath = repoRoot
    .appendingPathComponent("bindings/Native/target/release/librhwp_native_ffi.dylib")
    .path

guard let handle = dlopen(libPath, RTLD_NOW) else {
    let err = String(cString: dlerror())
    print("ERROR: dylib 로드 실패 — \(err)")
    print("       cargo build --manifest-path bindings/Native/Cargo.toml --release 를 먼저 실행하세요.")
    exit(1)
}

guard let writeSym = dlsym(handle, "rhwp_write_text"),
      let freeSym = dlsym(handle, "rhwp_string_free") else {
    print("ERROR: FFI 심볼을 찾을 수 없습니다.")
    exit(1)
}

let writeText = unsafeBitCast(writeSym, to: WriteTextFn.self)
let freeStr = unsafeBitCast(freeSym, to: StringFreeFn.self)

let outputPath: String
if CommandLine.arguments.count > 1 {
    outputPath = CommandLine.arguments[1]
} else {
    outputPath = repoRoot.appendingPathComponent("output/ffi-created.hwp").path
}

let body: String
if CommandLine.arguments.count > 2 {
    body = CommandLine.arguments.dropFirst(2).joined(separator: " ")
} else {
    body = "한글 English 123"
}

print("출력: \(outputPath)")
print("본문: \(body)")
print()

guard let resultPtr = outputPath.withCString({ outputPathPtr in
    body.withCString { bodyPtr in
        writeText(outputPathPtr, bodyPtr)
    }
}) else {
    print("ERROR: rhwp_write_text 반환값 null — 출력 경로를 확인하세요.")
    exit(1)
}

let jsonStr = String(cString: resultPtr)

if let data = jsonStr.data(using: .utf8),
   let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
   let ok = json["ok"] as? Bool, ok,
   let file = json["file"] as? String,
   let byteCount = json["byteCount"] as? Int {
    print("=== HWP 생성 성공 ===")
    print("파일: \(file)")
    print("크기: \(byteCount) bytes")
} else {
    print("JSON 파싱 실패 또는 에러 응답:")
    print(String(jsonStr.prefix(500)))
}

freeStr(resultPtr)
