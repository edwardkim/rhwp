# Task M100 #2164 최종 보고서

- 이슈: [#2164](https://github.com/edwardkim/rhwp/issues/2164)
- 브랜치: `codex/task_m100_2164`
- 재현 원본: `samples/issue2164/의견제출서(양식).hwp`
- 원본 SHA-256: `a8f1874850d9970ba2f9cff868ada599081c349d71fe3c4a3e7a1023e399c98c`
- 작성일: 2026-07-10

## 결과

Chrome 확장 v0.2.8에서 보고된 표 셀 Enter 문단 겹침을 실제 제보 HWP로 재현해
정정했다. 셀 문단을 분할하거나 병합할 때 이후 문단의 세로 좌표를 다시 연결하고,
프론트 커서가 새 셀 문단을 가리키도록 flat 위치와 `cellPath`를 같이 갱신한다.

## 변경 내용

1. 셀 구조 편집 뒤 문단별 `LINE_SEG.vertical_pos`와 캐럿 y를 순서대로 재계산한다.
2. 신규 빈 문단은 자신의 활성 글자 모양 높이를 사용한다.
3. 실제 `RowBreak`의 저장된 vpos 원점은 유지하되, 방금 Enter로 삽입된 문단의 임시
   원점은 재계산을 막는 경계로 오인하지 않는다.
4. 셀 Enter/Backspace 뒤 `paragraphIndex`, `cellParaIndex`, `cellPath`의 마지막
   문단 인덱스, 캐럿 rect 캐시를 함께 갱신한다.
5. 실제 제보 원본으로 첫 Enter, 연속 Enter, Backspace 후 재Enter, HWP 저장 후
   재로드를 검증하는 Rust 회귀 테스트를 추가했다.

## 시각 확인

`문단 부호` 표시 상태에서 `1111`, Enter, `2222`, Enter를 수행했을 때 두 번째 Enter
직후 캐럿은 첫 줄이 아니라 새 세 번째 문단에 위치한다.

```text
cellParaIndex / paragraphIndex: 2 / 2
path cursor y: 445.3px
DOM caret top: 455.3px
```

작업지시자가 실제 화면에서도 수정 동작을 검증했다.

## 전체 사전 검증

- `cargo build --release`: 통과
- `cargo test --release --lib`: 2190 passed, 0 failed, 7 ignored
- `cargo test --profile release-test --tests`: 통과
  - `tests/issue_2164_cell_enter_overlap.rs`: 3 passed
  - `tests/svg_snapshot.rs`: 8 passed
- `cargo fmt --check`, `git diff --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo test --doc`: 0 passed, 1 ignored
- `rhwp-studio`: `npx tsc --noEmit`, 183개 `npm test`, production build 통과
- `rhwp-chrome`: production build 통과
- `wasm-pack build --target web --out-dir pkg`: 통과

## 기준 PDF와 Visual Sweep

원본 HWP를 HWP 2020 CLI 인쇄 방식으로 PDF 변환해
`pdf/issue2164/의견제출서(양식)-2020.pdf`에 보존했다.

- PDF: 1쪽 A4, SHA-256
  `52c530901ced05ce19d70525930ba0bbb07735e92d019d9e90be9ebf2cfdbdea`
- visual sweep: [summary](assets/task_m100_2164_visual_sweep/summary.json),
  [대표 review 이미지](assets/task_m100_2164_visual_sweep/issue2164/review/review_001.png)
- SVG/PDF 페이지 수: 1 / 1
- 자동 overflow, overlap, question flow, line order 후보: 0쪽
- pixel match: 94.26049%, ink match 및 visual accuracy proxy: 7.79583%

직접 overlay 확인에서 셀 경계와 문서 흐름에는 자동 후보가 없었다. 다만 한컴 기준 PDF와
로컬 rhwp 렌더의 글꼴 metrics가 달라 ink 정합률은 낮다. 이는 이번 편집 중 Enter/캐럿
동작 정정의 성공 근거나 실패 판정으로 사용하지 않으며, 한컴과의 정적 폰트 fidelity는
별도 품질 축으로 남긴다. 동적 편집 결과는 PDF export가 아니라 실제 브라우저와
작업지시자 화면에서 검증했다.

## 반영

- source PR: [#2166](https://github.com/edwardkim/rhwp/pull/2166)
- merge commit: `f3712542e980b1eab625624f54eb49e789ddfc5d`
- 대표 검토 asset: `mydocs/pr/assets/pr_2166_issue2164_review.png`
- #2164는 source PR의 closing keyword로 자동 close된 것을 확인했다.
