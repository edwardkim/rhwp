# Task 1293 Stage 71: line_seg 기준 TAC 수식 textRun 배정 추적

## 목적

Stage70에서 `dump-pages`에 원본 미주 source와 `line_seg` 요약을 붙였다. 이번 단계에서는
그 정보를 사용해 문단 내부 `lineSegArray`/`Paragraph.line_segs`와 글자처럼 취급되는 수식
컨트롤의 TextRun/EquationNode 배정이 실제로 어떻게 이어지는지 확인한다.

## Stage70에서 정정된 전제

- 같은 원본 미주 컨트롤 안에서 연속 미주 문단의 첫 `line_seg.vertical_pos`가 같아도,
  한컴/PDF가 반드시 같은 단에 묶는 것은 아니다.
- `2024-11-practice-above0-between20-below2` page 10의 `문5`는 제목이 좌측 단 하단에 남고,
  본문/수식 textRun이 우측 단 상단으로 이어지는 것이 PDF 기준과 맞는다.
- 따라서 같은 vpos는 "분할 금지"가 아니라, 같은 line-seg 기준선에서 서로 다른 문단/textRun이
  이어질 수 있다는 신호로 해석한다.

## 확인할 경로

1. HWP/HWPX 원본 lineSegArray가 `Paragraph.line_segs`로 보존되는지 본다.
2. `control_text_positions()`가 글자처럼 취급해야 하는 수식 컨트롤의 char position을 어떻게 복원하는지 본다.
3. `compose_paragraph()`의 `ComposedLine.char_start`/`runs`와
   `paragraph_layout.rs::tac_offsets_for_line()`/`equation_only_tac_line_assignment()`가 같은
   line_seg 경계를 사용하는지 확인한다.
4. 페이지네이션에서는 제목/본문을 억지로 묶지 않고, 렌더러의 수식 TextRun/EquationNode가
   line_seg 기준으로 배치될 수 있도록 필요한 진단/보정을 찾는다.

## 대상

- `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp`
  - page 10 `문5` split
  - page 20 qflow 후보
- `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
  - page 11 `문13` 정상 배치
  - page 20 qflow 후보

## 계획

- 현재 CLI/렌더 트리에서 문단별 controls, control text position, composed line, line_seg를 함께 볼 수
  있는지 확인한다.
- 부족하면 임시 로그가 아니라 `dump-pages` 또는 별도 진단 명령에 최소한의 필드를 추가한다.
- 소스 수정은 문항 번호/문서명 조건이 아닌 `line_seg`/TAC 구조 조건으로만 진행한다.

## 진단 명령 추가

`rhwp dump-endnote-lines <파일.hwp> <section> <para> <control> [note-para]`를 추가해
원본 미주 컨트롤의 특정 미주 문단을 좁혀 본다.

출력 기준:

- `line_seg[n]`: 원본 `lineSegArray`가 보존된 `Paragraph.line_segs[n]` 값과
  `text_start`의 char index 매핑.
- `control[n]`: 문단 내 컨트롤 종류, `control_text_positions()`로 복원된 text position,
  TAC 수식의 저장 크기/스크립트.
- `composed_lines`: `compose_paragraph()`가 만든 TextRun 줄과 실제 renderer의
  `run_tacs` 기준으로 수식이 붙는 `layout_tacs`.

## Stage71 분석

- `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
  `s0:p108:ci0:note2`, `note3`에서 다음 패턴을 확인했다.
  - 이전 줄의 마지막 TextRun 끝 위치와 다음 `line_seg`/`ComposedLine.char_start`가 같다.
  - TAC 수식 position도 같은 값이다.
  - 기존 non-empty TextRun 경로는 문단 끝 TAC 보정을 위해 `pos == run_end`를 허용했고,
    다음 줄도 같은 TAC를 시작 위치로 받았다.
  - 결과적으로 수식이 이전 줄 끝과 다음 줄 시작에 중복 emit될 수 있다.
- 한컴 기준으로 이 boundary TAC는 “다음 line_seg 시작의 글자처럼 취급되는 수식”이다.
  따라서 다음 `ComposedLine.char_start == run_end`이면 현재 줄의 end TAC로 소유하지 않고
  다음 줄로 넘긴다.

## 검증

- `cargo fmt --all`
- `cargo build --bin rhwp`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 52개 통과.
- 부분 sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --target 2024-11-practice-above0-between20-below2 --out output/task1293_stage71_boundary_tac --rhwp-bin target/debug/rhwp`
  - 두 target 모두 SVG/PDF/render tree 페이지 수 1:1.
  - `equation_text_overlap_pages`는 비어 있음.
  - `frame_overflow_pages`는 비어 있음.
  - 남은 후보: `question_marker_flow_drift`, `line_band_drift`, `large_ink_region_drift`.

## 남은 문제

- boundary TAC 중복 emit은 공통 조건으로 제거했지만, 미주 전체 흐름은 아직 PDF와 다르다.
- 다음 단계에서는 `line_seg` boundary fix 이후에도 남는 qflow/large drift가
  미주 간격/구분선 위/미주 위치 계산 중 어느 축에서 발생하는지 계속 분리한다.
