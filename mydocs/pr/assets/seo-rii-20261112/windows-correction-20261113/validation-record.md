# 최종 검증 기록 — 2026-11-13

## 결정과 소스

- 사용자는 이번 macOS 기여자 주장을 수용했다. Apple SDK·Metal·ScreenCaptureKit 실기는 미실행이다.
- Windows CMake 보정·실제 runner·타일 편집/resize/복구·이동 번들 재검증을 승인했다. 원격 source push·PR·댓글·merge 승인은 아직 없다.
- 초기 base `a85d3265dd10b316d1a0ae029ca3f67245b38b77`, 초기 소스 `1e819721fe789b6b3b5c6db600635245039c9f98`의 7쪽 결과는 양쪽 100.00%였다. 이후 최신 base `41c889cfd8a869ad23199d6018f8b569879a575c`와 최종 제품 `4ab994ee67`로 검증했다.
- 본인 계정 열린 PR 0건. 현재 base와 제품 head의 merge-tree exit 0, tree `6d31baf062394787fa37b8eb5b8c8dc3fdc48a22`.

| 원 PR | 보존 head | 최종 원 head CI |
| --- | --- | --- |
| #8952 | `3077c620f3ec7c09cd93c2b79e7ea0825be12488` | 33 success·6 정상 skip, OPEN, GitHub mergeable UNKNOWN 재계산 중 |
| #8953 | `9cfe2754d92a720135445aed285ff91c0c75dc7e` | 33 success·6 정상 skip, OPEN, GitHub mergeable UNKNOWN 재계산 중 |
| #8954 | `1e14e4b8466b7ce46d9016faf1824bc05c9789de` | 33 success·6 정상 skip, OPEN, GitHub mergeable UNKNOWN 재계산 중 |

Draft #8962–#8967은 제외하며 부모 #536·#8701은 OPEN 유지. 새 통합 후보의 GitHub CI는 미실행이다.

## 공통 producer와 실제 바이너리

`bindings/Flutter/`와 이 검토의 `mydocs/pr/`를 제외한 Git mode/blob/path 38883행은 base와 같다. SHA-256 `e718a2e215aa0c2baa130243012b808634eb8523c4efdf60356c059d82ebe394`. Source60→최종 소스의 Rust·Native·WASM·공통 C header 입력도 같고, Windows 소비자 C++만 최종 소스로 다시 빌드했다. 별도 base 빌드는 주장하지 않는다.

| 바이너리 | 실제 source | SHA-256 |
| --- | --- | --- |
| Linux librhwp_native_ffi.so | `60f5659d5b` | `4d687150b6cfee89fb2333e1202b886b30a01b3267031f47f12897f57d9d545b` |
| Linux rhwp | `60f5659d5b` | `dac82fc87ddfb5f75fd2062c1d893333eca1968691999587cdefd7a2125b39e2` |
| Windows rhwp_native_ffi.dll | `60f5659d5b` | `644370026025681c8d84e74af5a4aa2d4cb73fe872286f225a1041fe7e354344` |
| fresh WASM rhwp_bg.wasm | `60f5659d5b` | `261812fcf6ac5751071cbcfcd1a4d6f879b9737c062e76e27db27c526d6ad2d9` |
| Windows rhwp_reference_app.exe | `4ab994ee67` | `3cf012c06b9261d5cf720ada74fec158c7540e0f17886664eb9907311e14d3ad` |
| Windows rhwp_native_reference_plugin.dll | `4ab994ee67` | `2467e872818ae99f6fad20fc329742b75b173c683fd4cd874c14f82e8d239ad2` |
| Windows flutter_windows.dll | 고정 engine `89c39edfdfae2949f6806e4a315c0c08522238b7` | `30494fb08f9f101a8fbe42568110aaa160efbd7341f3cfe877dc14214be3a233` |
| Windows rhwp_flutter_owner.dll | `4ab994ee67` | `c40b8f69ee8733fc48c4e1601b46ef4f0eeaee37b56dbd84c25b1db0b18988da` |
| Windows rhwp_native_ffi.dll | `60f5659d5b` | `644370026025681c8d84e74af5a4aa2d4cb73fe872286f225a1041fe7e354344` |

Linux Native 명령: `cargo build --locked --profile release-test -p rhwp -p rhwp-native-ffi --features native-skia-gl`, exit 0, 325.7초. Windows Native는 같은 profile에서 `-p rhwp-native-ffi --features native-skia-d3d`, exit 0, 499.977초.

Fresh WASM 명령: `RHWP_WASM_CDYLIB_ONLY=1 CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review CARGO_BUILD_JOBS=8 scripts/wasm-pack-locked.sh --target web --out-dir /home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/current-wasm-pkg`, exit 0, 1189.668초, release lto=false·wasm-opt 포함.

