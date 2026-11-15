# PR #9075 최종 검증 기록

기록: 2026-11-15T14:03:30.073291+09:00.

제품 source: `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`. 전체 회귀의 시작 head는 `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`이며 그 사이 제품 파일과 최종 검증 종료 시점의 파일 SHA가 같다. base는 `f8234f6f90cb25d9601c9d5d762e0a565aea19e4`다.

원 PR head: `1309dbea65b7a95e25fa7d6abd3e10b5e65e5eb4`. 기존 원본·독립 PDF를 바꾸거나 재인쇄하지 않았다.

Native binary SHA-256: `26005bcb8af59ff85b35c5b05f376cab93833ef5e24d640e036828cae774c53f`.
fresh WASM SHA-256: `7ccf4e7c338d3d083acfa453cf4128d026869e60739e5b703ac912f14ce8c637`.

## 실제 완료 결과

- 원 문서는 Native/fresh WASM 각각 전 14쪽 raw 100.00%이며 실제 PNG 전 14쪽이 이전 판독본과 각각 바이트가 같다. [같은 입력 4·5·6쪽의 직접 판독과 review/overlay](final/main/README.md)를 연결한다.
- 특별 보호 36개 데이터셋·3,542행은 상향된 정본 대비 하락 0행이다. 상승 행 수는 수용 조건 계약의 실제 결과를 따른다. 이전 수용 상승 30행의 하한을 그대로 지킨다. [전쪽 TSV](final/protected/README.md)·[수용 조건](final/protected-validation.json).
- 실제 필수 편람 HWPX Native gate 383쪽은 종료 코드0, 최저 99.96%다. 130쪽의 기존 WMF 차이를 포함하며 마스킹하지 않는다.
- 새 base의 다섯 원본은 같은 Linux·글꼴 조건에서 전 389쪽씩 대조했다. [양쪽 TSV](final/hft-style-coverage-tsv/README.md)와 [대표 직접 판독](final/rebase-f823-reading/README.md)을 구분한다.
- 최소 공개 글꼴 격리에서 전체 Rust·원 주장 4건·Native 3종·필수 lint와 workspace 검사를 실행했다. 실제 터미널 요약과 로그 해시는 아래와 같다.
- fresh Canvas browser 23 PASS, Studio Node 2,796 PASS·binding/embed 3 PASS·TypeScript 및 production build exit 0.
- Cargo 빌드 잡 8개, 테스트 스레드 12개, 공유 target/pr-review를 재사용하고 Cargo 실행을 직렬화했다. 회귀는 시스템·사용자 글꼴을 격리한 공개 5개 자원, 시각 측정은 원래 실제 공급 프로그램 환경이다.
- 원시 로그·run manifest·중간 JSON·중간 raster는 output에서만 관리한다. 비공개 글꼴 프로그램은 제출하지 않는다.

## 단계별 실제 종료 코드·로그 SHA

