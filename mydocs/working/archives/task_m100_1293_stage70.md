# Task 1293 Stage 70: 실제 flow mismatch source layout 분석

## 목적

Stage69에서 red marker 오탐을 줄인 뒤에도 실제 flow mismatch 후보가 남았다. 이번 단계에서는
남은 후보 page의 render tree와 compare PNG를 기준으로 source layout 원인을 좁힌다.

## 대상

- `2024-11-practice-above0-between0-below0`
  - page 20
  - page 11
- `2024-11-practice-above0-between20-below2`
  - page 10
  - page 20

## 확인 항목

- 문항 marker가 RHWP/PDF에서 어느 page/column에 배치되는지 확인한다.
- render tree의 해당 page text/shape bbox를 확인한다.
- 미주 설정값 0/0/0 또는 0/20/2가 page break 직전 content height에 어떻게 반영되는지 추정한다.
- source 수정이 필요한 경우 `typeset.rs`, `height_cursor.rs`, `layout.rs` 중 어느 경로인지 분리한다.

## 추가 분석 기준

작업지시자 피드백에 따라 남은 mismatch는 단순 `current_height` 수치 조정으로 보지 않는다.
다음 순서로 좁힌다.

1. 문단 원본의 HWPX `lineSegArray` / HWP5 `PARA_LINE_SEG`가 어떤 줄 경계를 제공하는지 확인한다.
2. rhwp 내부 `Paragraph.line_segs`의 `text_start`, `vertical_pos`, `line_height`,
   `text_height`, `baseline_distance`, `line_spacing`이 그 계약을 보존하는지 확인한다.
3. 글자처럼 취급되는 수식/그림/TAC 표가 `TextRun` 사이 또는 빈 `runs` 줄에서 어느
   `ComposedLine`에 배정되는지 확인한다.
4. 문항 제목만 하단에 남는 현상은 제목 한 줄 자체가 아니라, 직전/직후 문단의 TAC 수식/그림이
   `line_seg` 기준으로 예약한 높이와 렌더 `TextRun`/`EquationNode` 배치가 어긋난 결과인지
   확인한다.

선행 문서 `mydocs/tech/endnote_inline_eq_line_1239.md`의 결론처럼, 연속 인라인 수식은
텍스트 char position만으로 줄을 배정하면 한컴 `LINE_SEG` 경계를 놓칠 수 있다. 따라서
Stage70 이후 분석은 `control_text_positions()`가 반환한 char position만 보지 않고,
`tac_offsets_for_line()` / `equation_only_tac_line_assignment()`가 실제 `line_seg` 경계와
같은 줄 소속을 만들고 있는지 함께 확인한다.

## 현재 확인 결과

- `2024-11-practice-above0-between20-below2` page 10:
  - `pi=467`은 좌측 단 하단의 `문5）   22_11_실전 5) ⑤` 제목 한 줄이다.
  - `dump-pages` 기준 같은 vpos `67034`에서 다음 `pi=468` 본문이 우측 단으로 시작한다.
  - 진단 보강 후 `pi=467 src=s0:p37:ci0:note0`, `pi=468 src=s0:p37:ci0:note1`로
    확인됐다. 두 항목은 같은 원본 미주 컨트롤 안의 연속 미주 문단이고, 첫 `line_seg.vertical_pos`
    값이 모두 `67034`이다.
  - PDF 비교 결과 `문5` 제목은 좌측 단 하단에 남고, 본문/수식 textRun이 우측 단 상단으로
    이어진다. 따라서 같은 첫 vpos는 "한 단에 묶어야 함"이 아니라 "같은 line-seg 기준선에서
    문단/textRun이 단을 이어갈 수 있음"을 뜻한다.
- `2024-11-practice-above0-between0-below0` page 11:
  - 좌측 단은 `pi=537` 비TAC 그림과 `pi=538` 텍스트 tail 뒤에, 우측 단에서 `pi=539`
    `문13` 제목이 시작한다.
  - `pi=539 src=s0:p108:ci0:note0`, `pi=540 src=s0:p108:ci0:note1`도 같은 첫 vpos
    `222481` 묶음이다. 현재는 둘 다 우측 단에 있으므로, 같은 vpos 묶음 guard가 정상 배치를
    깨지 않는 회귀 감시 샘플로 본다.
  - 남은 차이는 `문13` 제목 자체보다 직전 `pi=537/538`의 shape/line-seg 소비와 다음 문항
    묶음의 줄 소속 판단이 한컴과 다른지 확인하는 쪽이 더 타당하다.

## Stage70 판단 정정

- 초기 가설: 같은 원본 미주 컨트롤 안에서 현재 첫 문단과 다음 문단의 첫
  `line_seg.vertical_pos`가 같으면 제목과 다음 textRun/수식 줄을 같은 단 묶음으로 봐야 한다고
  추정했다.
- 검증: 임시 guard를 넣어 `문5` 제목까지 우측 단으로 넘긴 뒤
  `output/task1293_stage70_same_line_seg_guard_v3/2024-11-practice-above0-between20-below2/compare/compare_010.png`
  를 확인했다.
- 결론: PDF 기준은 `문5` 제목이 좌측 단 하단에 남고, 본문/수식 textRun이 우측 단 상단으로
  이어지는 형태다. 따라서 같은 `vertical_pos`는 "분할 금지" 신호가 아니라, 글자처럼 취급되는
  수식/문단 textRun이 같은 line-seg 기준선에 걸쳐 단을 이어갈 수 있다는 진단 신호로 봐야 한다.
- 조치: `typeset.rs`의 same-line 묶음 guard 실험은 회귀이므로 되돌렸다. 이번 단계의 유효한
  산출물은 `dump-pages`에서 원본 미주 source와 `line_seg` 요약을 함께 보여 주는 진단 보강이다.

## 다음 확인

- 대상 문단을 `dump-pages`의 flatten `pi`에서 원본 미주 문단으로 역매핑한다.
- 이번 단계에서 `dump-pages`의 원본 미주 source와 `line_seg` 요약을 보강했으므로, 다음 단계는
  이 정보를 기반으로 `lineSegArray`/`line_seg`와 수식 `TextRun` 배정의 실제 차이를 확인한다.
- `paragraph_layout.rs`의 TAC 수식 줄 배정 경로가 `line_seg` 경계를 따라가는지 확인하고,
  필요하면 문항/문서 번호가 아닌 `LINE_SEG`/TAC 구조 조건으로 보정한다.
