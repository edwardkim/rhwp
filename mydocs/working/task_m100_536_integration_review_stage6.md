# seo-rii 통합 검토 6단계 — 최종 gate와 직접 출력 판독

## 검증 후보

최종 Rust·frontend gate는 `068769e9d4dd800ed9caca00a61589f3ef291906`에서 시작했다.
고정 base는 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`이다. 이 후보는 renderer
`d9a0b1c79` 이후 정식 검사·문서·독립 Print PDF만 추가했으며 production Rust/Studio source는 같다.
검증 도중 작성한 이 문서와 원 PR별 review는 아직 실행되지 않은 검사를 완료로 표시하지 않는다.

## 완료한 최신 frontend·ABI 선언 검사

- TypeScript, frontend unit **2,310 PASS / 0 FAIL / 2 skipped**, production build와 renderer
  contract를 모두 통과했다. 명령·소요 시간은 `final-frontend-gates.json`에 있다.
- 실제 Chromium CanvasKit의 정상 strict 2건과 overflow 전체 fallback 18건, 실제 font coverage와
  shaping replay를 통과했다. #7688의 실물 글꼴 준비·두 style 공급 순서·screen/print·실패 시 Canvas2D
  전환도 다시 통과했다. 기존 문서 폰트를 생략해 잘못된 기본 face를 조용히 표시하지 않는다.
- 같은 renderer 후보의 실제 `equation-lim.hwp` layer를 CanvasKit layout 경로로 재생했다.
  공급한 HyhwpEQ로 정상 출력했고 unsupported op·unregistered font fallback은 0이었다.
  이름 child를 제거한 기존 fallback 대조군과 실제 raster가 달라 optional child 소비를 확인했다.
  원본 SVG 우선 경로와 layout 경로를 분리한 진단이며, 변형 payload를 정상 Print fixture로 등록하지 않는다.
- C와 C++에서 public FFI header를 각각 `-Wall -Wextra -Werror -fsyntax-only`로 검사해 통과했다.
  Apple SDK·C# runtime을 실행했다는 뜻은 아니다. 실제 feature-on FFI 9종은 5단계 기록을 따른다.
- frontend 검증에만 동기화했던 worktree public JS/d.ts는 소유 변경만 복원했다.
  주 작업공간의 사용자 JS는 처음 해시 `fd499c9eb514018d2063c8eef1e58ec892c734d512cf5e88ab97bd9e5d47a0a3`
  그대로다. 공용 target과 다른 작업 branch는 보존한다.

## 같은 capture의 직접 판독과 대표 PNG

renderer `d9a0b1c79`의 전쪽 Native/fresh WASM capture에서 canonical `make_compares`,
`make_overlay_compares`, `make_review_panels`로 review·standalone overlay를 다시 합성했다.
새 head에서 재렌더했다고 표시하지 않는다. 원 capture manifest와 raster 해시는
`final-visual-panels/<backend>/<key>/panel-provenance.json`에 연결했다.
전체 페이지 TSV·점수·쪽수는 [5단계](task_m100_536_integration_review_stage5.md)와 같다.

| 원본·영역 | 직접 판독 결과와 범위 |
| --- | --- |
| lseg-01-basic / lseg-02-mixed 전체 1쪽 | 두 backend에서 본문·다음 문단의 소속, 줄바꿈·내용 순서와 누락/중복을 대조했다. 기존 글꼴 raster 차이를 완전한 픽셀 일치로 표시하지 않는다. |
| shape-group-02 전체 1쪽 | Native와 WASM 모두 2px 실루엣 100%. 외곽 path·그림자·위아래 gradient 방향과 경계가 Print와 대응한다. 엄격 RGB 픽셀 100%라는 뜻은 아니다. |
| pic-crop-01 전체 1쪽 | 두 그림의 crop·내용 보존·상하 순서·원점과 외곽을 대조했다. 미세한 raster/테두리 차이는 base 대비 확대되지 않았다. |
| form HWP / HWPX 전체 1쪽 | 5개 폼과 표시 문자열·체크/선택 상태·배치 대응을 확인했다. 각 backend 내 HWP/HWPX의 실제 page PNG 해시가 같다. Edit의 기존 윗·왼쪽 bevel 차이는 남아 있으며 폼 외형 전체를 새로 해결했다고 주장하지 않는다. 독립 Print는 사용자 HWPX 수동 출력이고 HWP 직접 Print로 바꾸어 부르지 않는다. |
| equation-lim 원본 1쪽, 동일 좌표 4배 확대 | HyhwpEQ 이름의 face·em 크기·scalar 순서와 첨자/앞뒤 분수의 관계를 대조했다. 분자·괄호의 기존 간격 차이는 남아 있다. 고정 devel과 보정 후보의 분수 영역 `(138,132)-(239,171)`은 Native와 WASM 각각 변경 픽셀 0이며, layer의 분자·분모·괄호 배치와 공통 scale도 같다. 해당 영역을 이번 수정의 해결 성과로 표시하지 않는다. 전체 TSV 범위는 줄이지 않았다. |

원래 7개 입력에서 최종 코멘트에 사용할 대표 review·overlay 12개를
[`mydocs/pr/assets/issue_536_seo_rii_20261017/`](../pr/assets/issue_536_seo_rii_20261017/)
아래 Native/WASM별 shape·극한·폼으로 보존 준비했다. 합계 668,500 bytes이며 원본과 복사본의
SHA-256을 대조했다. `final-visual-asset-plan.json`에 원 경로·stable 경로·해시가 있다.
PR 본문과 merge 후 원 PR 코멘트는 각각 실제 head/merge SHA에 고정한 Markdown 이미지로 표시한다.
진단 입력의 낮은 결과는 [별도 README](../pr/assets/issue_536_seo_rii_20261017/diagnostics/README.md)에
명시했고 입력·독립 Print를 재현 가능하게 보존한다. 정상 fixture·golden·승인 예외로 추가하지 않는다.

## 실제 Native 편집 후 저장본의 추가 대조

실제 `native_host.py`와 보정 라이브러리에서 새 문서 → 한글 조합 `ㄱ/가/각` → commit →
undo/redo → HWPX 저장·acknowledge → 재열기를 실행했다. 저장 전 live editor와 재열기한 viewer의
794×1123 RGBA frame은 SHA-256
`51c6088a280c146a80b7ca91664f7426a662dd422b4dce6b069594ad7b341722`로 바이트까지 같다.
별도 반복에서 생성한 저장본도 동일했다. 실제 함초롬바탕·돋움 파일을 `setFonts`로 공급한
대조군과 기본 frame도 같았다. 이 실물 범위의 캐시·재표시·저장 상태를 확인했으며 모든 문서·언어로
일반화하지 않는다. `repaint-contract.json`, `font-supply-control.json`에 결과·글꼴 해시를 남겼다.

- 입력: [`native-ime.hwpx`](../pr/assets/issue_536_seo_rii_20261017/native-ime.hwpx), 7,337 bytes,
  SHA-256 `dea591e15e91d9c23557a1a7630da0f80391ff9ea051fe3600b14ec5cb42adf3`.
- 이 파일은 Native API로 생성했으며 기존 template의 저장 제품 메타데이터를 그대로 가진다.
  한컴이 생성한 원본이라고 표시하지 않는다. 원본을 변경하지 않은 실제 한컴 Print를 별도로 얻었다.
- 독립 기준: [`native-ime-2020.pdf`](../../pdf/seo-rii-20261017/native-ime-2020.pdf), 9,150 bytes,
  SHA-256 `b58896a37a4cfb6ba73ea0e741b5aad20abe6a932f034f10db2cbcd8ce1e9c37`.
  job `db0f694e-41fc-41eb-91c0-35d92a8bdf3e`, Hancom `11.0.0.9136`, 전처리 없음,
  `pdf_print_method=0` / `hancom2020_pdf_driver_one_up`, 전체 1쪽이다.
- 고정 devel과 후보 각각 전체 Native/fresh WASM 모두 **100.00000%**, 쪽수 차이·하락 없음.
  기존 7개와 합쳐 현재 정식 비교 입력은 8개다. 실제 Native API의 screen frame도 별도로
  같은 Print와 100%였으며 이 경로를 Native CLI print capture로 바꾸어 표시하지 않는다.
- 같은 좌표 8배 확대에서 글자 `각`의 획·위치·내용 단일 보존을 확인했다. Skia/브라우저/PDF의
  미세한 잉크·hinting 차이가 있으므로 2px 100%를 엄격 픽셀 100%로 표시하지 않는다.
- 추가 capture는 worktree `068769e9d`에서 d9의 동일 production 실행 파일·WASM을 사용했다.
  `native-host-edit-sweeps.json`, `native-host-edit-panels.json`에 두 SHA의 역할과 capture hash를 구분했다.
  대표 review·overlay 6개를 Native·WASM·Native API screen별로 추가 보존했다. 전체 대표 PNG는 18개다.

## 실제 Direct PDF 소비 경로

보정 라이브러리의 실제 `rhwp_export_pdf_direct`로 기존 `shape-group-02.hwp` 전체 1쪽을
새 PDF에 출력했다. 선형 shader의 `/ShadingType 2`와 이미지 객체 부재를 확인했다.
이 파일은 rhwp 산출물이며 독립 기준 PDF로 등록하지 않는다. Poppler 96 DPI로 raster하여
같은 원본의 독립 한컴 Print와 canonical compare·overlay·review를 만들었다.
2px 실루엣 **100.00000%**, 누락·쪽수 차이 없음을 확인했고 gradient 방향·명암,
바깥 path와 그림자를 직접 판독했다. 엄격 RGB 일치율은 98.50%이며 2px 점수와 구분한다.
실제 ABI 응답·출력 SHA는 `direct-pdf-shape-group-02.json`, 벡터 및 시각 결과는
`direct-pdf-shape-review.json`에 연결한다. source/PDF 변환 명령의 인자·결과를 보존한다.
대표 review·overlay 2개를 stable `direct-pdf/` 경로에 추가해 전체 대표 PNG는 20개다.
이 확인은 Direct PDF의 실제 gradient 소비를 다루며 Native/WASM 전쪽 원본 TSV를 대체하지 않는다.

## Rust 최종 gate — 로컬 필수 검증 완료

모든 검사는 code candidate `068769e9d4dd800ed9caca00a61589f3ef291906`에서 순차 실행했다.
고정 base는 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`이다. 이후 이 기록·대표 PNG·실제 사용한
진단 입력과 Print PDF만 추가하며 production Rust/Studio source는 바꾸지 않았다.
전체 Cargo는 빌드 잡 8개·공용 `/home/tsjang/rhwp/target/pr-review`를 사용했고 동시에 실행하지 않았다.

