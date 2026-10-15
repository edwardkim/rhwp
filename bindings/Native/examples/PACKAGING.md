# Native Skia 참조 호스트 패키징

이 패키지는 현재 OS에서 빌드한 native C ABI 공유 라이브러리와 Python 참조 호스트를
한 디렉터리에 모은다. WASM·브라우저·Node 런타임을 사용하지 않는다. Python 3.9 이상이
필요하며, 창 예제는 별도 PySide6 설치가 필요하다. 운영체제 설치 프로그램이나 서명된
배포판은 아니다.

## 빌드와 패키징

저장소 루트에서 실행한다. Cargo target은 물리 워크트리별로 다른 경로를 사용한다.
아래 경로는 이 체크아웃만 사용하는 예시이며, 다른 체크아웃과 공유하지 않는다.

```sh
export CARGO_TARGET_DIR="$PWD/target/native-host-package"
cargo build --locked --release -p rhwp-native-ffi --features native-skia

python3 scripts/package-native-host.py \
  --probe-library "$CARGO_TARGET_DIR/release/librhwp_native_ffi.so"
mkdir -p dist
python3 scripts/package-native-host.py \
  --library "$CARGO_TARGET_DIR/release/librhwp_native_ffi.so" \
  --output dist/rhwp-native-host-linux \
  --source-revision "$(git rev-parse HEAD)"
```

macOS의 라이브러리 이름은 `librhwp_native_ffi.dylib`, Windows는
`rhwp_native_ffi.dll`이다. `--target`으로 빌드했다면 Cargo target triple 하위 경로를
명시한다. **패키징은 라이브러리가 실행되는 동일 OS·아키텍처에서 수행한다.**
패키저는 라이브러리를 검색하거나 빌드하지 않으며, 잘못된 바이너리에 다른 라이브러리를
대신 사용하지 않는다. 기본 feature로 빌드한 라이브러리는 거절한다.

라이브러리는 신뢰하는 빌드 산출물이어야 한다. capability 조회도 해당 바이너리 코드를
실행한다. 별도 Python 프로세스에서 로드하고 종료를 기다려 Windows DLL 잠금을 해제하며,
일반 패키징 경로의 probe에는 30초 제한을 둔다. 이는 임의 바이너리의 안전성을 보장하는
샌드박스가 아니다. `--probe-library` 단독 모드는 직접 조회하므로 이 시간 제한이 없다.

`rhwp_session_capabilities`의 실제 응답에서 `abiVersion: 1`, `ok: true`,
`nativeSkia: true`, `hostRequests: true`, `worker: true`를 모두 요구하고,
`rhwp_native_host_request` 심볼도 확인한다. 반환 문자열은 `rhwp_string_free`로 해제한다.
이 검사는 개별 worker 작업이나 화면 렌더링이 성공했다는 증거를 대신하지 않는다.

출력의 부모 디렉터리는 먼저 만들어야 한다. 출력 경로가 이미 존재하면 빈 디렉터리,
파일, 끊어진 심볼릭 링크도 거절한다. 복사와 probe는 같은 부모의 임시 디렉터리에서
진행하고, 출력 디렉터리를 배타적으로 만든 뒤 `manifest.json`을 마지막에 옮긴다.
실패하면 이 호출이 만든 임시 파일과 새 출력만 정리한다. 동시 생성된 다른 출력은
보존한다. 성공 종료 전에는 출력물을 소비하지 않는다. 이 절차는 비정상 전원 차단의
원자적 디렉터리 게시나 공격자가 동시 변경하는 파일시스템에 대한 보장을 제공하지 않는다.

## 포함 파일과 이동

- `lib/`: 명시한 라이브러리 한 개
- `examples/`: `native_host.py`, `native_worker.py`, `native_file_store.py`,
  `qt_native_host.py`, `qt-requirements.txt`
- `docs/README.md`, `docs/PACKAGING.md`, `docs/QT_NATIVE_HOST.md`, 저장소의 `LICENSE`
- `run.py`: 패키지 상대 경로만 사용하는 런처
- `manifest.json`: 플랫폼·아키텍처, 실제 capability 응답, 라이브러리 SHA-256,
  복사 파일과 런처의 SHA-256, 호출자가 선언한 `declaredSourceRevision`

