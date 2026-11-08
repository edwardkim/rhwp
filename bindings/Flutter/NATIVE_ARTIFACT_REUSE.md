# 기존 CI Native 산출물의 재사용 계약

`tool/native_artifact_provenance.py`의
`load_native_artifact(artifact_root, repository, platform, expected_source)`는 이미 내려받아
풀어 둔 CI 산출물만 읽는다. 다운로드·네트워크·빌드·binary 실행이나 Git 설정 변경을 하지 않는다.
반환값은 `{native: Path, nativeReceipt: Path, proof: dict}`이며 proof는 JSON으로 저장할 수 있다.

- Windows는 `rhwp_native_ffi.dll`, `native-build-receipt.json`, `windows-sdk-receipt.json`이
  각각 정확히 하나 필요하다. Native 영수증의 build 성공·source·hash와 outer 영수증의 Windows
  platform·동일 source·`native-skia-d3d`/`x86_64-pc-windows-msvc` Native cargo build 종료 0을
  확인한다. 이후 consumer 실패 때문에 outer build가 실패한 경우에도 Native 성공은 보존한다.
- macOS는 `librhwp_native_ffi.dylib`, `macos-sdk-receipt.json`, `metal-producer.json`이
  각각 정확히 하나 필요하다. SDK build·compile/link·Metal readback 성공, architecture/target,
  `native-skia-metal` build 종료 0과 producer 성공·동일 source/Native hash를 모두 확인한다.

영수증의 과거 CI 절대 경로는 열지 않는다. archive 안의 유일한 파일을 찾고 실제 SHA-256과
영수증을 대조한다. root와 하위 entry의 symlink/reparse point·hardlink·비정규 파일·중복 파일,
읽는 동안의 변경을 거절한다. archive는 8,192 entry·2 GiB, 단일 파일/Native는 512 MiB,
영수증은 32 MiB, 깊이는 32로 제한한다. 기존 출력·영수증의 source SHA를 새 SHA로 바꾸지 않는다.
Windows의 `DirEntry.stat()` 빠른 metadata는 identity/link 계수를 0으로 보고할 수 있으므로
scan과 읽기는 `Path.lstat()`의 권위 있는 결과를 사용하고 열린 descriptor의 `fstat()`와 대조한다.
권위 있는 link count도 0이면 거절하며, 실제 hardlink·symlink·reparse 경계를 완화하지 않는다.
CPython 3.12.10의 [경로 조회](https://github.com/python/cpython/blob/v3.12.10/Modules/posixmodule.c#L2008-L2018)는
ctime을 생성 시각으로 바꾸지만 [fd 조회](https://github.com/python/cpython/blob/v3.12.10/Python/fileutils.c#L1026-L1047)는
ChangeTime을 유지한다. Windows 양쪽 identity는 정확한 `birthtime_ns`로 이 의미 차이를 정규화한다.
mode·device/file ID·size·mtime·단일 link 검사는 유지하고, path의 원래 metadata 전후와 fd의 원래
ChangeTime 전후도 각각 비교한다. 생성 시각 metadata가 없으면 명시적으로 실패하며 시각을 반올림하거나
mtime/ctime 검사를 끄지 않는다. POSIX identity는 기존 ctime 검사를 그대로 유지한다.
[공식 문서](https://docs.python.org/3.12/library/os.html#os.stat_result.st_birthtime_ns)의
Windows 생성 시각 구분을 따른다. 읽기 오류에는 제한된 상대 이름과 scan/path/fd의 숫자 metadata만
기록하고 host 절대 경로나 파일 내용은 출력하지 않는다.

`expected_source`는 실제 Native 빌드의 소문자 40자리 Git SHA다. 현재 HEAD는 전체가 clean이어야
하며, 두 commit의 **전체 Git mode·object ID·path manifest**를 대조한다. 제외는 아래 세 범위뿐이다.

1. `bindings/Flutter/**`
2. `mydocs/pr/assets/issue8701/**`
3. `.github/workflows/desktop-platform-qualification.yml`

Cargo·Rust·Native·vendor·assets·Swift C header·compile-time 문서·알 수 없는 경로의 추가·삭제·
내용·mode 변경은 재빌드를 요구한다. 제외 범위와 비슷한 이름의 다른 경로도 거절한다.
tree metadata는 최대 250,000 entry·64 MiB다. sparse/promisor checkout은 허용하지만 Git 읽기에
자동 lazy fetch와 모든 transport를 차단한다. 필요한 commit/tree가 없으면 실패하며 caller가
먼저 정확한 과거 SHA와 tree를 fetch해야 한다. blob body를 내려받아 비교하지 않는다.

proof는 이전 Native source·현재 HEAD·동일한 Native 입력 manifest digest·제외된 변경 경로·
원래 빌드 command·바이너리/영수증 hash와 `nativeRebuilt:false`를 기록한다. 소스 동등성은 새 Rust
빌드나 새 Flutter consumer·GUI·GPU 완료·하드웨어 검증을 뜻하지 않는다. caller는 이 proof와
원래 영수증을 보존하고 새 SDK/plugin/실제 consumer 검증을 계속 수행한다.

```bash
python3 -B bindings/Flutter/test/native_artifact_provenance_contract.py
```

portable 검사는 잘못된 archive·영수증·소스 변경과 private promisor의 외부 fetch 실행을
거절하는 계약만 확인하며 실제 Windows/Apple SDK 또는 Native 실행의 대체가 아니다.
