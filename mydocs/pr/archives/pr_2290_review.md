# PR #2290 검토 - #2287 표 밀집 문서 과소분할 보정

- 검토일: 2026-07-16
- 대상: [PR #2290](https://github.com/edwardkim/rhwp/pull/2290), [Issue #2287](https://github.com/edwardkim/rhwp/issues/2287)
- 작성자: `planet6897`
- base / head 참고값: `devel` / `86bf306f8f38d7a5a780e506e91cd5cc3bf31b81`
- 규모: 9 files, +498/-5, HWP 재현 샘플 1개 추가
- mergeable: `CLEAN` (검토 시점 참고값이며 merge 전 최신 상태 재확인 필요)
- 리뷰어: `jangster77` 지정 완료

## PR 주장과 범위

PR은 RowBreak rowspan 블록의 연속 조각에서 `start_cut` 이후 잔여 높이가 0으로 계산되어 표 내용이
증발하는 결함과, 저장 `LINE_SEG`가 없는 TAC 그림/도형 anchor 문단의 높이가 0으로 붕괴하는 결함을 함께
보정한다. 구현은 `typeset.rs`의 블록 조각 높이 하한, `table_layout.rs`의 spacer-trim 잔여 측정,
`typeset.rs`/`height_measurer.rs`의 TAC 객체 줄 메트릭 합성에 걸친 renderer 변경이다.

연관 이슈인 [#2237](https://github.com/edwardkim/rhwp/issues/2237)과
[#2279](https://github.com/edwardkim/rhwp/issues/2279)는 PR 본문에서도 별도 잔여 축으로 열어 둔다.

## 기준 자료와 시각 검증

- 원본: PR에 추가된 `samples/task2287/1342000_edu_curriculum_map.hwp`
  - SHA-256: `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e`
- 기준 PDF: HWP 2020 MCP job `44748576-36a7-457b-9e19-88c195bcd492` 변환 결과 415쪽, SHA-256
  `598b67d8f6b231c34b7849aae5cedcf69d4b42b15eba5d5a8fd6cd9d565bb0fe`.
  142MB 전체 PDF는 현재 저장소 Git LFS 쿼터 정책상 새 객체로 추가하지 않고, 이 PR의 판단 범위인
  25-35쪽을 손실 없이 분리해
  [`pdf/task2287/1342000_edu_curriculum_map-2020-p025-p035.pdf`](../../../pdf/task2287/1342000_edu_curriculum_map-2020-p025-p035.pdf)에
  보존했다. 분리본은 11쪽, 4,571,674 bytes, SHA-256
  `5f108f123b3ff4c1e5c6ca76072538a23e676e927a9b14fad799962de8c4cf7d`다.
- visual sweep: PR head는 381쪽, 검토 기준 `devel`은 375쪽으로 렌더했다. 기준 PDF는 415쪽이다.
  검토 범위는 p25-35이며, PR head에서 자동 후보는 p30 한 쪽이다.
- 임시 sweep 경로: PR head
  `/private/tmp/rhwp-pr2290-review/target/visual-sweep-pr2290/task2287-pr/`, 기준 `devel`
  `/private/tmp/rhwp-pr2290-base/target/visual-sweep-pr2290-base/task2287-base/`.
- 대표 증적: [p26 review PNG](../assets/pr_2290/task2287_p026_review.png),
  [p30 review PNG](../assets/pr_2290/task2287_p030_review.png),
  [visual sweep summary](../assets/pr_2290/task2287_visual_sweep_summary.json).

수정 전 `devel`의 p26은 기준 내용이 들어온 뒤 render tree tail이 페이지 frame 밖으로 약 2,253px
넘쳤다. PR head에서는 p26의 overflow 표식은 사라졌지만 기준 PDF에 있던 본문이 대부분 비어 보인다.
대신 p30의 render tree에는 `대구`, `없음` 등 표 내용이 frame 아래 약 2,032px부터 계속 이어지는 tail
overflow가 남는다. p30은 `render_tree_frame_tail_overflow`, `line_band_drift`,
`column_line_band_drift`, `large_ink_region_drift`로 flag 되었고, p26/p30의 pixel match는 각각
93.39402%/90.90810%, 내용 픽셀 중심 보조 지표는 각각 10.85568%/11.53632%다.

따라서 p26의 `ymax 3027 -> 741`만으로는 해결을 입증하지 못한다. 대표 표의 내용이 다음 페이지 흐름으로
정상 분할된 것이 아니라, p26에서 빠지고 p30의 page frame 밖으로 이동한 상태다.

## 검증

- merge simulation: `upstream/devel`과 PR head clean
- `git diff --check`: 통과
- `cargo fmt --check`: 통과
- focused regression: `cargo test --profile release-test --test issue_2070_rowbreak_density` 2/2 통과
- TAC helper: `cargo test --profile release-test tac_object_stack --lib` 3/3 통과
- 전체 회귀: `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review-2290 cargo test --profile release-test --tests` 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- 검토 시점 최신 PR head의 GitHub Actions: `Build & Test`, `Native Skia tests`, `CodeQL`,
  `Canvas visual diff` 통과. `WASM Build`와 `Frontend package gates`는 경로 조건으로 skipped다.

## Findings

### P1 - 대표 표의 overflow가 해소되지 않고 p26에서 p30으로 이동한다

`src/renderer/typeset.rs`의 `rowbreak_use_row_offsets` 조각 높이 하한 보정은 문서 페이지 수를 375에서
381로 올렸지만, 동봉 재현 샘플의 대표 구간을 기준 PDF 흐름으로 되돌리지 못한다. p26 overflow가 사라진
대신 p26 내용이 빠지고 p30에 대형 tail overflow가 생겼다. 이는 PR의 핵심 주장인 rowspan 연속 조각 잔여
증발 보정이 사용자에게 보이는 표 분할을 아직 해결하지 못한다는 뜻이다.

수정 전후 p25-35의 내용 연속성, 각 fragment의 cut 소비와 frame 내 배치를 함께 맞춰야 한다. p26의 ymax나
문서 총 페이지 수만으로 pass하면 같은 내용 이동 회귀를 허용하게 된다.

### P2 - 동봉 HWP를 직접 고정하는 통합 회귀 oracle이 없다

현재 테스트 갱신은 다른 시장구조조사 문서의 페이지 수 pin만 309쪽으로 변경한다. TAC 변경도 helper 단위
3건만 검사한다. `samples/task2287/1342000_edu_curriculum_map.hwp`의 p26 내용 보존과 p30 tail overflow
부재를 확인하는 renderer 통합 회귀가 필요하다. 최소한 문제의 rowspan fragment가 다음 페이지로 정상
이어지고 frame 밖 tail을 만들지 않는 구조 assertion을 추가해야 한다.

## 최종 권고

**Request changes / merge 보류.** PR은 페이지 수 개선과 helper 단위 검증에는 성공했지만, HWP 2020 기준
PDF와의 대표 구간 비교에서 대형 overflow를 다른 페이지로 옮긴 상태다. p26-p30 내용 연속성과 frame 내
분할을 보정하고, 동봉 HWP를 직접 고정하는 회귀를 추가한 새 head에서 focused test, 전체 회귀, 최신 CI,
visual sweep을 다시 확인해야 한다. [#2287](https://github.com/edwardkim/rhwp/issues/2287)은 open으로 유지한다.
