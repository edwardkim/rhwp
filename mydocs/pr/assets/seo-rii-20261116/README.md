# seo-rii P120–P130 검증 원장

2026-11-16 KST. 제품 소스는 원 최종 head `4379aa080b01a5da26f03f154287057ff6fb8269`와 devel `443dafb08107152e066689544e88d90988339567`의 자동 병합 `3c46e4dc550aeddc8b91c79f090239c538cd1506`이다. 계획 문서를 더한 실제 실행 checkout은 `4f6fdb689c2fe45964080074c7a230d2023b4e53`이며 제품 보정은 없다. 입력은 기존 공개 `bindings/Flutter/test/fixtures/desktop-consumer.hwpx`(8,082바이트, SHA-256 `648dade82585fa2845b80ec510b5f55c67e8b6127688dd3748b9a2068d87a3d9`)이다.

## 실제 실행

| 경로 | 실제 확인 | 근거 |
| --- | --- | --- |
| Flutter 3.47.9 / Dart 3.13.7 | strict analyze, 기본 354 PASS + Native 미연결 6 SKIP; 실제 최종 Native/dispatcher를 연결한 360 PASS·0 SKIP | `flutter-results.json`, `flutter-native-linked.json` |
| Linux C++ | portable 21종·332 checks, 공통 타일·mailbox·trim·metrics; 실제 GTK·Flutter public SDK의 strict compile/link | `portable-results.json`, `linux-sdk-pkgconfig.json` |
| Windows C++ | portable/ASan/UBSan, 실제 VS2022·Flutter SDK `/W4 /WX` 3개 translation unit | `portable-results.json`, `windows-sdk-compile.json` |
| Linux Native GL | Rust 1.93.1 release·native-skia-gl 빌드 | `native-gl-artifact.json` |
| Windows Native D3D | 실제 MSVC·Windows SDK release-test·native-skia-d3d 빌드 | `windows-native-final.json` |
| Linux 실제 두 타일 | 초기·편집·resize 3단계, 전체 4,098/4,097px 영역·타일 경계의 같은 snapshot 독립 CPU 1x/2x replay, 실제 X11 root 픽셀, 종료 자원 0 | `linux-tile-receipt.json`, `linux-resize.png` |
| Windows 실제 두 타일 | 초기·편집·실제 Win32 창 resize·복구 4단계, 실제 DPR 1·D3D descriptor 열림·CPU replay·공개 immutable frame 보존·종료 자원 0 | `windows-tile-receipt.json`, `windows-recovered.png` |
| Windows private target pool | 실제 4단계에서 완료된 비공개 target 재사용 횟수 2, 종료 후 private target·byte 0 | 위 receipt의 limits와 result |
| GL 실제 trim·메모리 압력 | 9,662,208→0 논리 바이트; 동시 요청 같은 future·압력 신호 2개를 Native 요청 1개로 합침·공개 프레임 유지·기존 3단계 RGBA 동일 | `linux-trim-receipt.json`, `linux-trim-result.json`, `linux-producer-trace.json` |
| D3D 실제 trim·메모리 압력 | 3,763,968→0 논리 바이트; 압력 중복 요청 1개·공개 프레임 유지·기존 4단계 RGBA 동일·측정 sample 4개·종료 자원 0 | `windows-lifecycle-trim-validation.json`, `windows-trim-result.json`, `windows-producer-trace.json` |
| 실제 Linux 참조앱 | 공개 API의 별도 UI thread·Skia 설정으로 build; 2,400×1,400→3,200×1,600 resize·편집·WM_DELETE_WINDOW·저장 취소·재요청·폐기 후 exit 0; timeout/fallback 없음 | `linux-reference-build.json`, `linux-reference-result.json`, `linux-reference-trace.json`, 실제 PNG 2개 |

Flutter framework `c197ca5199e9ee5661e865596c9c07294dfab51c`, engine `89c39edfdfae2949f6806e4a315c0c08522238b7`. Linux는 별도로 소유한 authenticated TCP Xvfb/Mesa 소프트웨어 GL이며 사용자 WSLg 화면·입력·설정을 변경하지 않았다. Windows는 Windows 11 실제 호스트의 공개 desktop이다. Native DLL/.so, plugin, engine, dispatcher, runner와 전체 번들 해시를 receipt로 묶고 실제 module/Win32/X11 소유 신원을 대조했다.

## 추가 lifecycle 검사와 원 collector의 차이