| 실행 | exit | 초 | 원시 로그 SHA-256 |
| --- | ---: | ---: | --- |
| `suite-prepare` | 0 | 1.999 | `6f4e880764b006aef181122ae2fff7b1dc1624b4edf7f29e326bf61833d6077c` |
| `format-check` | 0 | 36.958 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `native-cli-build` | 0 | 420.222 | `b3df2ca5311471fcb295e87adcb2be50ffdadc0f125fbdb9ca69bb6dd91cbba0` |
| `original-native-focused` | 0 | 443.138 | `bcc8f0b385fbc3680b349df06a62bfda803fc40c30b6d9920c03ce4317687adf` |
| `fresh-wasm-build` | 0 | 548.658 | `929e4fe9382f6c8fcc0dea547dcd28a3ba8b62d21688de6e2997774ecc9e2abe` |
| `canvas-browser` | 0 | 85.697 | `e4c3a64711cca3a12304d73b26d57d43dfdee5408aa2e5074dcc08057b12f530` |
| `hft-style-coverage-final-suite-prepare` | 0 | 3.439 | `6f4e880764b006aef181122ae2fff7b1dc1624b4edf7f29e326bf61833d6077c` |
| `hft-style-coverage-final-format` | 0 | 47.258 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `hft-style-coverage-final-suite-manifest` | 0 | 2.681 | `9d1ebbfd0387295d16a4bcb4ef7c2ac9a0a3d11897e2d1a87f980c8f5da7da69` |
| `hft-style-coverage-final-unit-tiers` | 0 | 29.773 | `b0d22c7e9e78b2de09250df5931d136093b5d2ad3c7474ab5c37aa9c3dc75ad0` |
| `hft-style-coverage-final-full-minimal-rust` | 0 | 3487.316 | `7c76d6c85783ca79dbdf51b8dbeb5222b23b8cf854cdec71910649ee2137d0ba` |
| `hft-style-coverage-final-original-claim-native-focused` | 0 | 583.651 | `f6d1f20d6ffe46f90063fc89bf5f5c24789a7cd4e170102ff93e600ac5eb9f3e` |
| `hft-style-coverage-final-final-native-lib` | 0 | 625.773 | `016b458ce781b74b16acd68f36fa041c1623fd14b94e8cc5f02dc52c6f2c88b7` |
| `hft-style-coverage-final-final-native-picture-placeholder` | 0 | 71.788 | `6f13956d5c3a38f4e6e21507e7e9fa0809ed4a231f1ea3036828553944431851` |
| `hft-style-coverage-final-final-native-direct-pdf` | 0 | 25.770 | `2f23746086a3a696a3276ac67a1f436ac02b24bee414354f1472071a53ac0059` |
| `hft-style-coverage-final-original-claim-native-clippy` | 0 | 132.892 | `e5bcca3b29354d22cbc2b8f13ba4cb1e326648696db488e1332a2e0c0b09a797` |
| `hft-style-coverage-final-default-clippy` | 0 | 125.547 | `bd49b259977ec1c213066b98cec9c803cff2f7e793b9f2a4db3648f89f6813ef` |
| `hft-style-coverage-final-wasm-clippy` | 0 | 128.694 | `935754f5998c4ceb7c123e5eb6a05baaf90d791938cef4db70254f16541ed411` |
| `hft-style-coverage-final-workspace-build` | 0 | 240.683 | `c0946556fe9a474c9a4cc0f5d860baba32d820366b2d506140f47c7f3d8d71c3` |
| `hft-style-coverage-final-workspace-clippy` | 0 | 152.045 | `bb84194e42045a4cb962fe24b7dad84f355454ab9d689cc10ad40b566bed49ed` |
| `bindings-and-embed` | 0 | 0.285 | `165308b3c7e5d8eb83670b72faf469168a1e80cdd3c1d76024a3aaad48550ca4` |
| `typecheck` | 0 | 7.120 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `node` | 0 | 47.405 | `17ace5b99bca1e46dd4b25d9f83f69f78668eede3dde74380b82bc007399b253` |
| `production-build` | 0 | 29.725 | `5d2ec94e5ff8f7400dc72bb188d8f84d6ca8ac140db9735411a54f78fe430c12` |
| `suite-helper-contract` | 0 | 14.982 | `dd0dc6c36b49a0b529050695d671402fbbbaf0d499f97913f308907d2350d62a` |
| `font-registration-contract` | 0 | 4.280 | `9a56d58a9c776536abfd087dfe47fcc804dfe98625bda7a94b8014413ae04a27` |
| `handbook-contract` | 0 | 3.370 | `ac8d115f5895b5fcbf789db518fec795cc3736d31a9d608a7f09bb198b8db8e4` |
| `new-sample-security` | 0 | 472.967 | `e0ffb1527a21dab4543766db16555f49c253e326a4ada65b81d932da3aba5e74` |
| `changed-native-cases-clippy` | 0 | 12.054 | `ed00b7e40f0b21f6aa83f81a034701b1b1275862530cffe4e0af2283d12ad156` |

## 터미널의 최종 요약

### original-native-focused

```text
Summary [   0.079s] 4 tests run: 4 passed, 339 skipped
```

### canvas-browser

```text
ℹ tests 23
ℹ pass 23
ℹ fail 0
ℹ skipped 0
```

### hft-style-coverage-final-full-minimal-rust

```text
Summary [1294.861s] 13583 tests run: 13583 passed (11 slow), 45 skipped
```

### hft-style-coverage-final-original-claim-native-focused

```text
Summary [   0.128s] 4 tests run: 4 passed, 339 skipped
```

### hft-style-coverage-final-final-native-lib

