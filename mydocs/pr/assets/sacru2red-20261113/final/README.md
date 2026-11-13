# #7620 메인테이너 간격 보정 — 최종 검증 원장

현재 제품 source는 `e5a6a1ff9f`, 정식 회귀 보완은 `b4b8c27260`이다. 필수 로컬 검증을
2026-11-13에 완료했다. 원격 새 head의 CI·승인·통합은 아직 수행하지 않았다.

## 실제 개선과 독립 기대값

같은 원본 e·f의 Desktop Hancom 2020 `11.0.0.9136` Print(method 0, one-up)를 기준으로
Native Skia와 fresh 최적화 WASM Canvas를 96dpi·compat 2022에서 대조했다.
원본·기준 해시와 출력 경로는 [보정 전 원장](../README.md) 및 `samples/issue7620/MANIFEST.json`에 있다.

| 입력 | 전체 쪽수 | Native 전쪽 최저 | fresh WASM 전쪽 최저 |
| --- | --- | --- | --- |
| e-cellbreak | 2 | 100.00% | 100.00% |
| f-rowbreak | 2 | 100.00% | 100.00% |
| footer 대조군 | 13 | 96.17% | 96.11% |
| 원본 a-RowBreak / 원본 None 대조군 | 각 1 | 각각 100.00% | 각각 100.00% |

전쪽 TSV는 `tsv/`에 있다. `*-three-way.png`는 독립 Print·Native·fresh WASM을 같은 배율로
배치한 실제 출력이다. 행의 쪽 소속·모든 셀 글줄·표 외곽·뒤 문단·원래 빈 줄을 직접 확인했다.
대상에는 수식이 없으며 글자·줄·표의 실제 간격을 확대 대조했다.

빈 문단의 원본 LineSeg는 높이 1000HU·후행 간격 600HU이며 다음 줄까지 1600HU다.
`spacing-relation-probe.tsv`는 같은 원본의 실제 렌더 트리에서 이 전진 관계가 보정 전에는
Native/WASM 모두 FAIL, 보정 후에는 모두 PASS임을 기록한다. 절대 px 위치를 고정하지 않는다.
정식 Rust 회귀 2건에도 빈 줄·뒤 본문 소속과 원본 전진량 관계를 추가했다.

`devel-comparison.tsv`는 고정 devel `180a4ec972`와 보정 후보의 Native 19쪽·WASM 19쪽을
동일 조건으로 대조한다. 38행 모두 하락이 없고 footer의 양쪽 13쪽 PNG는 각각 보정 전후 바이트가 같다.
Native binary SHA-256은 `cc7dbf797446b9ff58fb2305001445fa8a2de00537688579491550e07002934a`,
fresh WASM은 `4b89b130aa5ff17899da91559ff97e20f6f2e9c4be7894b88d290a62875e0462`,
JS는 `f28c25b4a470ef133de11ea6240436fbbc78f7eb29d3904e758a31bcf4131094`다.
Native는 `cargo build --locked --profile release-test --features native-skia-gl --bin rhwp`,
WASM은 `RHWP_WASM_CDYLIB_ONLY=1 scripts/wasm-pack-locked.sh --target web`의 실제
release 최적화·wasm-opt 결과다. 두 빌드의 target은 루트 `target/pr-review`다.

추가로 실제 CLI SVG를 Chromium에서 래스터화했다. [별도 전쪽 대조](svg-comparison.tsv)에서
보정 전 e·f 2쪽은 각각 94.49%·93.19%, 보정 후 전 4쪽은 모두 100.00%다.
SVG 문자열만 검사하지 않고 표·뒤 문단의 실제 잉크와 간격을 직접 확인했다.

### 원본 정상 대조군