| 검사 | 실제 결과 |
| --- | --- |
| suite/unit-tier 정책 | 1,528 sources / 6,745 static attrs, 28 suites + 20 exceptions = 48/48, unit 4,103 / 296 modules. 기존 예산 완화 없음 |
| release build | exit 0 |
| release lib | 3,986 PASS / 0 FAIL / 12 ignored, 4 binaries |
| 전체 nextest | 10,643 PASS / 0 FAIL / 45 skipped, 58 binaries, 실행 924.922초 |
| 변경 case Native 집중 | 277 PASS / 0 FAIL / 4,691 filtered, 28 cases / 22 binaries. standalone glyph lowering 20건도 실제 PASS 확인 |
| 공식 Native Skia lib | 4,043 PASS / 0 FAIL / 12 ignored, 4 binaries |
| 공식 missing-picture / Direct PDF | 각각 2 PASS / 4 PASS |
| format / Native Clippy / WASM Clippy | 모두 exit 0 |
| workspace build / workspace all-targets Clippy | 모두 exit 0 |
| doc test / diff whitespace 검사 | 모두 exit 0 |
| 실제 feature-off ABI | 별도 default FFI 라이브러리 빌드 후 PDF·session Python contract 모두 exit 0 |
| 기존 HFT outline Native 검사 | canonical case resolver와 native-skia로 실행, exit 0 |

