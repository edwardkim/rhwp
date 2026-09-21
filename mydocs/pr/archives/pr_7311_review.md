---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-21
---

# PR #7311 검토 — Top 앵커 비끝 표 조각의 쪽 프레임 복원

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 contributor code head `d1d2118f`는 Center 반례를
PR에 포함하지 않아 적용 경계와 검증 입력 커밋 확인이 미충족이었다. collaborator가 현재
원 PR branch 위에 `cb107e299`를 별도 추가해 실제 HWP·한컴 2020 PDF와 Center 음성 대조군을
고정했다. 이 문서는 그 보정이 포함된 head `cb107e299`를 검토 대상으로 한다.

이 판정은 GitHub APPROVE·push·comment·merge를 뜻하지 않는다. 이 기록은 로컬 trailing commit으로만
준비했으며, push는 작업지시자의 별도 승인 뒤에만 수행한다. merge 전에는 push된 최신 head의
review-only CI, `MERGEABLE`/`CLEAN`, reviewer 지정 및 작업지시자의 merge 승인을 다시 확인한다.

## 접수와 경로

| 항목 | 값 |
| --- | --- |
| PR | [#7311](https://github.com/edwardkim/rhwp/pull/7311) |
| 작성자 | `planet6897` (기존 외부 contributor) |
| base / source branch | `devel` / `work/6923-wrapper-frame` |
| 원 contributor head | `d1d2118f01beddec3539b5c2e595529b89c7e38d` |
| 보정 head | `cb107e299ecc61438c66f3b0fc0755f52980be22` |
| 보정 소유자 | `jangster77`, `test(layout): Center 조각 표 반례를 fixture로 고정` |
| 원격 상태 참고값 | Open, non-draft, `MERGEABLE` / `CLEAN`, `maintainerCanModify=true` |
| reviewer | 아직 원격 지정하지 않음. 이 문서는 push 승인 전 로컬 기록이며, merge 전에 별도 확인 필요 |
| 관련 이슈 | [#6923](https://github.com/edwardkim/rhwp/issues/6923) — 원 PR 설명대로 부분 개선이며 이 review가 이슈 상태를 바꾸지 않음 |

- base route: `collaborator_external_pr.md`의 contributor head 직접 보정 경로.
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `review_only_fast_pass.md`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서,
  `visual_verification_governance.md`, `visual_sweep_guide.md`.
- 로컬 가시성 branch `maintainer/pr7311-evidence`는 원 contributor head 위에서 시작했고,
  contributor commit을 재작성하지 않았다. 보정 commit과 이 trailing 기록만 그 뒤에 추가한다.

## 변경과 보정 범위

원 변경은 `src/renderer/layout/table_partial.rs`의 단일 행 RowBreak 비끝 조각에서,
쪽 상단 시작 여부만 보던 조건을 **쪽 상단 시작 또는 첫 칸 `VerticalAlign::Top`**으로 넓힌다.
Top 앵커 칸은 상자를 늘려도 내용이 상단 안여백에 고정되므로, 표 상자는 내용 끝이 아니라
쪽 프레임에서 끝나야 한다. `projected_content`와 저장 reset paint geometry는 기존 제외 조건을
유지한다. Center·Bottom 칸을 문서 ID·좌표 clamp로 예외 처리하지 않는다.

원 head는 Center 비적용 경계를 PR 설명으로만 인용했다. `cb107e299`는 다음을 추가했다.

- `tests/fixtures/issue6923/156645214_240812(조간)_4개_아이돌굿즈_판매사업자_전상법의_위반행위_제재.hwp`
- 같은 입력의 한컴 2020 기준 PDF
- 19쪽 `para_index=204` 1×1 Center 표가 쪽 프레임 약 1023px로 확장되지 않고,
  PDF의 `217.3..1013.5px` 범위에 남는지 검사하는 음성 대조군

따라서 Top 적용 경로와 Center 비적용 경로가 모두 실제 commit 입력으로 고정된다.

## 공통 조판 원칙 심사

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 특정 파일명이 아닌 단일 행 비끝 조각·Top 수직 앵커·투영/저장-reset 제외라는 조판 조건을 사용한다. |
| 측정·배치 공통 결과 | 충족 | `single_cell_page_fragment_bottom`이 정한 같은 쪽 프레임 높이를 행 높이와 cell clip/border 배치가 소비한다. |
| 분할·이어받기 계약 | 충족 | Top 1쪽의 wrapper 하단과 Center 19쪽의 비확장 경계를 실제 render tree 좌표로 함께 검사한다. |
| 줄 소속과 점유 높이 | 충족 | Top 칸의 첫 줄이 표 상단 근처에 남아 상자 확대가 내용 위치를 바꾸지 않는 계약을 검사한다. |
| 사례와 증거의 독립성 | 충족 | 한컴 PDF의 물리 가로선/표 경계와 render tree 좌표를 대조하고, 반대 `Center` 수직 앵커 실물 문서를 제어군으로 사용한다. |
| 기준값 변경 | 비해당 | baseline·golden·허용치를 바꾸지 않는다. |
| 주장과 검증 범위 | 충족 | source/test 보정 뒤 로컬 lint·focused·Native/WASM Visual Sweep과 최종 head CI를 구분해 기록한다. |
| 검증 입력 커밋 | 충족 | 아래 HWP/PDF는 검증한 그대로 현재 PR 보정 head에 포함됐으며 개인 다운로드 경로만의 입력이 아니다. |

## 검증 입력과 기준 PDF

| 입력 | 역할 | SHA-256 |
| --- | --- | --- |
| [148738070 wrapper HWP](../../../tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp) | Top 앵커 적용 경로 | `41f8f0349840a72476606a45843caf01b12013b5e536fca5f6214ff76872ceb3` |
| [148738070 한컴 2020 PDF](../../../tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame-2020.pdf) | Top p1·p2 기준 | `0dde093557a0a11cec2f01af94f3a8dcf5004c30d267f6f5ea3fa5ac65f9b4bf` |
| [156645214 Center HWP](<../../../tests/fixtures/issue6923/156645214_240812(조간)_4개_아이돌굿즈_판매사업자_전상법의_위반행위_제재.hwp>) | Center 비적용 제어군 | `cb36c862d3d41757d15fb322b1733f0e0ce27bf20142db0a25c57a624739e23b` |
| [156645214 한컴 2020 PDF](<../../../tests/fixtures/issue6923/156645214_240812(조간)_4개_아이돌굿즈_판매사업자_전상법의_위반행위_제재-2020.pdf>) | Center p19 기준 | `71261bcb218eabc6f1b2af5ce81797a3505f417d1c2482ee5441dec11b6ba943` |

Center PDF는 HWP `lastSavedWith`에 맞춰 HWP 2020 engine으로 새로 변환했고, 실제 한컴 server는
`11.0.0.9136`이었다. PDF는 20쪽이며 검증한 p19와 commit blob의 SHA-256이 일치한다.

## 로컬 검증

보정 head에서 `CARGO_BUILD_JOBS=8`, `CARGO_TARGET_DIR=target/pr7311-maintainer`로 다음 결과를 확인했다.

| 검증 | 실제 결과 |
| --- | --- |
| #6923 회귀 | `regression_suite_020`: 3 passed — Top wrapper 하단, Top 내용 중립, Center 음성 대조군 |
| #7095 기존 경계 | `regression_suite_015`: 6 passed |
| Rust quality gate | manifest prepare와 base `upstream/devel` 비교, fmt, native Clippy, WASM32 Clippy, workspace build, workspace all-target Clippy 통과 |
| Fresh Native | release `rhwp` build 성공 |
| Fresh WASM | `scripts/wasm-pack-locked.sh --target web --out-dir pkg-pr7311-maintainer` 성공 |
| 최종 head GitHub CI | [CI run 35577504780](https://github.com/edwardkim/rhwp/actions/runs/35577504780) 성공. Lint, Native Skia, Frontend package gates, archive A–D, Build & Test 및 Rust CodeQL 성공 |

최초 잘못 고른 generated suite는 0 tests를 실행해 검증 결과로 세지 않았다. 이후 source branch의
정확한 `regression_suite_020`을 다시 실행한 결과만 위 표에 기록했다.

## Visual Sweep 직접 판독

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라
최종 보정 head에서 96dpi로 Top p1·p2와 Center p19를 Native와 fresh WASM으로 각각 새로 캡처했다.
각 sweep은 complete, 자동 구조 flag 0건이고 Native/WASM의 같은 페이지 render tree JSON은 바이트 단위로 같다.

| 경로·쪽 | backend | pixel match | ink / visual proxy | 직접 판독 |
| --- | --- | ---: | ---: | --- |
| Top p1 | Native / WASM | 86.67612% | 37.89228% | wrapper 하단 프레임과 표 경계가 PDF와 같은 물리 영역에 있으며 구조 flag 없음 |
| Top p2 | Native / WASM | 89.77438% | 26.66570% | 이어받은 표·본문 흐름과 경계가 유지되며 구조 flag 없음 |
| Center p19 | Native | 87.67437% | 15.61372% | 1×1 Center 제어 표가 PDF의 중간 끝점에 남고 쪽 프레임으로 확장되지 않음 |
| Center p19 | fresh WASM | 87.67919% | 15.65710% | Native와 같은 표 경계·흐름, 구조 flag 없음 |

pixel match와 visual proxy는 자동 보조 지표이며 사람의 최종 판정을 대체하지 않는다. Top p1·p2의
글리프·잉크 차이와 Center p19의 낮은 ink proxy는 남아 있으나, 직접 본 review/overlay에서는 이번 변경이
다루는 표 조각의 물리 프레임·경계에 구조적 이탈이나 새 겹침이 보이지 않았다. 따라서 전체 PDF fidelity를
승인하는 근거로 확대하지 않는다.

임시 원본 출력은 `output/pr7311-maintainer/{top,center}-{native,wasm}`에 남아 있다. 최종 comment에
직접 사용할 review와 standalone overlay만 아래 안정 경로로 보관하고, raw raster·compare·contact sheet·SVG·JSON·실행 로그는 commit에서 제외한다.

### 보관한 대표 증적

- Top p1: [Native review](../assets/pr7311_review/top_native_review_001.png), [Native overlay](../assets/pr7311_review/top_native_overlay_001.png), [WASM review](../assets/pr7311_review/top_wasm_review_001.png), [WASM overlay](../assets/pr7311_review/top_wasm_overlay_001.png)
- Top p2: [Native review](../assets/pr7311_review/top_native_review_002.png), [Native overlay](../assets/pr7311_review/top_native_overlay_002.png), [WASM review](../assets/pr7311_review/top_wasm_review_002.png), [WASM overlay](../assets/pr7311_review/top_wasm_overlay_002.png)
- Center p19: [Native review](../assets/pr7311_review/center_native_review_019.png), [Native overlay](../assets/pr7311_review/center_native_overlay_019.png), [WASM review](../assets/pr7311_review/center_wasm_review_019.png), [WASM overlay](../assets/pr7311_review/center_wasm_overlay_019.png)

## Merge 후 contributor PR comment 계획

실제 merge가 승인·완료된 뒤에만 merge SHA와 CI URL을 확인해 한국어로 감사와 함께 원 PR에 게시한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을
직접 연결하고, 위 3쪽 × Native/WASM의 review 6장과 standalone overlay 6장을 실제 merge commit SHA로
고정한 raw URL 이미지로 표시한다. 각 쪽의 flagged=0, pixel match, visual proxy가 자동 보조값이라는
설명과 표 프레임의 사람 판독 범위·남은 글리프/잉크 차이를 함께 남긴다.

형식은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7311_review/<filename>`을
쓴다. UTF-8 파일을 `--body-file`로 게시한 뒤 API로 한글·merge SHA·12개 이미지 URL을 재조회한다.
#6923은 이번 부분 개선으로 새로 close하지 않으며, PR 본문의 이슈 상태도 변경하지 않는다.
