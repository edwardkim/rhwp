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
- 추가/변경 원문 11개(원 PR 8개 + reviewer 입력 3개)를 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 실제 전달한 security corpus 3종 탐지 검사: 1 test PASS, 11입력 검사 완료. 첫 실행의 잘못된 test target 지정은 실행 증거에서 제외하고 실제 생성 suite `regression_suite_027`로 재실행했다.
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