기존 `TileProbe`에 실제 `NativeGpuLifecycle`을 붙인 추가 검사만 ignored output에 생성했다. 제품 Dart·C++·Rust는 변경하지 않았다. 원 collector는 wrapper가 없는 4회 캡처만 기대한다. 실제 WM_SIZE의 `didChangeMetrics → pause/cover → resume/invalidate`에서 중간 view 5와 최종 view 6이 별도로 캡처되어 전체는 5회다. 첫 추가 시도들은 원 probe의 `captures == index`, `submits == index` 가정에서 중단됐다. source나 제품이 아니라 **추가로 붙인 검증 코드**의 가정 오류다.

최종 추가 runner는 각 게시의 같은 전체 stamp를 가진 snapshot 1개·두 타일의 동일 source·유효한 현재 세대·독립 CPU/실제 픽셀을 계속 검사한다. 원 collector는 4단계 픽셀·복구·실제 exit 0까지 완료한 뒤 `captures == 4` 조건 하나에서 실패했다. 이 실패 receipt는 `windows-lifecycle-original-collector-receipt.json`으로 그대로 보존했다. 별도 `validate-windows-lifecycle-trim.py.txt`(비활성 증거 사본)는 실제 5개 캡처가 `[view 1,3,5,6,7]`의 서로 다른 전체 stamp임을 확인하고 나머지 원 completion·close·zero-resource 조건을 재검사한다. 실제 captures 값이나 원 실패 결과를 덮어쓰지 않는다. `windows-lifecycle-trim-validation.json`의 PASS는 이 추가 검사만의 판정이다. wrapper 없는 원 정식 4단계 qualification의 PASS와 구분한다.

## 판정 범위

이 PR들은 문서 parser·IR·줄 구성·페이지 배치·PageLayerTree/PaintOp·저장·WASM Canvas 생산자를 바꾸지 않는다. 기존 PaintOp를 소비하는 opt-in Native GL/Metal/D3D의 타일 키·자원 장부·완료 수명·호스트 이벤트/종료·진단을 변경한다. 독립 CPU replay는 같은 불변 snapshot의 producer와 타일 합성 의미를 대조하는 기준이며 한컴 Print와 조판 일치율을 뜻하지 않는다. 실물 문서·한컴 PDF·새 조판 golden이나 좌표 pin을 추가·변경하지 않았다. WASM에 cfg 제외된 Native consumer를 검토한 결과를 fresh WASM 전체 문서 실행으로 표시하지 않는다.

소프트웨어 GL/Windows API 표시 성공으로 물리 GPU 가속·consumer GPU 완료 fence·전체 VRAM/RSS·물리 FPS·전력 개선을 주장하지 않는다. `managedBytes`는 Native target 장부, Windows private pool은 별도 장부다. 기본 비활성 GPU opt-in 및 부모 #536·#8701의 남은 qualification 범위를 유지한다. macOS는 메인테이너가 환경을 제공할 수 없어 기여자의 주장을 수용한 작업지시자 결정을 적용한다. portable 정책/코드 검토를 Apple SDK compile/link·Metal 실기로 표시하지 않는다.

## 재현 명령과 보존

- `flutter analyze --no-pub --fatal-infos --fatal-warnings`; `flutter test --no-pub --concurrency=1 --reporter=expanded`(Native .so·dispatcher define 연결).
- `python3 -B bindings/Flutter/linux/tests/check_linux_plugin.py`; `python3 -B bindings/Flutter/windows/tests/check_windows_bridge.py --sanitize`.
- `cargo build --locked -p rhwp-native-ffi --features native-skia-gl --profile release --target-dir /home/edward/mygithub/rhwp/target/pr-review`.
- 실제 Windows: `cargo.exe build --locked --target x86_64-pc-windows-msvc --profile release-test -p rhwp-native-ffi --features native-skia-d3d`; 같은 물리 공용 target을 임시 mapped drive 경로로 사용했다.
- `build_linux_probe.py --tile-group`, `build_desktop_probe.py --tile-group` → 각 `qualify_*_tile_group.py`. 전체 인수·해시는 각 receipt에 있다.
- `reference_app.py build --mode debug --trace` → 생성 bundle 검증 뒤 실제 X11/Win32 이벤트 실행.

원본 `.log`는 ignored `output/pr-review/seo-rii-20261116`에 있으며 커밋하지 않는다. SDK·Native 바이너리·임시 runner·VM service 주소·Xauthority cookie도 커밋하지 않는다. 검토 후 종료 정리는 정식 증거를 먼저 보존하고 canonical 절차로 수행한다.

## 원 code head와 광범위 CI 재사용