```text
test result: ok. 3853 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 26.91s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### hft-style-coverage-final-final-native-picture-placeholder

```text
Summary [   3.506s] 2 tests run: 2 passed, 324 skipped
```

### hft-style-coverage-final-final-native-direct-pdf

```text
Summary [   2.881s] 36 tests run: 36 passed, 292 skipped
```

### bindings-and-embed

```text
ℹ tests 3
ℹ pass 3
ℹ fail 0
ℹ skipped 0
```

### node

```text
ℹ tests 2796
ℹ pass 2796
ℹ fail 0
ℹ skipped 0
```

### suite-helper-contract

```text
ℹ tests 24
ℹ pass 24
ℹ fail 0
ℹ skipped 0
```

### font-registration-contract

```text
ℹ tests 2
ℹ pass 2
ℹ fail 0
ℹ skipped 0
```

### new-sample-security

```text
Summary [  20.824s] 20 tests run: 20 passed, 628 skipped
```

## 저장 경로와 증거의 한계

원문 셀의 전체 내용과 두 조각, 후속 제목의 누락·중복을 실제 render tree와 독립 PDF로 대조했다. HWPX serializer는 IR의 저장 line_segs를 소비하며 렌더링에서 그 값을 덮어쓰지 않는다. glyph-fit·Print glyph cache와 소수 paint 목적 영역은 저장 IR을 갱신하지 않는다. 이 변경에 반복 저장 검증을 생략한 근거는 해당 소비 경계이며 모든 조판 변경에 대한 일반 면제가 아니다.

권고안4쪽의 base/후보 WASM paint JSON 전체와 CanvasKit 소비 코드가 같다. 기존 실제 CanvasKit 반경 전달 증거를 재사용하며 새 CanvasKit 캡처·새 CK 점수를 주장하지 않는다.

로컬 검증 완료는 원격 CI·PR 생성·병합 완료를 뜻하지 않는다. 실제 원격 결과는 후속 PR review에 기록한다.


## 리베이스 이전 검증 기록

다음은 이전 base53/source43에서 완료한 역사적 실행이며 현재 head의 통과를 대신하지 않는다.

### 이전 실행 기록

> 새 base `f8234f6f90` 반영 뒤 재검증 중이다. 아래 수치는 리베이스 전 source43의 완료 기록이며 새 head의 통과로 대신하지 않는다. [리베이스 보고서](rebase-f823-analysis.md)를 따른다.

기록 시각: 2026-11-15T07:28:46.182146+09:00.

원시 build·test·lint 로그, run manifest, metric·분석 JSON과 중간 PNG는 제출에서 제외한다.
실행 중에는 ignored `output/pr-review/pr9075/`에서 확인하며 종료 정리를 따른다.
정식 보호 baseline의 입력·조건·수용 manifest와 명시적 글꼴 환경은 별도로 관리한다.

## 최종 제품과 원 문서

- 제품 소스: `43adf3f7fe412e71797456887e2be44b8ef8d14f`.
- base: `53da4d2da28ae0cacf8feac3a989cc6a69fb2622`; 원 PR head: `1309dbea65b7a95e25fa7d6abd3e10b5e65e5eb4`.
- 공개 86036 입력 SHA-256: `f224d490226b3673554b484d6ce1e2b1c3bbaf0aac02340706edf5fc9d0a4025`.
- 기존 독립 Print PDF SHA-256: `c9ba8e9d4b4e2cb32277d030161dedb59d9067eba0a4665b40f9c0b46cc47317`.
- PDF Creator: Hwp 2022 12.0.0.4547; Producer: Hancom PDF 1.3.0.547. 파일명·원 작성자의 engine 2020 설명과 구분한다.
- 원본·PDF를 수정하거나 재인쇄하지 않았다. compat 2022·Print·96 DPI·색상 경계 보정 전 2px raw로 대조했다.
- [최종 Native TSV](final/main/native_silhouette.tsv)·[fresh WASM TSV](final/main/wasm_silhouette.tsv): 각각 전 14쪽 100.00%.
- 두 경로의 5·6쪽 review/overlay를 직접 판독했다. [대표 증적](final/main/README.md)의 같은 입력·쪽을 사용한다.
- 실제 출력 트리와 독립 PDF에서 첫 소유 줄은 5쪽, 나머지 두 줄은 6쪽, 후속 문단 43 「Ⅱ. 규제의 적정성」은 7쪽임을 대조했다.
- 공식 100.00%는 RGB 픽셀 동일성을 뜻하지 않는다. 비공개 글꼴 프로그램은 커밋하지 않는다.

### 생산 파일 SHA-256

- Native: `5e9c68af4bb8b902b56b5a80678156da193bbcd983080a7b804e0d16ca1dccd6`.
- WASM `rhwp.js`: `75eaf7070854c66d31b2223558a4128d05d00dff46b51051ec943cfe250c6365`.
- WASM `rhwp_bg.wasm`: `b2fd15c785c7201c63b5c1942f47ae2b244db9c303614338638daa439d1b9f3c`.

## 필수 회귀와 lint

- 기본 전체 Rust: 최소 공개 글꼴 격리에서 13,562 PASS, 실패 0, 명시적 skip 45.
- Native 원 주장 집중: 3 PASS. Native lib: 본체 3,853와 하위 182 PASS, ignored 12.
- Native 그림: 2 PASS. Native 직접 PDF: 36 PASS.
- 실제 Canvas browser: 23/23 PASS. 새 두 반례는 보정 전 FAIL을 확인했고 기대값 완화 없이 제품의 CSS family 파서를 수정했다.
- Studio 전체 Node: 2,794/2,794 PASS. Studio 구현이 같은 단계의 증적을 재사용한다.
- 같은 fresh package를 pkg/Studio 실행 경로에 반영하고 해시 일치를 확인했다. 바인딩·embed 3건 PASS, TypeScript exit 0. 기존 pkg의 오래된 선언 문제를 실제 산출물 반영으로 해결했다.
- 회귀 글꼴 노출은 저장소 공개 5개 자원만 허용했다. 시스템·사용자 글꼴을 격리하고 `RHWP_FONT_PATH`를 해제했다.
- 빌드 잡 8개·테스트 스레드 12개, 공유 `target/pr-review`를 사용하며 Cargo는 직렬 실행했다.
- 시각 측정은 최소 글꼴 격리와 다르게 기존 일반 시스템 글꼴·명시적 공급 환경을 사용했다.
- Native 집중 Clippy·기본 Clippy·WASM Clippy·workspace build·workspace all-target Clippy까지 모두 종료 코드 0.
- 최종 Native 전쪽 보호 검증과 실제 편람 HWPX `--handbook-gate`까지 종료 코드 0으로 통과했다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast
```