전체 nextest의 package는 workspace default-members `rhwp`, `rhwp-contracts`, `rhwp-ooxml-chart`,
`rhwp-password-crypto`를 그대로 포함하고 `rhwp-native-ffi`를 추가했다. FFI `host_panic`의 실제 PASS를
확인했다. Native 집중 필터는 grouped module prefix에 별도 `binary(/^issue_4969_shaping_glyph_lowering$/)`를
OR로 연결해 standalone 20개가 빠지지 않게 했다. 필터 제외 건수를 PASS로 세지 않는다.

feature-off 검사에는 Skia가 켜진 기존 `.so`를 재사용하지 않았다. default `rhwp-native-ffi`를 dev
profile로 빌드한 뒤 별도 복사본의 해시와 source SHA를 기록했다. 실제 PDF와 session contract의
`--native-skia disabled` 경로를 실행했다. feature-on 실제 9종은 앞 단계의 별도 결과로 유지한다.

### 재현 명령과 실제 시간

아래 명령은 모두 위 worktree에서 실행했다. 공용 target과 `CARGO_BUILD_JOBS=8`을 사용했다.
긴 Native 집중 필터의 전체 인자·선택 결과는 `final-rust-gates.json`과
`final-focused-native-selection.json`에 보존했다.

| gate | 명령 | 전체 소요 초 |
| --- | --- | ---: |
| final-manifest-policy | `node scripts/rust-test-suite-manifest.mjs --check --base-ref fe5725dc384689b45d60ef8ea4c83138bc2701d8` | 1.4 |
| final-unit-tier-policy | `node scripts/rust-unit-test-tiers.mjs --check --base-ref fe5725dc384689b45d60ef8ea4c83138bc2701d8` | 21.9 |
| final-release-build | `cargo build --locked --release --target-dir /home/tsjang/rhwp/target/pr-review` | 1229.8 |
| final-release-lib | `cargo test --locked --release --target-dir /home/tsjang/rhwp/target/pr-review --lib` | 1101.0 |
| final-full-nextest | `cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests -p rhwp -p rhwp-contracts -p rhwp-native-ffi -p rhwp-ooxml-chart -p rhwp-password-crypto --test-threads 12 --no-fail-fast` | 1776.1 |
| final-focused-native | `cargo nextest run --locked --cargo-profile release-test --features native-skia --target-dir /home/tsjang/rhwp/target/pr-review <22개 --test 및 위 OR 필터> --test-threads 12 --no-fail-fast` | 498.8 |
| final-skia-lib | `cargo test --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --lib -- --test-threads 12` | 443.8 |
| final-skia-issue_2225_missing_picture_placeholder | `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 21.9 |
| final-skia-render_p37_direct_pdf_export | `node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 3.4 |
| final-fmt | `cargo fmt --all -- --check` | 38.4 |
| final-native-clippy | `cargo clippy --locked --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 108.4 |
| final-wasm-clippy | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 181.7 |
| final-workspace-build | `cargo build --locked --workspace --target-dir /home/tsjang/rhwp/target/pr-review` | 190.7 |
| final-workspace-clippy | `cargo clippy --locked --workspace --all-targets --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | 193.3 |
| final-doc-tests | `cargo test --locked --doc --target-dir /home/tsjang/rhwp/target/pr-review` | 179.9 |
| final-diff-check | `git diff --check` | 0.2 |
| final-ffi-disabled-build | `cargo build --locked -p rhwp-native-ffi --target-dir /home/tsjang/rhwp/target/pr-review` | 164.1 |
| final-ffi-disabled-pdf | `python3 bindings/Native/tests/pdf_direct_contract.py --library /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/final-disabled-librhwp_native_ffi.so --native-skia disabled --sample samples/lseg-01-basic.hwp` | 0.2 |
| final-ffi-disabled-session | `python3 bindings/Native/tests/session_contract.py /home/tsjang/rhwp/output/pr-review/seo-rii-20261017/final-disabled-librhwp_native_ffi.so --native-skia disabled` | 0.4 |
| final-hft-outline | `node scripts/run-rust-test.mjs hft_outline_font_contract -- --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia` | 36.3 |

