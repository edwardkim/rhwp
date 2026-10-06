---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7210_review.md
last_verified: 2026-09-17
---

# PR #7210 검토

## 최종 판정

**승인** — 문서 교체 완료 이벤트에서 resize bbox cache·실패 cache를 함께 비우는 변경을 수용한다.

[원 PR #7210](https://github.com/edwardkim/rhwp/pull/7210): 수정: 표 크기 조절 캐시 정리를 문서 열기 공통 깔때기로 옮긴다 (#7194)
관련 [이슈 #7194](https://github.com/edwardkim/rhwp/issues/7194).
이 판정은 아래 변경 범위의 로컬 검토 결과이며 GitHub APPROVE 제출·원격 merge와 구분한다.

## Head·통합 계보·CI

- 원 head `62f0ea5a7d26993b4d411c29b77b4ba9e94499f7`, base `devel`. 검토자는 `jangster77`이다.
- source `62f0ea5a7d26993b4d411c29b77b4ba9e94499f7` → applied `4a679095e76eab47ac0364f90267821794cfff31`
- 통합 branch `codex/planet-review-20260917`, code head `cd074a4da`, fixture head `6600d48b2`.
- 확인한 성공 check/workflow: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35146277343/job/104963072860), [CI](https://github.com/edwardkim/rhwp/actions/runs/35146277267/job/104963074723), [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/35146276568/job/104963070372), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35146277176/job/104963071705), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35146277257/job/104963072383), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35146276822/job/104963070637), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35147113505). SKIPPED job은 검사 성공으로 계산하지 않는다.
- [공통 실행·전체 계보](pr_7210_review.md#통합-검토-공통-실행-기록), [처리 계획](../pr_7210_review_impl.md).

## 코드 경로와 독립 실행 증거

main.ts의 문서 교체 성공 → document-swapped → InputHandler의 cachedTableRef/cachedCellBboxes/tableBboxFetchFailures 초기화. 기존 open-document-bytes 진입에만 의존하지 않는다.

3147199를 파일 입력으로 연 뒤 bbox에 cellIdx=999와 실패 sentinel을 심고 3026219를 열었다. document-swapped 2회 / open-document-bytes 0회였으며 최종 ref=null, bboxes=null, failures=[]를 확인했다. source 문자열 검사의 통과만으로 이 판정을 내리지 않았다.

관련 실행: **Studio 전체 검사 중 document-swapped 관련 source guard 및 실제 파일 입력 전환**. Rust 전체 focused 34개 / Studio 1755개 통과.
원 PR의 수정 전 FAIL 기록은 작성자 증거이며 이번 reviewer가 소스 rollback으로 재실행한 것으로 세지 않는다.
reviewer가 비교한 base는 공통 기록의 실제 Native binary다.

## 남은 차이·보류 해제 또는 merge 전 조건

이 PR은 문서 렌더링 좌표를 고치지 않는다. #7214의 중첩 표 제한값 결함은 별도 PR의 보류 사유다.

최종 통합 head에서 CI를 확인하고, 통합의 다른 보류 사유가 해결된 뒤 merge한다.

## 공통 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일 ID 분기 없이 문서/셀/줄 속성을 사용한다. 적용 범위와 비적용 대조군을 위에 구분했다. |
| 측정·배치 일관성 | 비해당 | 문서 레이아웃을 바꾸지 않는 이벤트/가이드 변경이다. |
| 분할·이어받기 계약 | 비해당 | 이 PR은 분할 컷·continuation 소유 규칙을 바꾸지 않는다. |
| 줄 소속과 점유 높이 | 비해당 | 실제 줄/그림/표의 대상 의미와 검사 범위는 위 실행 증거 참조. 해당하지 않는 편집 UI에 조판 사례 전수를 요구하지 않는다. |
| 사례와 증거의 독립성 | 충족 | 공개 원문과 별도 한컴 PDF, actual Studio 입력 또는 정상 대조군 사용. 잔차를 숨기지 않았다. |
| 기준값 변경 | 비해당 | baseline/golden을 재생성하지 않았으며 실패를 허용치 증가로 해소하지 않았다. |
| 주장과 검증 범위 | 충족 | 실행 검출 결함, 코드상 우려, 미검증, 기존 차이를 구분했다. 전체 회귀·원격 CI 완료를 주장하지 않는다. |

실제 전환에 사용한 원문/PDF는 [fixture README](../../../tests/fixtures/planet_review_20260917/README.md)에 있다. Studio screenshot은 [#7211](pr_7211_review.md)과 [#7214](pr_7214_review.md)에 공유한다.

## 메인터너 보정 최종 검증

렌더링/UI 검증 코드 head **`54c24ebddb1a578786a6eb082c40c493dcde07f1`**,
테스트 lint 보정 head `f94dece59`, branch
`codex/planet-review-20260917`, base `fcbd00e0fabc4b309a887357033f92e2d511cd75`.
원 source 12개를 유지한 채 아래 4개 보정을 누적했다. 원 PR CI의 성공과 이 로컬 누적 head의
검증은 별개이며 아직 통합 PR·원격 CI·GitHub APPROVE·merge를 수행하지 않았다.

| 보정 commit | 해결한 보류 사유 | 확인한 최종 계약 |
| --- | --- | --- |
| `c1c9e2047` | #7214 중첩 표 제한값이 바깥 표를 조회 | 동일 CellPath 조회/resize, 바깥 표 불변, Undo 원복 |
| `2a9810642` | #7221·#7228 원점/예약/paint 불일치 | 첫 조각의 저장 원점과 cut 높이를 예약·실제 배치가 공유 |
| `95eed7197` | 위 보정에서 드러난 12쪽 마지막 줄 이월 | 선언 하단이 마지막 줄간격 중간이면 초과분만 제외 |
| `54c24ebdd` | #7225 실제 3→4쪽 회귀 | 음수 Percent 전진, 개체 여백, 소유 줄 수용 높이와 원본 vpos 보존 |

#7215는 #7221·#7228 보정을 포함한 동일 head에서 era 신호·HWP3 대조·실물 20쪽을 다시 검증했다.
다섯 보류 항목의 수용 여부는 개별 문서 상단을 따른다. 기존 조판 결함 전부를 해결하거나
관련 이슈 전체를 종료할 근거로 확대하지 않는다.

### 최종 로컬 검사

실행 환경은 macOS, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`,
전용 target `target/planet-review-20260917`이다. `f94dece59`에서 새 테스트의 마지막 표 탐색을 `filter().last()`에서 동등한 `rfind()`로 바꾼 lint 보정은
렌더링/UI 코드에 영향을 주지 않는다. 이 보정 후 fmt·workspace all-target Clippy와 해당
#7196 case 2개가 다시 통과했으며, 전체 회귀·Skia·fresh WASM·Visual Sweep은 위 렌더링 head의 결과다.
원시 로그는
`/private/tmp/rhwp-planet-review-20260917`에 보관하며 commit에는 넣지 않는다.

| 검사 | 실제 결과 |
| --- | --- |
| 최종 focused | 13 case / 41 PASS, 실패 0 |
| 전체 nextest | 9,974 PASS / 51 skipped, exit 0; 빌드 포함 679.3초 |
| Native Skia lib | 4,112 PASS / 13 ignored, exit 0 |
| Native Skia 그림·직접 PDF | 각각 2 PASS / 4 PASS, 모두 exit 0 |
| fmt·native/WASM/workspace Clippy·workspace build | 모두 exit 0; workspace Clippy의 새 테스트 `filter().last()` 경고는 `rfind()`로 정리 후 재검사 통과 |
| suite manifest·unit tier 정책 | base `fcbd00e0f` 대비 모두 exit 0 |
| Studio TypeScript·전체 test | 1,758 PASS / 2 skipped, tsc exit 0; UI 코드 `c1c9e2047` 이후 변경 없음 |
| fresh WASM | 최종 head 빌드 exit 0, 4분10초 |
| 실제 Studio 재검증 | 최종 fresh WASM에서 3026219 drag 시작, path 조회 12 / flat 조회 0, 바깥 표 불변·Undo 원복·page error 0 |

재실행 명령은 저장소 루트 기준이다. Cargo 검사는 동시에 실행하지 않았다.

```bash
export DEVELOPER_DIR=/Library/Developer/CommandLineTools
cargo nextest run --locked --cargo-profile release-test --target-dir target/planet-review-20260917 --tests --no-fail-fast
cargo test --locked --profile release-test --target-dir target/planet-review-20260917 --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/planet-review-20260917 --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/planet-review-20260917 --features native-skia
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/planet-review-20260917 -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/planet-review-20260917 -- -D warnings
cargo build --locked --workspace --target-dir target/planet-review-20260917
cargo clippy --locked --workspace --all-targets --target-dir target/planet-review-20260917 -- -D warnings
CARGO_TARGET_DIR=target/planet-review-20260917 scripts/wasm-pack-locked.sh --target web --out-dir /private/tmp/rhwp-planet-review-20260917/final-wasm
```

테스트 lint 재검사에서 도우미의 재계산 suite와 기존 생성 wrapper가 달라 최초 실행은
0개/exit 4였다. 이 결과를 성공으로 세지 않고 실제 `regression_suite_005`에서
`-E 'test(/issue_7196_page_top_spacing_trim_restore::/)'`로 실행해 2 PASS/exit 0을 확인했다.

원 sample 8개 보안 검사는 초기 실행 범위 그대로다. 전체 nextest의 환경변수 없는 성공을 새
sample 검사로 세지 않는다. 전달한 11경로 중 `tests/fixtures/` 3개는 검사 필터에서 제외됐음을
초기 기록에도 정정했다. 이번 보정에서 HWP/HWPX를 추가·수정하지 않았다.

### 최종 Visual Sweep

각 backend **15입력·28선택쪽**을 DPI 96으로 새로 캡처했다. 비교 번호가 다른 #7196은 아래
내용 대응 overlay를 별도로 생성했다. 최종 PNG 중 Native/WASM **22/28 byte 동일**,
6쪽(balance 4·5, hwpctl 52·57, trim 10, trim_counter 3)은 raster 차이가 남는다.
render-tree는 27/28 동일하며 속기록 20쪽의 4개 머리말/꼬리말 `pi`만 플랫폼 `usize::MAX`
표현(64bit/32bit)이 다르다. 그 sentinel을 구분하면 28쪽 모두 좌표·내용이 같다.
이를 PDF 일치율로 해석하지 않는다.

명령 형식:

```bash
venv/bin/python scripts/visual_sweep.py \
  --file-target <label> <input> <reference-pdf> \
  --rhwp-bin target/planet-review-20260917/release-test/rhwp \
  --pages <pages> --dpi 96 --out <output>
# WASM은 위 명령에 다음 인자를 추가
# --wasm-pkg /private/tmp/rhwp-planet-review-20260917/final-wasm
```

입력별 정확한 경로는 아래 표와 각 개별 review의 링크를 따른다. 선택 쪽은 전체 문서 검토를
뜻하지 않는다. compare·standalone overlay·review를 산출하고 영향 경계를 직접 판독했다.

| 입력 식별자 | 선택 쪽 / 기준 PDF | 직접 판독한 개선·대조 |
| --- | --- | --- |
| 2983289·3184393 그림 여백 | 각 1 / issue7193 기존 PDF | 기존 그림 여백 경로 보존 |
| 3011411·36473713 제어문자 | 각 1 / issue7190 기존 PDF | TAC 및 앞 제어문자 줄 소속 보존 |
| 156403546·156451317 | 각 1 / issue7198 기존 PDF | 음수/양수 후속 host 대조 |
| hwpctl_API_v2.4 | 12·13·26·52·53·55·56·57 / 기존 Hancom2020 PDF | 첫 조각과 이어받기, 마지막 코드 줄·후속 표 보존, 105쪽 유지 |
| 148733091 속기록 | 20 / 기존 PDF | era 신호 제한 후 HWP3 대조 유지 |
| 156760012 #7196 | 8–10 생성; **rhwp 9↔PDF 8, rhwp 10↔PDF 9**로 별도 비교 | 앞쪽 감사 문구·다음 붙임3 시작 보존; 전체 11/10쪽 차이는 남음 |
| 156676190 반례 | 1–3 / `pdf/planet-review-20260917/156676190-2020.pdf` | 4→3쪽, 첫 본문371.8px·사진표811px·2쪽 첫 본문114.47px; 3쪽 사진 배율 잔차는 #7225 문서에 원인 기록 |
| 3026219 중첩 표 | 1 / `pdf/planet-review-20260917/3026219-2020.pdf` | 실제 Chrome resize/Undo와 외부 표 불변 |
| 3147199 부분 테두리 | 1 / `pdf/planet-review-20260917/3147199-2020.pdf` | hover 대조 |
| 2025 행정업무운영 편람 | 130 / 기존 PDF | 그림 네 방향 여백 대조 |
| 36395325 결재문서 | 4 / 기존 `pdf/task2243/` PDF | 양수 저장 줄 간격 대조, 5쪽 유지 |
| worklife_balance_index_156607916 | 4–6 / 추가 Hancom2020 PDF | 셀 여백·표 후속 흐름 대조, 6쪽 유지 |

PDF와 남는 글꼴 폭/굵기·기호·기존 표/그림 크기 차이는 유지해 기록했다. 특히 156676190
3쪽 사진은 기존 셀 그림의 `pic_w.min(inner_area.width)` 축소 경로 때문에 선언 폭보다 작다.
원문 선언/PDF 그림 폭을 대조해 원인을 확인했으며 이번 페이지 회귀 해결을 그 차이의 해결로
바꾸어 보고하지 않는다. 목표 경계는 페이지 수뿐 아니라 실제 표 상자·마지막 줄·후속 내용으로
판정했다. 중간 보정에서 발생한 hwpctl 12쪽 줄 이월과 양수 간격 대조군 5→6/6→7쪽 회귀를
검출한 뒤 수정·철회하고 최종 head를 다시 캡처했다.

추가 증거 **62 PNG**는 #7214·#7215·#7225·#7228의 `maintainer_` 경로로 보존하며 #7221은
#7228과 같은 컷 증거를 공유한다. 원래 85 PNG는 수정 전 이력으로 보존한다.
기존 Git HWP/HWPX를 이름 바꿔 중복 추가하지 않았다. 추가 PDF 1개는
[fixture 출처 기록](../../../tests/fixtures/planet_review_20260917/README.md)에 저장 제품·변환
engine·실제 빌드·SHA256을 남겼다.


## 통합 검토 공통 실행 기록

- 기준: `upstream/devel=fcbd00e0fabc4b309a887357033f92e2d511cd75`, local devel 동기화 후 `codex/planet-review-20260917` 생성.
- 통합 code head `cd074a4da`, 원문/PDF 추가 head `6600d48b2`. 후자는 입력·증거만 추가했고 코드는 같다.
- 검토 당시 planet6897의 열린 non-draft 10개. 각 PR에 reviewer `jangster77` 지정. 충돌 없이 12개 고유 commit을 `cherry-pick -x`했다.
- #7228/#7215에 중복된 #7221 rebased source `6722f274747d789dafa37d4466986bf541b1c479`은 원 #7221과 stable patch-id `8fefd6c66fe5195e355e6520b702f167c8b5d164`가 같아 한 번만 적용했다. #7228도 #7215에서 중복 적용하지 않았다.
- 2026-09-17 16:07 KST 재조회: 원 PR 10개 모두 OPEN/non-draft, intake head와 동일. check rollup은 SUCCESS/SKIPPED이며 StatusContext도 SUCCESS. skip은 실행 통과로 세지 않는다. 누적 local tree의 CI는 아직 실행하지 않았다.
- 전용 target `target/planet-review-20260917`, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`. 다른 target은 삭제/변경하지 않았다.
- Native release-test build exit 0(2m24s), fresh WASM build+wasm-opt exit 0(3m53s). fresh WASM을 Studio와 Visual Sweep에 명시적으로 연결했다.
- `rhwp-studio`: `npx tsc --noEmit` exit 0, `npm test` **1755 passed / 2 skipped / 0 failed**.
- Rust focused: **34 passed / 1797 filtered**, 10개 case를 9개 생성 suite에서 선택. 이는 전체 회귀 통과가 아니다.
- `fidelity_compare --text-only --export-all-svg --layout-ledger`: 13입력 exit 0. CLI는 0-based, Visual Sweep은 1-based를 사용했다. 처음 잘못 지정한 fidelity 쪽 범위 결과는 폐기하고 재실행했다. exit 0을 PDF 일치 판정으로 바꾸지 않았다.
- Native/fresh WASM Visual Sweep 각각 13입력·21선택쪽 완료. 목표 trim은 rhwp10↔PDF8 / rhwp11↔PDF9의 의미 대응을 별도 2쪽 추가했다. 원래 trim 8–10 동일 번호 비교는 목표 개선 판정에 쓰지 않았다.
- 21개 Native/WASM PNG 중 19개 byte-identical. hwpctl p52/p57은 raster 차이가 있고 render-tree JSON은 완전히 동일했다. 두 backend의 review/overlay를 모두 보존하고 완전 동일이라고 보고하지 않는다.
- base Native는 이전 검증 binary를 재사용했다. build source `6dd78f9e5`와 `fcbd00e0f`의 src/crates/Cargo.toml/Cargo.lock diff가 비어 있음을 확인했다.
- 원문 3개는 기존 Git 전체 HWP/HWPX/PDF의 크기·SHA-256 중복 확인 후 원래 이름으로 추가했다. [입력 provenance](../../../tests/fixtures/planet_review_20260917/README.md)에 저장 제품·SHA·MCP engine·실제 버전·PDF SHA를 기록했다.
- 세 PDF는 MCP `start → status → download`, engine2020 / Hancom11.0.0.9136 / preprocess none으로 생성했다. PDF 1.4 등의 컨테이너 버전으로 배제하지 않는다. 기준 PDF의 페이지 수는 PDF 전체 기준이며 sweep summary의 선택 raster 수와 다르다.
- `cargo fmt --all -- --check`, suite manifest 정책, unit-test tiers 정책: exit 0.
- 추가/변경 원문 11개 경로를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 전달한 security corpus 3종 탐지 검사: 1 test PASS. 다만 이 검사의 경로 필터는 `samples/`만 허용하므로 실제 보안 검사 대상은 원 PR의 8개다. `tests/fixtures/`의 reviewer 입력 3개는 제외되었으며 보안 검사 완료로 계산하지 않는다. 해당 3개는 별도의 실제 파싱·시각 검증에 사용했다. 첫 실행의 잘못된 test target 지정은 실행 증거에서 제외하고 실제 생성 suite `regression_suite_027`로 재실행했다.
- 원문/PDF/대표 compare·standalone overlay·review PNG만 보존한다. 실행 raw는 `/private/tmp/rhwp-planet-review-20260917`; log/tsv/json은 commit하지 않는다.
- 전체 nextest, Native Skia 전체, Clippy bundle은 누적 tree에서 재실행하지 않았다. 실행 검출/계약 blocker를 해결하기 전 통합 전체 검증 완료로 보고하지 않는다. 원 PR CI의 성공도 누적 tree 승인 대신 쓰지 않는다.

### 재현 명령

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
DEVELOPER_DIR=/Library/Developer/CommandLineTools CARGO_TARGET_DIR=target/planet-review-20260917 \
  node scripts/run-rust-test.mjs <아래_case> -- --cargo-profile release-test
RHWP_BIN=target/planet-review-20260917/release-test/rhwp venv/bin/python \
  tools/fidelity_compare/fidelity_compare.py --source <원문> --reference-pdf <PDF> \
  --label <key> --out-dir <raw> --text-only --export-all-svg --layout-ledger <start0> <end0>
venv/bin/python scripts/visual_sweep.py --file-target <key> <원문> <PDF> \
  --rhwp-bin target/planet-review-20260917/release-test/rhwp --pages <1-based> --dpi 96 --out <raw>
# fresh WASM: 위 sweep 명령에 --wasm-pkg /private/tmp/rhwp-planet-review-20260917/wasm-pkg 추가
```

case: `issue_7189_nested_table_resize_by_path`, `issue_7035_hwp3_tolerance_needs_era_signal`,
`issue_7193_picture_inner_margin`, `issue_7190_hwpx_lineseg_axis_evidence`,
`issue_7203_split_float_anchors_to_paragraph_top`, `issue_7198_negative_spacing_host_after_float_table`,
`issue_7196_page_top_spacing_trim_restore`, `issue_7203_stored_rewind_fragment_trailing_trim`,
`issue_5961_hwpx_lineseg_axis_projection`, `issue_6368_row_cut_fp_epsilon`.

### Source 적용 계보

| 원 PR | source SHA | 로컬 적용 SHA |
| --- | --- | --- |
| #7210 | `62f0ea5a7d26993b4d411c29b77b4ba9e94499f7` | `4a679095e76eab47ac0364f90267821794cfff31` |
| #7211 | `55ad9f1b79130d075c6710bac2c2014e5443f239` | `08e8b617208bdfa90c5754c76c5f31f161fcc62d` |
| #7214 | `fc55c1a138500007cabeffff3d1146621ca45161` | `542f2442036a71b1033deb9f78b94fd001f572d1` |
| #7217 | `0ce544f913dfd8671a7167389ac3b5ac77aadfd2` | `09204fd03700d4d5190dd7168bc53581f628e550` |
| #7220 | `06ceed2db48d56b53d63f1b64a7796a87cc1b9eb` | `b472bf515a5339068baac41e6d73c920646e74c3` |
| #7220 | `82225af0edf8204d3fa78aeccee7e648f816682e` | `9d94231187d11d1ccd5b495d00bf8155eb11a26b` |
| #7221 | `3ac1356dc46fab66994f55bba598f599da9c5e2e` | `502dfe7d9e8b951e53e01d7322741a100e2e06ec` |
| #7223 | `92810d4b5ebc3bb5191414339b710bb1d35bc4b2` | `dd8db6bbb453825e2197cdc09970ea856190e2e5` |
| #7223 | `9744cfb806e04e77be82891b2535ef78636ae1a5` | `f1cf9a4f5f3dc67705405222c018e3c678537500` |
| #7225 | `cf0c06f846ba37e6a96aba80a48cdc817b5b8894` | `003dabe99e7e57e365c5b8d4a9544a115906a778` |
| #7228 | `a954fc95b769ae30d3aa631a1ba153d7ba28d2d6` | `ca404c82746dc70dd67c387fc74bdb7ec42aec87` |
| #7215 | `7d052733c3818914c80819d9767733fc803ff39d` | `cd074a4da5e5afc38c8894d9e88a5d39490b3537` |

## Merge 후 contributor PR comment 계획

실제 최종 head CI와 통합 merge가 완료된 뒤 원 source PR에 한국어로 적용 commit·통합 PR·merge SHA·CI URL과 감사 인사를 남긴다.
이번 review는 아직 remote push/통합 PR/merge 단계가 아니다. 보류가 남으면 완료·이슈 종료 댓글을 게시하지 않는다.
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고,
대표 compare/review뿐 아니라 위 **standalone overlay**도 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/...`로 본문에 직접 포함한다.
실제로 확인한 쪽·backend·개선 범위와 기존 차이를 함께 적는다. 다쪽 경계는 앞/뒤 쪽을 모두 포함하며 #7225는 156676190의 1–3쪽 및 추가 4쪽 해소 여부를 숨기지 않는다.
UTF-8 body 파일과 `--body-file`로 게시하고 한국어·이미지 URL·실제 head를 다시 확인한다. 관련 이슈의 남은 범위가 있으면 열린 상태를 유지한다.