위 명령은 최소 글꼴 bwrap 안에서 실행했다. 실제 단계별 종료 코드·수행 시간·원시 로그 해시는 아래와 같다.

| 완료 검사 | exit | 초 | 원시 로그 SHA-256 |
| --- | ---: | ---: | --- |
| `print-primary-final-suite-prepare` | 0 | 3.103 | `df56e6fbfe92ac822748362989f870708b99101c9b63fb328616b90fefa10e37` |
| `print-primary-final-format` | 0 | 42.137 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `print-primary-final-suite-manifest` | 0 | 1.534 | `c3bf3a7aa783ddaed73ab2fd72696e7e88cd2dc6166c9b2ee0c2668299790a3c` |
| `print-primary-final-unit-tiers` | 0 | 27.348 | `b0d22c7e9e78b2de09250df5931d136093b5d2ad3c7474ab5c37aa9c3dc75ad0` |
| `print-primary-final-full-minimal-rust` | 0 | 2921.446 | `ac7c10970c2ac64e78ee02645595af3406fedb315207ea410507974af6dfb94e` |
| `print-primary-final-original-claim-native-focused` | 0 | 582.221 | `127ece4277e37e22578e3f525ffaaed64b9beb8bce72c8910280ea09898138f1` |
| `print-primary-final-final-native-lib` | 0 | 746.193 | `f657ad700c64125dd7b90ce3077891e3994fddfabb20410eb2221ae5c9a4fbd6` |
| `print-primary-final-final-native-picture-placeholder` | 0 | 79.498 | `73bb64894444dbcaf1ff04e16ccd23d8c531adb9ca8e15080bee57e8eda7e2f7` |
| `print-primary-final-final-native-direct-pdf` | 0 | 25.605 | `4d232c59e629c4a5ae9675c07050a6e5c121094fe23263b3ac3a47dcae9c62f0` |
| `print-primary-final-final-native-cli-build` | 0 | 444.341 | `8ab31e1a41e5824e88a67b41f926aad2b4c93b40976b33d8f096abe0f22b8e4f` |
| `print-primary-final-original-claim-native-clippy` | 0 | 120.928 | `4d9e92e3c2d21c16df0d5d6149121916dc7fac9c50e9c2fcfbe11abec57fa677` |
| `print-primary-final-default-clippy` | 0 | 93.710 | `834f7552cb29ad1d460ccc1de090941aecefc19ba13e791425c57ff0cf33925a` |
| `print-primary-final-wasm-clippy` | 0 | 108.079 | `74c73d7fd8515c2fd68f76dc77b7f5905dd2e2200f7750e3477d6d9913a64267` |
| `print-primary-final-workspace-build` | 0 | 181.948 | `1399392ce0c535f6c7ceb7e5bdcc29554660af882b32d4938024e6ab5aeaaa9d` |
| `print-primary-final-workspace-clippy` | 0 | 172.524 | `6322a4eb1fbc1e0216b9a715b2df52ba59c1aac9bdccf91d38f39b4991b631c9` |

### 실제 최소 글꼴 노출

| family | 저장소 자원 | SHA-256 |
| --- | --- | --- |
| Noto Sans KR | `ttfs/opensource/NotoSansKR-Regular.ttf` | `6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74` |
| RHWP Old Hangul | `ttfs/opensource/RHWPOldHangul-Regular.otf` | `0a32ff354d11c955f819d84d91e695927d41aca0fd74b4a286b6235345101af4` |
| RHWP Public Batang | `ttfs/opensource/RHWPPublicBatang-Regular.ttf` | `7e3aee5ee1d8f44cc37d35dbd07e5f66414fee5ce552ebda6b4347643d9d0b78` |
| Noto Sans KR ExtraLight | `ttfs/opensource/NotoSansKR-ExtraLight.ttf` | `67b4003e2be99ea44a7c957e4b35acde1b5e6e82a54fe195c5598c3c617bc2e3` |
| Noto Sans Symbols2 | `ttfs/opensource/NotoSansSymbols2-Regular.ttf` | `876598db5f1e3bef2bc77c4d94c5b0d5d2c1620be4878a06f18fb5434e331688` |

## 보호 전쪽 검증과 기준 상향