`declaredSourceRevision`은 40자리 Git commit ID를 기록한 선언이다. 외부에서 제공한
라이브러리가 실제로 그 commit에서 빌드됐다는 증명은 아니다. 파일 해시는 바이트의 동일성
확인용이며 서명·출처 인증·재현 빌드 증거가 아니다. manifest 자신은 해시 목록에서 제외한다.

디렉터리를 같은 OS·아키텍처의 다른 위치로 옮긴 뒤 어디서든 실행할 수 있다. 런처는
패키지 파일 해시와 경로 이탈, 플랫폼을 확인하며 외부 `--library` 재지정을 허용하지 않는다.
입력·출력 인수의 상대 경로는 호출한 작업 디렉터리를 기준으로 한다.

```sh
python3 /moved/rhwp-native-host/run.py headless --output /tmp/new-document.hwpx
python3 /moved/rhwp-native-host/run.py headless \
  --input /documents/source.hwp --output /documents/new-copy.hwpx

# 선택 사항: 별도 가상환경에서 Qt 의존성을 설치한 뒤 창 예제를 실행한다.
python3 -m pip install -r /moved/rhwp-native-host/examples/qt-requirements.txt
python3 /moved/rhwp-native-host/run.py window --input /documents/source.hwp
```

headless의 `--output`은 필수이며 새 HWPX 파일이어야 한다. window의 `--input`과
`--output`은 선택 사항이며, 후자는 기본 저장 경로다. Qt의 추가 의존성은 requirements로
설치하며 Python·PySide wheel·시스템 폰트는 패키지에 포함하지 않는다. OS의 동적 로더,
시스템 라이브러리와 폰트 환경은 여전히 필요하다. 예를 들어 Linux는 빌드와 실행 환경의
Fontconfig/FreeType 호환성을 별도로 확인해야 한다. 저장소 LICENSE의 복사는 포함된 모든
native dependency의 재배포 의무를 해결했다는 주장이 아니며, 실제 배포 전 third-party
고지·라이선스·서명·SDK 및 지원 OS 범위를 별도로 검토해야 한다.

## 검증과 CI 범위

```sh
# native 라이브러리·Qt·Cargo 없이 임시 파일과 mock capability로 검사한다.
python3 bindings/Native/tests/package_native_host_contract.py
```

계약 검사는 기존 출력 보존, 실패 정리, feature 거절, manifest 해시, 디렉터리 이동 및
런처 경계만 검증한다. 이 성공을 실제 native 렌더링 또는 Qt 창 실행으로 해석하지 않는다.
실제 라이브러리로 패키징한 뒤 headless/창 예제와 대상 OS를 각각 실행하고 결과를 기록한다.
worker를 쓰는 호스트는 종료 요청 후 작업 완료와 release까지 확인하고 라이브러리를
언로드해야 한다. 창·IME·접근성의 실제 OS 통합 및 한컴 시각 비교는 별도 검증 대상이다.

`Native host package` workflow는 `workflow_dispatch`로만 시작한다. Linux x86_64,
macOS arm64, Windows x86_64의 native-skia FFI 빌드 → capability probe → 패키징 →
패키지 headless 실행 → 7일 보관 artifact 업로드를 수행한다. PR/push 자동 CI를 추가하지
않으며, GitHub Release·패키지 배포·코드 서명은 수행하지 않는다. 로컬에서 workflow 파일과
mock 검사를 확인한 것과 각 OS runner의 실제 성공을 구분한다.

workflow promotion policy에는 manual `direct` 증거로 등록한다. 같은 exact candidate SHA의
`Package linux-x86_64`, `Package macos-aarch64`, `Package windows-x86_64` job 성공과
각 플랫폼의 `rhwp-native-host-<platform>` artifact를 모두 요구한다. artifact 이름은 run 안에서
고정하며, run/head 연결과 각 패키지 manifest의 선언 SHA를 별도로 확인한다. 기존 required check나
배포 권한을 바꾸지 않으며, 로컬 mock 검사를 수동 workflow 실행 증거로 대체하지 않는다.