원 이슈의 실제 입력 해시와 같은 `samples/issue7620/a-rowbreak-float.hwp`(RowBreak) 및
`samples/issue7621/control-float.hwp`(None)를 재사용했다. 새로 만든 입력으로 바꾸지 않았다.
Desktop Print의 실제 제품·출력 방법은 위 e·f와 같고, 두 입력의 수정 전후 Native/WASM은
각각 1쪽이며 모두 100.00%다. [원본별 전쪽 값](normal-controls-comparison.tsv)을 보존했다.
[같은 영역 직접 대조](normal-controls-three-way.png)는 각 쪽의 원래 y=90–350px 영역을
이동·배율 변경 없이 나란히 놓은 것이다. 앞 본문·표 괘선·셀 내용·뒤 문단의 간격을 확인했다.

| 파일 | SHA-256 |
| --- | --- |
| 원본 a HWP | `44c065e5a770e88903e314b53e1b199f2ab1385e50a79eebb5e8ae8716ec64a3` |
| 원본 None HWP | `81d61a11efd5070e64767b8cd32491ba8af93ec212351607694e10aece0a9480` |
| `pdf/issue7620/a-rowbreak-float-2020.pdf` | `55929dc8751d50afbfdbf1679536537d0c94376c6026b7e2f2dc605d0a5a5025` |
| `pdf/issue7620/control-float-2020.pdf` | `adb141318dcba5240d5794bdbf1245e9c6c4e1569efb9d24b0f79471993b1677` |

원본 a의 MCP job은 `000edbfc-a50b-45fa-a880-d5b69e723c02`, None은
`65d40f47-4e8c-4e71-bb87-5b921eccb37b`다. 다운로드 결과의 입력 구분용 이름을 위 정식
PDF 경로로 옮겼으며 PDF bytes는 그대로 보존했다.

## 공통 경로와 기존 검사

공통 `HeightCursor::vpos_adjust`가 실제 소비한 줄간격을 lazy 원점에서 중복 역산하지 않게 했다.
기존 단 원점이 있으면 그 축의 일치 검사도 유지하며, 실제 줄 하단 누락·불일치, 비연속·합성·편집,
모순되는 기존 원점의 bridge는 유지한다. fit과 paint가 같은 커서와 실제 줄 상자 하단을 소비한다.

기존 `lazy_path_applied_and_base_set`의 원점 부재 case 6은 실제 줄 하단 116px·흐름 124px,
저장 1000+1000+600HU를 주면서 간격을 한 번 더 계상하도록 기대했다. 독립 e·f Print와
실제 전진 관계로 이 전제가 틀림을 확인했다. 기대 상대 기준을 800HU로 보정했고 나머지 부정 대조군은 유지했다.
새 source-side 테스트 모듈이나 정본 수치·허용치를 추가·완화하지 않았다.

## 필수 검사 결과

- Native Clippy·WASM Clippy·workspace build·workspace all-target Clippy: 통과.
- 최소 글꼴 집중 회귀: 8 PASS. 전체 release-test는 13,432 PASS·45 정상 skip이며
  본문 초과·셀 초과·off-canvas·text overlap·IR·쪽수·새 샘플 보안 검사를 포함한다.
  실행 wrapper가 SIGTERM으로 종료돼 exit 영수증은 없지만 자식 nextest의 실제 최종
  Summary와 13,432건의 전체 PASS 로그를 확인했다. exit 0을 직접 수집한 결과와 구분한다.
- Native 공식 3종: 라이브러리 4,035 PASS·12 정상 ignore, placeholder 2 PASS,
  direct PDF 36 PASS, 합계 **4,073 PASS**. 세 명령 모두 exit 0이다.
- 최종 fmt·unit tier 계약·suite manifest 검사: 통과. 기존 source-side 단위 검사 수는 늘리지 않았다.
- 격리된 Fontconfig에 노출된 프로그램은 저장소의 NotoSansKR Regular/ExtraLight,
  NotoSansSymbols2, RHWPOldHangul, RHWPPublicBatang 5개다. 시각 검증 글꼴과 분리했다.