- 최종 fresh WASM: 18개 입력·1,771쪽 전체 완료, 기존 정본보다 하락 0쪽·상승 15쪽.
- [WASM 전쪽 TSV](final/protected/README.md)는 입력마다 동일한 최종 실행의 전체 결과다.
- 최종 Native도 18개 입력·1,771쪽 완료, 하락 0쪽·상승 15쪽이다.
- 실제 편람 HWPX Native gate 383쪽: 1,801.002초, 종료 코드 0, 공식 최저 99.96%. 같은 실제 TSV를 보호 대조에도 사용했다.
- 최종 전쪽 수용 조건은 [정식 baseline 계약](final/protected-validation.json)에 기록한다. 중간 run manifest와 구분한다.
- 정본 보호값을 `1063d78078b704a2bfb712ca3929ab251596dd391b41bc1a9a44e9935e4990e5`에서
  `8f073dddd67cf8f44bea7c0315c3f9950ba73641e9393747cdbc52d24b33f9a1`로 상향했다.
  [상향 영수증](../visual-protected-baseline/updates/pr9075-20261115.json)에 Native 15·WASM 15쪽의 이전/새 값을 기록한다.
- Native와 필수 검증까지 완료한 같은 최종 수용본으로 상승분을 반영했다. 페이지별 최댓값을 이어 붙이거나 기존 하한을 낮추지 않았다.
- 상향 후 관련 Python 검사 21건이 모두 통과했다. 실제 완료된 편람 manifest를 새 정본으로 재대조했고, 편람 HWPX Native 383개 하한은 이전과 바이트 단위로 같음을 확인했다.
- 최종 3,542행 모두 상향된 하한을 충족한다. 데이터·해시 연결만 바뀌었으므로 같은 raster를 다시 산출하지 않았다.
- 원격 PR 생성·CI·승인·병합·원 PR 종료는 아직 실행하지 않았다.

## 이전 단계의 실행 근거

아래 SHA는 이번 검토에서 생성한 실제 실행 로그를 식별한다. 원시 로그는 Git에 넣지 않으며,
기존 단계 분석은 당시 source의 관측 기록이다. 최종 제품의 미완료 검사를 대신하지 않는다.