## 실제 실행과 경계

| 항목 | 실행 결과 |
| --- | --- |
| CMake direct | exit 0 |
| CMake junction | exit 0 |
| CMake relative-directory-symlink | exit 0 |
| CMake unc-directory-symlink | exit 0 |
| metadata 수정 전 두 새 양성 검사 | 둘 다 exit 1로 기존 결함 재현 |
| Linux runtime bundle·Native artifact·reference app 계약 | 13+27+10개, 전부 PASS |
| 실제 Windows runtime bundle·Native artifact 계약 | 13개 수집: 11 PASS·POSIX 2 skip, 별도 27개 PASS |
| Windows 실제 unchanged `.exe` 읽기 | 수정 전 거부, 수정 후 정규 hash·PE identity 수용 |
| Windows 참조앱 종료 수정 전 | `0xc000041d`, Event 1000의 flutter_windows.dll access violation `0xc0000005`, RVA `0x1551f`; MessengerSetCallback export `0x154f9` |
| 최종 Windows two tile | `windows-two-tile-consumer-passed`, initial→revision→resize→recovered, exit 0 |
| 최종 Native teardown | frameBytes·frameHandles·inFlightReplays·leases·retainedFrames·retainedSnapshots·snapshotEstimatedBytes·snapshotHandles·sources 모두 0 |
| 최종 Windows 이동 참조앱 | 1쪽·6쪽 열기, 1→6쪽 이동, 문서 닫기·WM_CLOSE exit 0 |
| 최종 Linux 이동 참조앱 | 1쪽·6쪽 열기, 1→6쪽 이동, 문서 닫기·WM_DELETE_WINDOW exit 0 |
| 실제 Native 연결 Flutter | 280 PASS·0 skip, 49.364초 |
| portable Metal policy | 1,669개·일반/ASan/UBSan 모두 PASS, Mac 실기와 구분 |
| 최종 Windows portable | frame lifetime 51·import lifetime 4,224·close registry 1,683·bounded tile 25·group error 35 및 disabled ABI 5개 PASS |
| 별도 runtime C/Dart FFI | 3개 ABI 시나리오 PASS, 실제 Native 렌더링과 구분 |

실제 Windows CPU/시각 collector의 source/nonce/stamp·독립 replay·seam·foreground 조건을 완화하지 않았다. probe PID 소유 창만 foreground로 활성화했다. 창은 1264×681→1296×713, Win32/FlutterView DPR 1.0이다. 다른 OS DPR 쌍은 미실행이다. `actualDprComparison`, `consumerGpuCompletionVerified`, `hardwareAccelerationVerified`, `targetReuse`, `consumerAndDriverMemoryIncluded`는 false다.

참조앱 source는 최종 `4ab994ee67`, Native supplied source는 `60f5659d5b`다. Windows/Linux 모두 builder의 생성 source·runner를 제거하고 이동한 공백 경로에서 RHWP 환경 변수 없이 별도 cwd로 실행했다. Windows의 위 5개 module은 이동한 bundle 안에 로딩됐다. 초기 Linux mapping inode는 초기 base의 별도 이력이며 최신 실행으로 바꾸어 기록하지 않는다.

## 실행 명령과 종료 코드

아래는 실제 명령과 결과의 최종 Markdown 기록이다. 원시 로그·run manifest·중간 JSON은 ignored output 또는 소유 Windows 임시 검증 폴더에만 두고 PR에 포함하지 않는다.

