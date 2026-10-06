# PR #7317 리뷰 — 저장 줄 전진의 측정·조판 정합

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#7317](https://github.com/edwardkim/rhwp/pull/7317) |
| 작성자·self-review | `jangster77` collaborator self PR, reviewer 미지정 |
| 관련 이슈 | [#6656](https://github.com/edwardkim/rhwp/issues/6656) |
| base / code candidate | `devel` / `18d3c78f78ffccd58a333b42b3b63c4bc33c5ac9` |
| 문서 작성 시점 상태 | Open, non-draft, `MERGEABLE` / `CLEAN` (code head CI 성공 후 확인) |
| 규모 | 6 files, +158 / -25 (code candidate) |

## 구현과 호출 경로

문단 0.7의 저장 사다리는 `vpos 0→1600HU`인데 `lh + ls`는 2160HU였다. 기존 #6691은
`LayoutEngine`의 paint cursor에는 저장 사다리가 있었지만, `HeightMeasurer`의 fallback
measurement는 2160HU를 계속 소비했다.

`stored_line_flow_height`가 저장 `LINE_SEG`의 다음 좌표를 쓸 수 있는 조건을 한 곳에서 판정한다.
현재 줄 상자가 저장 `lh`와 같고 사다리가 증가하며, advance가 글자 또는 글자처럼 취급되는
개체의 최소 높이를 침범하지 않을 때만 저장값을 쓴다. Layout과 HeightMeasurer가 같은 결과를
소비한다. Pagination의 `TypesetEngine`은 줄 상자 예약을 유지한다. 저장 advance를 page budget에
확장한 초기 후보가 #1139 미주 경계와 body-overflow/off-canvas baseline을 회귀시킨 것을 CI로
재현해 원복했다.

## 조판 원칙 심사

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 실제 HWP 저장 사다리와 한컴 PDF 3쪽을 근거로 했으며 문서 ID 분기가 없다. |
| 측정·배치 일관성 | 충족 | 공통 `stored_line_flow_height`를 LayoutEngine·HeightMeasurer가 호출한다. |
| 분할·이어받기 계약 | 충족 | TypesetEngine의 page-budget 줄 상자 예약은 유지했고, #1139·overflow baseline 회귀 4개를 직접 재실행해 통과했다. |
| 줄 소속과 점유 높이 | 충족 | 줄 상자와 spacing은 보존하고 다음 줄 전진만 저장 좌표로 제한적으로 바꾼다. |
| 사례와 증거의 독립성 | 충족 | 실제 HWP는 수정 전 `28.8px` 대 저장값 `21.333px`으로 실패했고 수정 후 통과했다. |
| 기준값 변경 | 비해당 | golden·baseline·허용치를 변경하지 않았다. |
| 주장과 검증 범위 | 충족 | focused 계약, shard, lint, Native/fresh WASM Visual Sweep을 code candidate에서 실행했다. |

## 검증 입력과 결과

| 입력 | 역할 | SHA-256 | commit 포함 |
| --- | --- | --- | --- |
| `samples/hwpctl_ParameterSetID_Item_v1.2.hwp` | 실제 저장 사다리·회귀 입력 | `76645c03f0372529bad746e9287cb04efc86c6b1aa57963caae4a1c3c004c39e` | 기존 추적 파일 |
| `pdf/hwpctl_ParameterSetID_Item_v1.2-2022.pdf` | 한컴 기준 PDF 3쪽 | `b23d4836b9d1eb8dd9582e5541aeca34dfb54718d5829b21af4f7969085dd746` | 기존 추적 파일 |

- 수정 전 새 회귀 계약은 `measured=28.8px`, `stored=21.333px`로 실패했고 수정 후 통과했다.
- `cargo fmt --all -- --check`, native·WASM32·workspace all-target Clippy (`-D warnings`),
  `cargo build --locked --workspace`, manifest base 비교를 통과했다.
- `regression_suite_014` 210개를 통과했다. #1139 미주 경계, body-overflow partition 10·11, off-canvas partition 11도 수정 후 5/5 통과했다.
- code head `18d3c78f7`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/35591844316)는 Lint, Native Skia, test archive A–D, Build & Test를 성공했다.
- Native와 fresh WASM으로 각각 3쪽 Visual Sweep을 직접 실행·열어 확인했다. 양쪽 모두 구조 후보 0건,
  pixel match 95.45142%, visual accuracy proxy 81.36943%였다. 기존 glyph/그림 raster 잔차는 남지만
  이번 줄 전진 보정의 새 drift·overflow 후보는 없었다.

## Visual Sweep 증적

| backend | review | overlay |
| --- | --- | --- |
| Native | [3쪽 review](../assets/pr7317_review/native_review_003.png) | [3쪽 overlay](../assets/pr7317_review/native_overlay_003.png) |
| fresh WASM | [3쪽 review](../assets/pr7317_review/wasm_review_003.png) | [3쪽 overlay](../assets/pr7317_review/wasm_overlay_003.png) |

## 최종 판정

**승인.** 실제 HWP의 수정 전 실패와 수정 후 Layout·HeightMeasurer 정합을 확인했고,
pagination 회귀 4개도 재현·해소했다. 기준 PDF 3쪽의 Native/fresh WASM 증적도 직접 판독했다.
code head CI는 성공했으며, trailing head에 대한 문서 CI 성공과 `MERGEABLE`/`CLEAN` 재확인 및
작업지시자의 merge 승인이 남았다.

## Merge 후 contributor PR comment 계획

최종 merge SHA와 해당 head CI가 확정된 뒤에만 `--body-file`로 한국어 PR comment를 게시하고 API로
본문·이미지를 다시 확인한다. comment에는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment),
실제 merge SHA·CI URL, 3쪽 구조 후보 0건·pixel match 95.45142%·visual accuracy proxy 81.36943%와
이 수치가 전체 fidelity 판정이 아닌 보조값임을 적는다. merge commit에 존재하는 아래 네 이미지를 함께 넣는다.

- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7317_review/native_review_003.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7317_review/native_overlay_003.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7317_review/wasm_review_003.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7317_review/wasm_overlay_003.png`