실제 summary·exit code·source SHA·전체 명령·원 로그는 ignored
`output/pr-review/seo-rii-20261017/final-rust-gates.json`, `final-extra-contracts.json`,
`logs/final-*.log`와 각 summary JSON에 있다. TSV 원본은 base/head × Native/WASM × 8개 입력의
32개를 바이트 그대로 별도 제출 묶음에 복사하고 해시를 대조했다. ZIP·Git commit으로 대체하지 않는다.
이후 사용자가 원문을 PR 본문에 직접 넣도록 지시했다. 실제 제출 형식과 최신 base 재검증은 [7단계](task_m100_536_integration_review_stage7.md)를 따른다.

로컬 구현·필수 검증은 완료해 원 PR별 판정은 **메인터너 보정 후 수용 가능**으로 갱신한다.
이 판정은 원 contributor head를 직접 merge하는 승인이 아니다. 실제 통합 PR 생성 전 최신 devel/source
head와 원본 TSV 제출 조건을 확인하고, 정확한 통합 PR head CI가 녹색일 때 같은 PR의 docs-only trailing
commit으로 실제 CI·오늘할일을 추가한다. trailing head CI 확인 전 merge하지 않는다.
Docker 없는 host `--no-opt` fresh WASM이며 Docker 최적화 배포 검증·Apple SDK·C# runtime은 미검증이다.
부모 #536 전체나 낮은 별도 극한 진단을 해결·종료로 표시하지 않는다.

## 후속 base 변경

이 단계의 통과는 위 SHA 기준이다. PR 직전 #7953이 병합되어 리베이스했으며 현재 검증 상태는 [7단계](task_m100_536_integration_review_stage7.md)를 우선한다.
