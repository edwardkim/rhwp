# #7946·#7855·#7856 3단계 — 최종 회귀와 통합 PR 준비

- Issue: [#7946](https://github.com/edwardkim/rhwp/issues/7946), [#7855](https://github.com/edwardkim/rhwp/issues/7855), [#7856](https://github.com/edwardkim/rhwp/issues/7856)
- 승인: 2026-10-16 사용자의 “다음 절차 진행을 승인합니다” — 최종 전체 회귀·통합 PR 준비
- 제품 소스: `46155219766953d8f4cb348ad96c1c8e97434354`; 기준 devel: `4662c069e557e928ed7968d061121fde90b1f027`
- 판정: 아래 로컬 PR 준비 조건 충족. 원격 push·PR 생성·GitHub CI·self-review·merge는 미실행이다.
- 증적: `output/pr-review/issue7946/pr-preparation/` (ignored). 구현·직접 Studio 판정은 [2단계](task_m100_7946_stage2.md)를 따른다.

## 1. 최종 검사 결과

검증 전후 fetch에서 devel은 같은 SHA였고 후보가 이를 이미 포함했다. 추가 source merge나
제품 보정은 필요하지 않았다. `merge-tree --write-tree upstream/devel HEAD`도 성공했다.
이번 단계는 아래 검사와 단계 문서·대표 PNG만 추가한다. 검증한 제품 source 18개 파일과
pkg/public JS·WASM의 SHA-256은 2단계의 `verification-state.json`과 모두 같다.
따라서 동일 후보의 Docker WASM·Rust lint·Studio 전체 검사를 의례적으로 반복하지 않았다.

| 검사 | 실제 결과·로그 |
| --- | --- |
| 기본 전체 integration·unit 회귀 | `cargo nextest run --locked --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --tests --test-threads 8 --no-fail-fast`: **10,435 PASS·0 FAIL·45 skip**, 실행 482.628초. `nextest-full.log` |
| Native Skia library | `native-skia,pdf-import`의 `cargo test --locked --profile release-test ... --lib`: **4,043 PASS·12 ignored**. root library 3,861 및 workspace library 15·165·2건의 합계. `native-skia-lib.log` |
| Native Skia 그림 누락·직접 PDF | 공식 `issue_2225_missing_picture_placeholder` **2 PASS**, `render_p37_direct_pdf_export` **4 PASS**. 해당 이름의 로그 |
| PDF 활성 집중 | detection 2·frontend 11·resource limits 111·CLI 5·explicit text margin 1: **130 PASS**. `native-matrix.json`, `base-native-steps.json` 및 각 case 로그 |
| 최종 VS Code package | compile/prepublish·VSIX packaging 성공. **55파일·19,858,335 bytes**, 포함한 `extension/dist/media/rhwp_bg.wasm`의 SHA는 후보 Docker WASM과 동일. `vscode-package-final.log` |
| 재사용한 2단계 검사 | fmt·Native/WASM/workspace Clippy·workspace build·manifest, Studio 2,021 PASS·2 skip·type/build·실제 UI 15항목, 실제 Worker 85 assertion·5 Worker, 의존성 소비/정책/바인딩. 실제 제품 코드와 검사 파일이 같음을 hash로 확인 |

Native Cargo 검사는 모두 공용 target에서 순차 실행했다. 16 논리 CPU·31 GiB 환경에서 Native
build jobs 8·test threads 8을 사용했다. 별도 Docker named target volume의 기준 WASM은 jobs 4·
codegen-units 16을 사용했다. 공유 target을 삭제하거나 이슈별 target을 추가하지 않았다.
nextest 0.9.137은 요구 최소 0.9.91을 충족하며 권장 0.9.140 경고가 있었다. 사용하지 않은 JUnit
`report-skipped` key 경고도 보존했으며 이를 실행 실패나 새 skip으로 세지 않았다.

## 2. 정확한 devel 대비 전쪽 시각 회귀 대조

과거 PR의 출력 대신 기준 SHA의 실제 Native와 Docker WASM producer를 준비했다. 기준 제품은
기본 작업공간의 `output/pr-review/issue7946/base-4662/`에, 후보 producer는 이 작업공간의
`pr-preparation/candidate/rhwp`와 기존 `pkg/`에 보존했다.

| producer | SHA-256 |
| --- | --- |
| devel Native (`release-test`, `native-skia,pdf-import`) | `aeafd29a6490cf2a0f0ab26ffc5e073410cb11620fbc6bc59e97a33c4dc4823c` |
| 후보 Native (동일 profile/features) | `d5654c04a8bd2cc7da04cec504408a62d6366beccbcafa35b79a0fe2eed24080` |
| devel Docker WASM (`release`, `pdf-import`, codegen 16) | `70b7e94ab67868131e9dc9d3274525c259f98c91d07fa631b17dc732d9243380` |
| 후보 Docker WASM (동일 profile/features/codegen) | `de814d24b0d56924259e7ac2e277bedf57d0dbd6d5a91e067b32b097423d2246` |

원본·독립 한컴 Print는 2단계의 동일 12쪽 PDF이며 Print·compat 2022·96 DPI·공급 글꼴
파일/해시·decoding·raster backend·실루엣 계산 조건과 전체 쪽수가 양쪽에서 같은지 확인했다.
TSV 원래 소수값을 `Decimal`로 비교해 기준−후보가 0.01%p 이상인 쪽을 찾았다.

| backend | 기준 / 후보 최저값 | 전체 쪽수 | 최대 하락 | 0.01%p 이상 하락·미달·누락 |
| --- | --- | --- | --- | --- |
| Native Skia PNG | 90.14609% / 90.14609% | 12 / 12 | **0.00000%p** | **0·0·0** |
| fresh WASM Canvas2D PNG | 90.16832% / 90.16832% | 12 / 12 | **0.00000%p** | **0·0·0** |

각 backend에서 전쪽 PNG 바이트도 기준/후보가 모두 동일하다. Native 비교용 `release-test`
후보의 12쪽 PNG·TSV는 2단계 `release` 후보 결과와도 전부 동일하므로 이미 직접 판독·사용자
통과한 대표 review/overlay를 재사용한다. 새로 캡처했다고 표시하지 않는다. 이 동일성은 해당
입력·조건·출력 경로의 관측이며 전체 엔진의 backend 간 픽셀 동등성 주장으로 넓히지 않는다.
Studio 실제 Canvas 판정과 Print Canvas2D TSV도 별도의 검증으로 유지한다.

- 전쪽 원본: `base-native-scores`, `head-native-scores`, `base-wasm-scores`, 기존 `../wasm-scores`
- source/build·입력·글꼴 조건: 각 `pdf/run_manifest.json`, 기준 producer의 `build-products.json`
- Decimal 대조 24개 backend/page 쌍: `devel-comparison.json`
- 대표 3·11쪽: `mydocs/pr/assets/issue_7946_studio_pdf/`의 Native/WASM review·standalone overlay 8개

## 3. 기준 Docker cache 재사용 오판의 분리

첫 기준 빌드는 compile 0.43초로 이전 `/app` 산출물을 재사용했다. 기준 `wasm_api.rs`에는
`supports_pdf_import`가 없는데 생성 JS에는 후보 전용 `supportsPdfImport()`가 남아 있었다.
JS hash도 후보와 같았다. 이 실행을 중단했고 기준 TSV·회귀 통과 근거로 쓰지 않았다.
실행 성공이나 pkg/public의 동일 hash만으로 source 일치를 증명하지 못한 실제 사례다.

기준 Rust 파일의 바이트 hash를 유지하면서 수정 시각을 갱신해 root package의 재컴파일을
유도했다. 별도 `fresh-pkg` 출력에서 실제 compile 3분 42초·전체 8분 32초와 후보 API 부재를
확인했다. 기본 public JS는 변경되지 않았다. `stale-cache-diagnosis.json`, `docker-build.log`
(제외한 실행), `docker-build-corrected.log`, `build-products.json`에 근거를 보존한다.
잘못된 cache 산출물을 기준으로 인정하거나 공유 cache를 일괄 삭제하지 않았다.
동일 문제의 확인 절차를 [개발 환경 안내](../manual/dev_environment_guide.md)에 보완했다.

## 4. 입력·조판 계약·제출 자료

실행한 공개 PDF와 HWP/HWPX/HML 대조군은 제품 commit `461552197`의 파일과 바이트가
같다. 경로/해시는 2단계와 `input-commit-check.json`에 연결한다. 새 fixture/golden·baseline·
허용치·레이아웃 px assertion은 추가하거나 변경하지 않았다. 해당 PDF 경로를 참조하는
기존 Rust 회귀/공용 corpus의 절대 위치 핀도 검색되지 않았다.

조판 값의 생산·측정·배치·분할 로직은 이번에 바뀌지 않았다. 실제 적용 경로는 2단계의
기존 PDF 생성자 → 공통 IR → 문서 교체 → Canvas이며, 원본 handle/출력 handle과 PDF 기능
가용 여부만 연결했다. 구조·배치 보정의 일반/특수 분할 계약은 비해당이다. 각 출력의 독립
PDF 비교와 실제 Studio 사용자 판정은 별도로 충족했다. 기존 B10-1 미달은 그대로 후속이다.

대표 PNG 8개는 source 산출물과 byte 대조했고 standalone overlay도 직접 열어 확인했다.
PR 본문은 `pr-body-ko.md`에 전쪽 Native/fresh WASM TSV 원문과 제출 head SHA로 고정한
실제 Markdown review/overlay 표를 포함한다. TSV·중간 JSON·로그·private font는 커밋하지 않는다.
원본 배포본의 LICENSE 끝 빈 줄·`dist/runtime/quote.js` 공백 2건은 1단계 원장과 동일하게
보존했다. 이 두 vendor 파일을 제외한 전체 authored diff의 whitespace는 정상이다.

## 5. 남은 원격 절차

로컬 본문·생성 명령은 `pr-preparation/pr-body-ko.md`, `registration-command.txt`다.
내부 타스크의 [PR 생성 별도 승인](../manual/codex/docs_and_git_workflow.md#internal-task-pr-approval)을
받은 후 upstream 작업 branch push·devel 대상 Open PR 생성·정확한 code head CI 확인 순서로
진행한다. PR 번호를 예측하거나 PR review/오늘할일을 미리 만들지 않았다.

번호와 CI 성공이 확인된 뒤에는 [self 경로](../manual/pr_review/collaborator_self_merge.md#821-pr-채번과-오늘할일-생성갱신-시점)의
문서-only trailing 기록과 후행 CI, 별도 승인된 merge·이슈 종료를 따른다. 기존 사용자
worktree·변경·공유 cache는 보존했고 이번 단계에서 GitHub 원격 변경은 수행하지 않았다.