| 명령 | exit | 소스/범위 |
| --- | --- | --- |
| `/home/edward/.local/share/rhwp-tools/flutter-c197ca5199e9/bin/flutter pub get --offline` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/macos/tests/tile_group_qualification_test.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/macos/tests/native_view_qualification_test.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/macos/tests/sdk_qualification_contract.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/test/reference_app_contract.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/test/runtime_bundle_contract.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/test/windows_tile_group_qualification_contract.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/test/windows_continuous_qualification_contract.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/test/desktop_qualification_contract.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/test/native_gpu_tile_qualification_contract.py` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `c++ -std=c++17 -O2 -pthread -Wall -Wextra -Werror bindings/Flutter/macos/tests/metal_tile_group_test.cc -o /home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/portable-current/metal-tile` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `/home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/portable-current/metal-tile` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `c++ -std=c++17 -O1 -g -pthread -Wall -Wextra -Werror -fsanitize=address,undefined -fno-omit-frame-pointer bindings/Flutter/macos/tests/metal_tile_group_test.cc -o /home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/portable-current/metal-tile-sanitized` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `/home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/portable-current/metal-tile-sanitized` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/test/runtime_bundle_ffi_contract.py --dart /home/edward/.local/share/rhwp-tools/flutter-c197ca5199e9/bin/dart` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `/home/edward/.local/share/rhwp-tools/flutter-c197ca5199e9/bin/flutter analyze --no-pub --fatal-infos --fatal-warnings` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `/home/edward/.local/share/rhwp-tools/flutter-c197ca5199e9/bin/flutter test --no-pub --reporter expanded` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `/home/edward/.local/share/rhwp-tools/flutter-c197ca5199e9/bin/flutter pub get --offline` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `/home/edward/.local/share/rhwp-tools/flutter-c197ca5199e9/bin/flutter analyze --no-pub --fatal-infos --fatal-warnings` | 0 | 60f5659d5bbd814c6653d961fb85be2703a285d9 |
| `python3 -B bindings/Flutter/test/runtime_bundle_contract.py` | 0 | 4ab994ee67ff66e35aa9168ac6fae506b6b26122 |
| `python3 -B bindings/Flutter/test/native_artifact_provenance_contract.py` | 0 | 4ab994ee67ff66e35aa9168ac6fae506b6b26122 |
| `python3 -B bindings/Flutter/test/reference_app_contract.py` | 0 | 4ab994ee67ff66e35aa9168ac6fae506b6b26122 |
| `python3 -B bindings/Flutter/windows/tests/check_windows_bridge.py --output /home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/windows-portable-final` | 0 | 4ab994ee67ff66e35aa9168ac6fae506b6b26122 |
| `python3 -B /home/edward/mygithub/rhwp-worktrees/review-integration-8701-seo-rii-reference-qualification/bindings/Flutter/tool/build_dispatcher.py --output /home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/current-native/librhwp_flutter_owner.so` | 0 | source60; 변경 없는 Linux 소비자 |
| `/home/edward/.local/share/rhwp-tools/flutter-c197ca5199e9/bin/flutter test --no-pub --reporter expanded --dart-define=RHWP_NATIVE_LIBRARY=/home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/current-native/librhwp_native_ffi.so --dart-define=RHWP_OWNER_LIBRARY=/home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/current-native/librhwp_flutter_owner.so` | 0 | source60; 변경 없는 Linux 소비자 |
| `python3 -B /home/edward/mygithub/rhwp-worktrees/review-integration-8701-seo-rii-reference-qualification/bindings/Flutter/tool/reference_app.py build --flutter-sdk /home/edward/.local/share/rhwp-tools/flutter-c197ca5199e9 --native /home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/current-native/librhwp_native_ffi.so --native-source 60f5659d5bbd814c6653d961fb85be2703a285d9 --output /home/edward/mygithub/rhwp/output/pr-review/seo-rii-20261112/linux-reference-current-build --trace` | 0 | source60; 변경 없는 Linux 소비자 |

Windows 최종 빌드 명령은 `python -B bindings/Flutter/tool/reference_app.py build --flutter-sdk <고정 SDK> --native <위 DLL> --native-source 60f5659d5bbd814c6653d961fb85be2703a285d9 --output <owned-reference-output> --continuous --trace` 및 `python -B bindings/Flutter/tool/build_desktop_probe.py --flutter-sdk <고정 SDK> --native <위 DLL> --native-receipt <같은 Native 빌드 기록> --output <owned-tile-output> --tile-group`다. 둘 다 최종 source4ab에서 exit 0. collector는 `qualify_windows_tile_group.py --runner <실제 exe> --native <같은 DLL> --dispatcher <실제 owner> --fixture <공개 desktop-consumer.hwpx> --build-receipt <고정 빌드 기록> --output <owned-capture> --expected-device-scale 1 --timeout 280`, exit 0이다. placeholder는 재실행 때 실제 owned 경로로 대입하며 글꼴·시스템 설정·판정 문턱을 바꾸지 않는다.

## 입력·독립 Print와 전쪽 TSV

96 DPI·print profile·색상 경계 보정 전 2px raw, 동일 producer를 최신 base와 후보의 공통 출력으로 사용했다. 웹/Canvas 경로는 실제 최적화 WASM이다. 최저값과 전쪽 행은 아래 원본을 따른다.