| 당시 로그 식별자 | 원본 SHA-256 | 관측 summary |
| --- | --- | --- |
| `canvas-image-stage/canvas-coverage-all-browser.log.txt` | `9161c1738ba0e7e3290a53fc3109980cb29ca1b1b09a6e15296cb0bd3ea8b693` | 별도 단계 분석 문서의 실행 결과 참조 |
| `canvas-image-stage/canvas-crop-border-and-jpeg-after.log.txt` | `7294830ec714c79d59403f77de3ad1c886034e52b6dc393b9f8a8f42ab24002b` | 별도 단계 분석 문서의 실행 결과 참조 |
| `canvas-image-stage/canvas-crop-border-before.log.txt` | `8daf7b9c9e76db4f22f96f187914252be09c2fa67bda92fabb63fc91f551ee0d` | 별도 단계 분석 문서의 실행 결과 참조 |
| `final/validation/final-native-direct-pdf.txt` | `4d232c59e629c4a5ae9675c07050a6e5c121094fe23263b3ac3a47dcae9c62f0` | Summary [   2.631s] 36 tests run: 36 passed, 297 skipped |
| `final/validation/final-native-lib.txt` | `52c0af5cfb2630e605a18c6d5a5b4bbb7216702f6ca379f51cbffe976baab02f` | test result: ok. 3853 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 27.25s; test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s; test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| `final/validation/final-native-picture-placeholder.txt` | `73bb64894444dbcaf1ff04e16ccd23d8c531adb9ca8e15080bee57e8eda7e2f7` | Summary [   4.086s] 2 tests run: 2 passed, 325 skipped |
| `final/validation/format.txt` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | 별도 단계 분석 문서의 실행 결과 참조 |
| `final/validation/full-minimal-rust.txt` | `ac7c10970c2ac64e78ee02645595af3406fedb315207ea410507974af6dfb94e` | Summary [1087.131s] 13562 tests run: 13562 passed (9 slow), 45 skipped |
| `final/validation/original-claim-native-focused.txt` | `127ece4277e37e22578e3f525ffaaed64b9beb8bce72c8910280ea09898138f1` | Summary [   0.117s] 3 tests run: 3 passed, 310 skipped |
| `final/validation/suite-manifest.txt` | `c3bf3a7aa783ddaed73ab2fd72696e7e88cd2dc6166c9b2ee0c2668299790a3c` | 별도 단계 분석 문서의 실행 결과 참조 |
| `final/validation/suite-prepare.txt` | `df56e6fbfe92ac822748362989f870708b99101c9b63fb328616b90fefa10e37` | 별도 단계 분석 문서의 실행 결과 참조 |
| `final/validation/unit-tiers.txt` | `b0d22c7e9e78b2de09250df5931d136093b5d2ad3c7474ab5c37aa9c3dc75ad0` | 별도 단계 분석 문서의 실행 결과 참조 |
| `hft-centered-bar-stage/hft-centered-bar-before.txt` | `1a33c541621d3a862744bb7b9a3b34efbf7143442819ff614186f4e45d02f6de` | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s |
| `hft-centered-bar-stage/hft-centered-bar-combined-build.txt` | `548631e4047617f9c0a9696035c39cd47278a6ce2a7a758466328e88d15b7c51` | 별도 단계 분석 문서의 실행 결과 참조 |
| `hft-centered-bar-stage/hft-centered-bar-final-char_overlap_caret_axis.txt` | `e5ede5fcf7e3a2438cf2203f9bcb8e15f8f51874685bdddb67beee9c5b70c601` | Summary [   0.256s] 10 tests run: 10 passed, 362 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-char_overlap_hancom_shapes.txt` | `407942d7d335769d2bcdb9e92d5bbde9a4830ecfb1356bd9bd8ecd4b3b4a040d` | Summary [   0.054s] 4 tests run: 4 passed, 308 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-char_overlap_missing_pua_host_style.txt` | `c3f64f055cc43c8af20bb7f22b5b0ec5ad2d876945f7653ccdda157b787bc4b5` | Summary [   0.820s] 2 tests run: 2 passed, 325 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-cli-build.txt` | `5bc51ea4de519805cddaa94d2f627fb129dc62ade3617972bde9a639ebb234e8` | 별도 단계 분석 문서의 실행 결과 참조 |
| `hft-centered-bar-stage/hft-centered-bar-final-clippy.txt` | `f6d0f7b67390953e79bbbec71bbf687b96639410181f8a91681a3cfb74829223` | 별도 단계 분석 문서의 실행 결과 참조 |
| `hft-centered-bar-stage/hft-centered-bar-final-combined-build.txt` | `99546de39138a888b2c943cfd58772e61b0ac6c8573bc19d4e6a4cf2a296b134` | 별도 단계 분석 문서의 실행 결과 참조 |
| `hft-centered-bar-stage/hft-centered-bar-final-hft_centered_bar_glyph_fit.txt` | `199fa3931df41e68bfc89f102ea823a734d8793588da161c10f839a69a8e57bf` | Summary [   0.161s] 2 tests run: 2 passed, 348 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-hft_declared_en_dash_advance.txt` | `3e181fabef63f26200b8771603771ea077d8b0d48e429878536ed273bb4d9355` | Summary [   0.024s] 2 tests run: 2 passed, 345 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-hft_outline_font_contract.txt` | `2dbb9c090b67a878b351aecd7374ff2188e30e93799c5b8f2174c85a7fcf729d` | Summary [   3.789s] 23 tests run: 23 passed, 339 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-issue_2225_missing_picture_placeholder.txt` | `0f9a9734785006b2b8df7e89add8ef3b824556950c938785fb315e75be1c5c99` | Summary [   2.796s] 2 tests run: 2 passed, 325 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-issue_8153_empty_overlap_carrier.txt` | `41e42d2f5b578af50debc736dfaf8e5a8e892834395ab2bf6e1d44560530a019` | Summary [   5.630s] 4 tests run: 4 passed, 336 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-issue_8153_symbol_fallback.txt` | `2983264e44d83cd92657d347929dc197c11d64f582173f29a6b6036d14cfe86c` | Summary [   0.227s] 2 tests run: 2 passed, 337 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-issue_8315_symbol_slot_advance.txt` | `16e9b04047aa1ebbcc8e988ba2283516b77a512c0f8347788633fabf545d3ccc` | Summary [   0.038s] 3 tests run: 3 passed, 369 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-issue_9060_rowbreak_first_line_fragment.txt` | `a67063f92c37f1622c14d9fe9c58bf5c1125adf741605da34b706c2f33facebd` | Summary [   0.087s] 3 tests run: 3 passed, 327 skipped |
| `hft-centered-bar-stage/hft-centered-bar-final-lib.txt` | `d36316b3b88c0872cbca31778c055ea2d30d2e6cc6f40f622690c8277215cfaa` | test result: ok. 3853 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 48.55s; test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s; test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| `hft-centered-bar-stage/hft-centered-bar-final-render_p37_direct_pdf_export.txt` | `859acce79293278117338c2dd7b71e6dcf16e6426c149568bf5f0916b3516413` | Summary [   3.358s] 36 tests run: 36 passed, 297 skipped |
| `hft-centered-bar-stage/hft-centered-bar-helper-registration-node.txt` | `3a836875368a00444da37e33eb0cc1600babc769cbd3782fcabbc1ae4d70c6f7` | 별도 단계 분석 문서의 실행 결과 참조 |
| `native-slant-family-stage/validation-logs.txt` | `e53024c18bd0a87cc8ffbbfdac4bd375c6c2164d55426226456147308c8718d0` | Summary [   0.641s] 7 tests run: 7 passed, 330 skipped; Summary [   0.700s] 2 tests run: 2 passed, 341 skipped; Summary [   0.756s] 6 tests run: 6 passed, 368 skipped; Summary [   2.327s] 23 tests run: 23 passed, 340 skipped; Summary [   2.189s] 2 tests run: 2 passed, 325 skipped; Summary [   2.932s] 36 tests run: 36 passed, 296 skipped; test result: ok. 3853 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 30.51s; test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s; test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s |
| `nested-source-contracts-stage/hft-en-dash-before.txt` | `27db172c4060a4a4172aff54cbcf64bbeff53069643d8bd578fbe23acfd3fb69` | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| `nested-source-contracts-stage/missing-pua-host-before.txt` | `c9399f8d1222fe2b86c270fdf76199e331ad53522fcb682d7827491a927d3c89` | test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s |
| `nested-source-contracts-stage/native-validation-logs.txt` | `7d0035ea582d1733a96ac580eea072212c3d2f46cef1b8b5ab19821d96e5b0e8` | Summary [   0.434s] 2 tests run: 2 passed, 345 skipped; Summary [   0.023s] 2 tests run: 2 passed, 335 skipped; Summary [   0.049s] 3 tests run: 3 passed, 366 skipped; Summary [   0.057s] 4 tests run: 4 passed, 308 skipped; Summary [   0.184s] 10 tests run: 10 passed, 359 skipped; Summary [   3.940s] 4 tests run: 4 passed, 335 skipped; Summary [   2.618s] 23 tests run: 23 passed, 340 skipped; Summary [   0.076s] 3 tests run: 3 passed, 328 skipped; Summary [   2.109s] 2 tests run: 2 passed, 325 skipped; Summary [   2.633s] 36 tests run: 36 passed, 295 skipped; test result: ok. 3853 passed; 0 failed; 12 ignored; 0 measured; 0 filtered out; finished in 28.77s; test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s; test result: ok. 165 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s |
| `print-primary-stage/after-browser.txt` | `d487fcc8b9910d2a39e18dea0986055e25d137f9a0b65332bdb3be253a468e8e` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/before-browser.txt` | `cffb14c5a85c7c73fc88a95cd2b117ea092234d62bfaeef32b9582141e38a5e4` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/css-chain-before-browser.txt` | `b79e56ee09a717369b87feaa831f4c32c251fe1bf72958e3243d5bda4746a48c` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/font-registration.txt` | `b3d308e6c21f6017d20894b319a003adb0b28f836dbd6a1e4fe90a60c615b321` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/minimal-font-exposure.txt` | `bec9d066c0393bb053565b5c56d7111231ad7dbb5e7f2faefdd9f5cdbf3d2487` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/optimized-wasm-build.txt` | `c9a4c06e7a3f9c8066120afaef8838fee1feaf13dacf616e545787fcf43a7c78` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/shared-range-studio-unit.txt` | `d3ac0f815a53427c54e71db04dd58c59a7cbec49a61773c5ef3babb0df6e4da8` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/shared-range-typecheck.txt` | `0c1cf388c77b76ab855176304eac4842047db6b2aa707407626cbcf0d0415533` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/studio-all-node.txt` | `7e0e12661d8c82cf1b481b039c071957e5ae683c58e119ae910eaac441f3fa56` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/studio-typecheck.txt` | `0c1cf388c77b76ab855176304eac4842047db6b2aa707407626cbcf0d0415533` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/studio-unit.txt` | `3c0e4ba9dfd0022436ae7862f6a155b44af1ada935ae0984d959f1d432fc9d47` | 별도 단계 분석 문서의 실행 결과 참조 |
| `print-primary-stage/wasm-check.txt` | `c730a8f0d3b4187f7c82aeee7bbd503a618d6a9ec3fd236318a8a03a6d88d7cc` | 별도 단계 분석 문서의 실행 결과 참조 |