원 PR #9132의 head `4379aa080b01a5da26f03f154287057ff6fb8269`가 실제 원격 merge 대상이다. 새 code head·source/test/fixture/workflow 보정·fork push는 없다. 로컬 current-base merge는 검증 전용이며 원 PR head와 구분한다. [CI run 43829950196](https://github.com/edwardkim/rhwp/actions/runs/43829950196)과 `live-final-checks.json`의 동일 head 전체 CI·Native·lint·Frontend·CodeQL·Adapter·Proptest·정책 성공을 확인했다. 정상 skipped 작업을 실행 성공으로 세지 않는다.

정본 `pr_review_workflow.md` 3.2.2의 네 조건을 적용한다: (1) 원 후보 SHA와 녹색 check SHA가 같음, (2) 메인터너 source/test/fixture/workflow 보정 없음, (3) 최신 base와 원 후보의 clean merge tree가 실제 실행한 tree와 같음, (4) 별도 focused Rust 21개와 실제 GL/D3D 소비·CPU/화면·호스트 경계 확인 완료. 이 조건으로 전체 release-test nextest·Native Skia 광범위 회귀는 원 code head의 GitHub 결과를 재사용하며 로컬 재실행 PASS로 기록하지 않는다. `scope-and-remote-candidate.json`은 원격 후보와 실제 로컬 tree를 연결한다.

추가로 로컬 current-base source에서 최소 글꼴 격리의 focused 8+8+5개, fmt, Native root Clippy, WASM32 Clippy를 실행해 통과했다. 공개 fallback 5개만 보이는 filesystem 격리와 visible 목록을 기록했다. suite manifest·unit tier 정책도 통과했다. 중복 broad workspace build는 원 후보 그대로의 직접 통합 경로를 확정하면서 중지했다(`optional-broad-stop.json`, `rust-gates.json`의 exit -15). 중지한 빌드를 PASS로 표시하지 않으며, 그 뒤 workspace all-target lint·로컬 전체 nextest를 실행한 것으로 주장하지 않는다. 새 제품 보정이나 code head가 생기면 이 재사용 판단을 취소하고 새 head의 전체 필수 검증으로 돌아간다.

## Python 도구 계약과 호출 방식

현재 SDK/compiler 소비 도구의 모델 계약 10개 묶음, Metal cache 정책, Apple SDK launcher 모델, 실제 Native owner shim과 명시한 mock library의 큐/TLS/수명, Dart FFI 이동 번들 검사(총 unittest 230개와 별도 assertion)를 완료했다. `python-final-validation.json`·`python-contracts-proper-launch.json`·`python-explicit-arguments.json`에 명령과 결과를 연결했다. 인수를 요구하는 `native_owner_contract.py`를 unittest discovery로 잘못 불렀던 초기 NameError는 올바른 `--dispatcher`·`--mock` 호출로 해소했다. 실제 Mac artifact를 요구하는 `native_metal_sdk_contract.py`는 자동 모델 검사와 다르며 인수 없는 초기 호출은 usage 오류였다. 이를 Mac 실기 성공으로 세지 않는다. Mac 실기는 작업지시자의 기존 수용 결정에 따라 미실행 상태를 명시했다.

최종 SDK launcher 모델·owner·FFI 검사에는 각각 명시한 SDK/fixture만 공급했다. 소스·검사 명령이나 허용치를 통과를 위해 완화하지 않았다. 여기에 보존한 `.py.txt`·`.dart.txt`는 ignored output에서 사용한 추가 관측 코드의 **비활성 증거 사본**이며 제품 소스·활성 suite를 등록하지 않는다.

Windows가 작성한 증거 사본은 Git 공백 규칙에 맞춰 CRLF를 LF로만 정규화했다. 데이터·성공/실패 판정은 바꾸지 않았다. 원본 바이트와 저장 사본의 SHA-256 및 변환은 `evidence-line-endings.json`에 기록했다.

## 원격 통합

작업지시자가 원 PR 머지와 후속 처리를 승인한 뒤 원 11건의 exact-head review를 각각 게시했다. #9132를 `97575816999577744f0e2e7f2e16a41d5961bebf`로 merge했고, 원 11건 모두 MERGED·원 SHA devel 포함·실제 merge tree 일치를 확인했다. contributor fork branch는 보존했다. 정식 증거는 후속 운영 기록 commit에 포함되므로 댓글의 raw 이미지 URL은 그 실제 commit을 고정한다. Mac 실기 미실행과 물리 GPU/전체 메모리/한컴 fidelity 범위는 위 판정대로 유지한다.

병합 후 자동 [duration 메타데이터 갱신](https://github.com/edwardkim/rhwp/actions/runs/43866524638)은 **success**다. 실제 step의 collect·prepare·publish 성공을 확인했다. [영수증](duration-refresh-result.json)을 보존했다. 후속 제품 CI·CodeQL·Render Diff·Adapter·Proptest·Oracle 재실행은 하지 않았다.