| 파일 | 바이트 | SHA-256 |
| --- | --- | --- |
| [native-radial.tsv](native-radial.tsv) | 232 | `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f` |
| [native-bitmap.tsv](native-bitmap.tsv) | 132 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [native-zero-size.tsv](native-zero-size.tsv) | 130 | `40d8e3b0ffc8817111da299d316adfcd1dbef82f0b1a73c92ae1b91babb346ac` |
| [wasm-radial.tsv](wasm-radial.tsv) | 232 | `cdcf3e026b8de27abee1649a14fc0767bd3b6d02806046e34dcc9164c24df91f` |
| [wasm-bitmap.tsv](wasm-bitmap.tsv) | 132 | `75764217b6231001246ab7d8b934bdc683297f16bd9ca4c983fffc90aa768466` |
| [wasm-zero-size.tsv](wasm-zero-size.tsv) | 130 | `4d7ff75304a812abfe0d6a9f013bb9e529207b739cdd424e497bd70e1404b774` |

- 입력 `tests/fixtures/gradient/radial-corner-print-contract.hwpx` SHA-256 `01884d8ea148e21fdb68194dd23c353c635903e4fc19d8b242e1efbe1d0aaf0d`; 독립 PDF `tests/fixtures/gradient/radial-corner-print-contract-2020.pdf` SHA-256 `1f0a7707198e81b8bd9a93efe9e5e6a35fe3ffd9e0131ebb4851c71e9ec2d1a8`.

- 입력 `samples/issue8160/bitmap-channel-transfer-independent.hwpx` SHA-256 `cdb26249443ca1a2e0200245ece0df75255acc7b20278fbdaa8378e398db9784`; 독립 PDF `pdf/issue8160/bitmap-channel-transfer-independent-2024.pdf` SHA-256 `cb99add7db63dc4f783a7484a611b43175fbd08ab68fa5d3b22ddd223dcf4880`.

- 입력 `samples/issue8664/zero_size_para_before_table.hwp` SHA-256 `29dc4af9b4b746fbd089cff02029fc97bb8a8419d4818b2ddb7785f0fdbac9ce`; 독립 PDF `pdf/issue8664/zero_size_para_before_table-hwp-2024.pdf` SHA-256 `b46b42542293a39defb63e80a15ca81e33b6f38f5b59cf34e884e53cd949ab95`.

Zero-size는 한컴 2024 13.0.0.3901 사용자 수동 Print를 사용했다. 원 Cairo/webhwp 기준은 제외했다. 저장 호환 버전과 인쇄 제품은 다르다. 실제 경로·Print 출처는 README에 연결했다. 모든 내용 영역을 직접 확대 판독했으며 소수점 raw와 실제 문자/외곽선·음영·간격 판정을 구분했다.

최신 base #9025의 TOP→표→AFTER 및 #9033의 중첩 셀 검색/추출을 실제 실행했다. 검색은 `rhwp search tests/fixtures/issue8989/nested-values.hml 42,360명 --json`, 추출은 `rhwp extract-data tests/fixtures/issue8989/nested-values.hml --json`으로 각각 exit 0. 두 단계 cellPath와 42,360·4,676·1,234,000을 한 번씩 보존했고 원소 4개는 미절단이다.

## 최종 대표 이미지 계획

수용 PR 본문과 병합 후 contributor 댓글에는 아래 7개 최종 패널만 사용한다. 페이지별 raster·중복 캡처·원시 build/test 로그·중간 JSON은 제외한다.

- Windows 편집·resize 타일: `windows-tile-revision-resize.png`.
- Windows 이동 번들 두 문서: `windows-relocated-documents.png`.
- Linux 이동 번들 두 문서: `linux-relocated-documents.png`.
- 독립 Print·Native·fresh WASM 대조: `radial-first.png`, `radial-last.png`, `bitmap-first.png`, `zero-size-first.png`.

## 원 기여·Draft ancestry 확인

대상 세 원 head와 제외한 Draft 6개 tip을 실제 Git 객체로 확인했다. fetch 오류나 미보유 객체를 제외 판정으로 대신하지 않았다.

| PR | 판정 | SHA |
| --- | --- | --- |
| #8952 | preserved | `3077c620f3ec7c09cd93c2b79e7ea0825be12488` |
| #8953 | preserved | `9cfe2754d92a720135445aed285ff91c0c75dc7e` |
| #8954 | preserved | `1e14e4b8466b7ce46d9016faf1824bc05c9789de` |
| #8967 | excluded | `76509196390a1550381c3b7d73ca3ee685801e85` |
| #8966 | excluded | `b4482c3b3424c15ce26a376c153b0569f0e0374d` |
| #8965 | excluded | `120a153c85058a8982af6a0dfde458f4b64486f7` |
| #8964 | excluded | `ab4ea2f732b16d3c4274a6b96a59b9e82eafeffd` |
| #8963 | excluded | `87f439771f078a3b3935ce69a8d01a006850f2a4` |
| #8962 | excluded | `35549c66eae46d178b34ddabd629e84a05548c86` |
