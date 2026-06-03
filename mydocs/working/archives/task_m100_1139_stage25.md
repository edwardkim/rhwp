# Task M100 #1139 Stage 25

## 목적

Stage24 이후 `3-09월_교육_통합_2022.hwp`의 16쪽 `문26)` 겹침, 17쪽 하단 overflow, 미주 문단 드래그 선택 회귀를 한컴오피스 정답지(PDF) 기준으로 해결한다.

## 시작 기준

- 기준 커밋: `0fd5c100` (`task 1139: Stage24 미주 분할 검증 문서 정리`)
- Stage24까지는 문30(`pi=928..`) 시작 위치 및 분배 조건을 회귀 테스트로 고정했다.
- Stage25부터는 “완전 해결 전 커밋 금지” 규칙을 적용한다.
  - Stage25가 시각적으로 확정되기 전까지는 로컬 커밋을 만들지 않는다.
  - Stage25 최종 확정 시 1개의 커밋으로만 관리한다.
- 2026-05-30: 로컬 커밋 누적분(`73d324d9`, `d56eccf0`, `e8e56b89`, `28041e2b`)은 `git reset --mixed 0fd5c100`으로 히스토리에서 제거하고 워킹트리 변경으로 되돌렸다.

## 보고된 문제

- 16쪽 우측 단의 `문26)` 내용이 서로 겹쳐 보인다.
- 17쪽 마지막 내용이 페이지 하단으로 overflow된다.
- 이전 단계에서 가능했던 미주 포함 내용 드래그 선택이 회귀되어 선택 하이라이트가 생성되지 않는다.

## 판단

- `HeightCursor`의 broad start-height backtrack 조건을 넓히는 접근은 16쪽 `문26)`의 Y 위치를 과하게 당겨 겹침을 만들었다. 현재 워킹트리에는 `src/renderer/layout.rs` 변경이 없다.
- 미주 가상 문단 중 lineSeg `vertical_pos`가 단조 증가하는 문단은 렌더링 시 줄별 원본 VPOS를 직접 기준으로 삼아야 한다. 일반 누적 높이 방식으로만 배치하면 16/17/18쪽 하단에서 한컴보다 크게 밀리거나 overflow가 발생한다.
- 드래그 선택 회귀는 본문 뒤에 붙는 미주 가상 문단(`section.paragraphs.len() + n`)을 선택/커서 질의가 원본문단 배열에서만 찾으면서 발생했다.

## 구현 기록(현재 워킹트리 변경)

- `src/renderer/layout/paragraph_layout.rs`
  - 셀 밖의 미주 가상 문단에 한해 lineSeg `vertical_pos`가 단조 증가하면 첫 줄 Y를 기준으로 줄별 Y를 재계산한다.
  - 이 보정은 가상 미주 문단에만 적용해 일반 본문/셀 문단의 높이 누적 동작을 건드리지 않는다.
- `src/document_core/queries/cursor_nav.rs`
  - 본문 문단과 미주 가상 문단을 함께 찾는 `get_render_paragraph_ref`를 추가했다.
  - `get_line_info_native`, `resolve_paragraph`, `get_selection_rects_native`가 미주 가상 문단을 정상 참조하도록 연결했다.
- `src/document_core/queries/cursor_rect.rs`
  - 커서 좌표 계산 중 footnote/endnote marker 위치, inline flow cursor fallback, inline control fallback이 미주 가상 문단도 참조하도록 보정했다.
- `src/renderer/typeset.rs`
  - 문30 케이스에서 split 후보 우선순위를 `split_endnote_to_fit` 우선으로 재정렬했다.
  - `split_endnote_to_fit` 판정에서 `en_fit` 및 `total_advance_fit` 기준을 함께 고려하도록 조정했다.
  - 기본 미주 사이 간격 문서는 새 미주 시작 임계값을 모든 단에서 95%로 완화해 17쪽 좌측 단 하단에 `문29)` 시작이 남도록 했다.
  - split된 미주 문단 뒤에는 이전 문단 전체 bottom vpos를 다음 문단 기준으로 재사용하지 않도록 끊었다.
- `src/renderer/height_cursor.rs`
  - 미주 하단부 backward 보정 허용은 유지하되, 보정된 y를 반환하는 경우 page/lazy base를 추가로 이동하지 않도록 정리했다.
  - 회귀 방지 unit test `backward_correction_keeps_page_base`를 추가했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - 17쪽 좌측 단 하단 `pi=900..901` 잔류를 고정했다.
  - 17/18쪽에서 `pi=931`이 `lines=0..4` / `lines=4..9`로 이어지는 조건을 고정했다.
  - 16쪽 `문26)` 미주 가상 문단(`pi=868`)의 줄 정보, 커서 좌표, 선택 사각형 생성 회귀 테스트를 추가했다.

## 검증 결과

- `cargo fmt --check` 통과.
- `cargo build` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
- `cargo test --lib backward_correction_keeps_page_base -- --nocapture` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
- `dump-pages`:
  - 16쪽(페이지 인덱스 15) 우측 단에 `pi=867..874`가 들어가며 `문26)` 마지막 수식 문단이 페이지 안에 남는다.
  - 17쪽(페이지 인덱스 16) 좌측 단 하단에 `pi=900..901`이 남는다.
  - 17쪽 우측 단에 `pi=928..930` + `PartialParagraph pi=931 lines=0..4`가 남는다.
  - 18쪽(페이지 인덱스 17) 좌측 단은 `PartialParagraph pi=931 lines=4..9`로 이어진다.
  - 18쪽 좌측 단은 `pi=949`까지, 우측 단은 `pi=950`부터 시작해 하단 overflow를 피한다.
- SVG export:
  - 16쪽(페이지 인덱스 15) `LAYOUT_OVERFLOW` 없음.
  - 17쪽(페이지 인덱스 16) `LAYOUT_OVERFLOW` 없음.
  - 18쪽(페이지 인덱스 17) `LAYOUT_OVERFLOW` 없음.
  - 비교 산출물:
    - `output/task1139_stage25_verify/compare_016_side.png`
    - `output/task1139_stage25_verify/compare_017_side.png`
    - `output/task1139_stage25_verify/compare_018_side.png`

## 현재 상태

- 자동 검증은 통과했으며, 16/17/18쪽 overflow 및 미주 선택 회귀는 Stage25 범위에서 해결했다.
- 2026-05-30: 작업지시자가 “현재까지 상황 커밋 후, 19페이지 한컴과 다름. 새로운 스테이지 26 시작”을 지시했다.
- 19쪽 한컴오피스 대비 차이는 Stage25 커밋 이후 Stage26에서 별도 추적한다.