- 편람 HWPX Native 383쪽 `--handbook-gate`: 최저 99.96%, 공식 값 하락 0.
- 정본 보호 전체: Native 1,771쪽·WASM 1,771쪽, 각각 18개 데이터셋의 **공식값 하락 0**.
  [전체 비교](protection/comparison.tsv)와 `protection/native/`, `protection/wasm/`의
  전쪽 TSV 원본을 보존했다. 정본 `visual-protected-baseline/final.tsv`와 각 데이터셋의
  입력·PDF 신원을 고정했고 허용치·정본·측정 방법을 바꾸지 않았다.
  정본 SHA-256은 `ee41455cf7f05585487d92961ccb9fd8160d5d1e15c3567019ea89e2b7bf2aeb`다.
- 최초 편람 HWP 145쪽은 보정 전후 PNG가 같았으며,
  원 보호 기록의 `handbook145-print-observed-20261031` 글꼴 환경을 복원해 100.00%를 확인했다.
- #7905의 Native 표시값 2행은 보정 전후 PNG가 전 6쪽 모두 같았다. 원 검증의 세 디렉터리
  공급을 개별 파일 인자로 풀어 준 것이 원인이었다. `font_paths.rs`는 경로별 그룹을 만들고,
  `skia/font_catalog.rs::load`는 첫 그룹의 가족에 속한 style 묶음을 선택한다.
  같은 23개 파일도 그룹을 나누면 첫 face만 그 가족을 선점한다. 기존 실행 명령의 디렉터리
  경계·파일명·프로그램 SHA를 복원하니 동일 binary로 6쪽 모두 공식값 이상이며
  1·2쪽은 원 기록과 같은 99.97%·98.86%다. GL/CPU 변경이나 제품 보정·기준값 예외는 사용하지 않았다.
- yeongwol은 원 실행의 `planet6897_20261103/yeongwol-font-environment.json`을 다시 공급해
  원래의 새굴림→Malgun Gothic 선택을 복원했다. 중간 자동 캐시 JSON만 따르며 원 인자를
  생략한 실행과 구분한다. 명시 글꼴 선택은 점수를 보고 새로 고른 것이 아니라 이전 실제 명령의 조건이다.
- 옛 웹폰트는 원 URL·원본 SHA와 canonical 변환 옵션(`recalcTimestamp=False`,
  `reorderTables=False`)으로 같은 Native 프로그램 SHA를 확인했다. 독립 구형 수식 probe도
  동일 입력의 실제 Desktop Print에서 준비했으며 기존 75/77개 Type3 프로그램·폭이 각각 같다.
  OTF 이름 ID/시각에 따른 파일 SHA 차이와 동일 프로그램 증거를 구분하고 윤곽·전진을 수정하지 않았다.

원시 로그·JSON·중복 래스터·독립 글꼴 표본과 로컬 글꼴 프로그램은 루트의 ignored `output/`에만 있다.

위 Rust 실행은 공용 `CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review`,
빌드 8 job이며 회귀는 문서화한 최소 글꼴 격리에서 실행했다. 핵심 명령은 다음과 같다.

```bash
cargo clippy --locked -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings
cargo build --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast --test-threads 4
cargo test --locked --profile release-test --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --features native-skia
cargo fmt --all -- --check
node scripts/rust-unit-test-tiers.mjs --check --base-ref 180a4ec97205b67c4dc4f90416063c01ecfdf396
node scripts/rust-test-suite-manifest.mjs --check --base-ref 180a4ec97205b67c4dc4f90416063c01ecfdf396
```

격리 환경에는 NVM의 Node 실행 파일만 읽기 전용으로 bind했다. 최초 Node 경로가 보이지 않아
보조 명령이 시작 전에 끝난 결과는 성공에 포함하지 않고, bind 뒤 실제 실행한 2건·36건을 채택했다.