## 최종 대표 파일의 무결성

| 파일 | SHA-256 |
| --- | --- |
| `final/main/native_silhouette.tsv` | `9d8f34337c868e3d2cad49121bf162d68dd6b80d10680df138612452480f5d09` |
| `final/main/native/review_005.png` | `a686b731b16e27c8b389892eb20627bfa56c5f20835015898b386e178e3e1500` |
| `final/main/native/overlay_005.png` | `d0fa4334744397601c4220eb25dfabdac83c54b5044218e8a81f47ba025095d4` |
| `final/main/native/review_006.png` | `f560afe98223106be96943bd2c3954e8858ea102961d21642ac01101d91d9b0d` |
| `final/main/native/overlay_006.png` | `df438a15478475e2a5b2700c2dc90eb95179678a7009536f92f569b2dbd90111` |
| `final/main/wasm_silhouette.tsv` | `9d8f34337c868e3d2cad49121bf162d68dd6b80d10680df138612452480f5d09` |
| `final/main/wasm/review_005.png` | `39259739ad0783a446cc21d784ddbca9575ebcd6f5964656fe9aa617dfb3f515` |
| `final/main/wasm/overlay_005.png` | `a6f06205a3eba3072218e9a20279e53215314950689aa1aaeb6ac11c5bd08c84` |
| `final/main/wasm/review_006.png` | `839b5edf150ae539edf644a23dc190e3e0551ccbf0f8a5077f30d53b7b71048c` |
| `final/main/wasm/overlay_006.png` | `d7d3f3940959bac91be8e54e97fc0c00e8255fa7216e83320fe00aa716aea7a0` |

## 내용 보존 추가 회귀

기존 세 검사의 assertion은 유지했고, `split_fragments_preserve_all_source_text_and_following_heading`를 새로 추가했다.
두 조각이 소유하는 줄과 셀 전체 텍스트의 누락·중복, 후속 제목이 7쪽에 한 번만 나오는지 검사한다.
기본·Native 최소 글꼴 환경에서 각각 네 함수가 모두 통과했다. 제품 소스는 기존 측정 source와 같고,
전체 13,562건 통과에 추가한 한 함수를 별도로 실행해 현재 13,563개 고유 검사의 범위를 연결한다.

포맷이 소스 바이트 수 기반 suite 배정에 영향을 주어 첫 manifest 검사가 중단됐고,
포맷 뒤 suite를 다시 생성해 실제 배정된 `regression_suite_019`에서 검사·Native Clippy를 실행했다.
실물 출력·기대값·입력을 바꾸거나 generated 파일을 커밋하지 않았다.

- 검사 소스 SHA-256: `07f0618f55c29b1eb50ea535622a945b86e5ea1c9c58ee37a33c0af36ff67dce`.
- 추가 단계의 필수 default/WASM/workspace lint도 모두 종료 코드 0이다.

| 추가 검사 단계 | exit | 초 | 원시 로그 SHA-256 |
| --- | ---: | ---: | --- |
| `prepare` | 0 | 2.021 | `9938e96ffe09d8c5be92cdaa3434e9d899e342cc6e8682417f6c661100ba19ba` |
| `format` | 0 | 35.549 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `prepare-after-format` | 0 | 2.039 | `9938e96ffe09d8c5be92cdaa3434e9d899e342cc6e8682417f6c661100ba19ba` |
| `format-check` | 0 | 36.476 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `manifest` | 0 | 1.087 | `437a02411f2d41193935c1cc6dedf687bec2161277b80b5b1077293da87d682b` |
| `default-focused` | 0 | 491.095 | `7d18bfe06bdd07cb5865a9e0fc61567f87f840c61405d267b1846b8856eef085` |
| `native-focused` | 0 | 459.337 | `ba9d25daa92dbaf439fbc7fa92d9b5b7e98963b0a577605fc0bd6b5a6009b6c1` |
| `native-test-clippy` | 0 | 13.566 | `3d6b28fca5295d50f1884e62b8496406acf1ab46a8ef01d0bae0612d370db88d` |
| `default-clippy` | 0 | 1.306 | `57bcdf8fbdad4529b29d74839a73c59a27c793a90195e6279bcd8763621ab49b` |
| `wasm-clippy` | 0 | 3.199 | `5c05bd74bc59c78d52c4084fbf3111ec685ca88e6c55da828e347ae58c7e2576` |
| `workspace-build` | 0 | 2.449 | `8f8b1dce1679886c7a6a71d75b3045d4de5bc0750769a34a065ab4f0bf2212e4` |
| `workspace-clippy` | 0 | 30.111 | `34a594dd70fe68c1686c1ab55efe4d88674499d4607880bcc9f3ed5a27b19224` |

## 보안 검사 실행 범위 정정

최초 보안 명령의 `--tests`가 필터와 무관한 전체 suite 재컴파일까지 요청하는 것을 확인했다. 해당 실행만 중단하고 실제 두 보안 모듈의 `regression_suite_021`·`regression_suite_010`을 명시해 다시 실행했다. 신규 입력·세 검출기·assertion은 바꾸지 않았다. 위 완료 결과는 정정한 명령의 실제 종료 코드이며, 중단한 실행을 통과로 계산하지 않는다. 이미 완료한 전체 Rust 13,583건 결과는 별도다.

## 제출 문서 확인

정본 `scripts/check_markdown_links.py`의 동일 경로 수집·내부 링크·capability 등록부 검사 함수를 대상 Markdown 27개에 실행해 오류 0건을 확인했다. 변경하지 않은 redirect 검사를 위해 모든 추적 바이너리를 읽던 중복 범용 실행은 종료했다. 검사기 SHA-256은 `03a292f69bff96e9fc91ae1d111596e9fa76b6634224b78aed2cd679c6ded009`이며 검사기 자체는 수정하지 않았다. `git diff --check`도 통과했다.
