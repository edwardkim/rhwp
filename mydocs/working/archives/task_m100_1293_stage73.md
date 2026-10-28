# Task 1293 Stage 73: 미주 advance 판정 line_seg 정밀화

## 목적

Stage72에서 페이지네이터의 줄별 TAC 소유권을 렌더러와 맞췄지만,
`2024-11-practice-above0-between0-below0`와
`2024-11-practice-above0-between20-below2`의 `question_marker_flow_drift`는 그대로 남았다.

이번 단계는 남은 drift를 `format_paragraph` 높이 측정 문제가 아니라 미주 묶음의
단/쪽 이동 판단 문제로 보고, 다음 값들을 같은 표로 대조한다.

- 문단 단위가 아니라 `lineSegArray -> line_seg.text_start -> TextRun/TAC` 순서로 좁힌다.
  수식은 떠 있는 객체가 아니라 글자처럼 취급되는 `Control::Equation(treat_as_char)`이며,
  `composed.tac_controls`의 위치가 해당 `line_seg`/TextRun 소유권을 결정한다.
- 미주 원본 `lineSegArray`의 `line_seg.vertical_pos`, `line_height`, `line_spacing`
- `format_paragraph`가 만든 `line_advance`
- `compute_en_metrics`의 `en_fit`, `total_advance_fit`
- `advance_for_fit`, `advance_for_new_endnote`, `advance_for_internal_rewind`의 결정
- 실제 `dump-pages`의 page item source와 한컴/PDF compare PNG의 문항 흐름

## 대상

- `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
  - 12쪽: RHWP는 오른쪽 단이 문16부터 시작하지만 PDF는 문18부터 시작한다.
- `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp`
  - 10쪽/20쪽: `question_marker_flow_drift`가 남아 있다.

## 계획

1. 환경 변수로 켤 수 있는 미주 advance 진단 출력을 추가한다.
2. `RHWP_ENDNOTE_LINE_DEBUG=1`로 미주 문단의 줄별 `line_seg`, composed line,
   글자처럼 취급되는 TAC 수식/객체의 text position을 함께 출력한다.
3. page item을 실제로 밀거나 남기는 조건을 문항 번호가 아니라 `line_seg` 구조와 미주 모양 값으로
   설명할 수 있는지 확인한다.
4. 진단 결과가 명확하면 공통 조건으로 수정하고 focused test와 partial sweep으로 검증한다.
5. 진단만 추가해도 원인이 불명확하면 stage73에는 진단 결과만 기록하고 다음 stage에서 좁힌다.

## 진행 메모

- `RHWP_ENDNOTE_ADVANCE_DEBUG=1` 진단을 추가해 `advance_for_fit/new/internal`의 최종
  판정값을 확인할 수 있게 했다.
- `RHWP_ENDNOTE_LINE_DEBUG=1` 진단을 추가해 각 미주 문단의 `line_seg.text_start`,
  `vertical_pos`, `line_height`, `line_spacing`, `format_paragraph` 줄 높이, composed line의
  char 범위, 해당 줄에 소속된 TAC 수식/객체를 같은 로그에서 볼 수 있게 했다.
- 12쪽 문제에서 `구분선위0/미주사이0/구분선아래0` 샘플은 RHWP가 문16 제목을 오른쪽 단으로
  밀지만, 한컴/PDF는 문16/문17 일부를 왼쪽 단 하단에 남기고 오른쪽 단을 문18부터 시작한다.
  따라서 단순히 문16의 advance threshold만 낮추거나 높이는 방식은 정답이 아니다.
- 이전 실험으로 "수식이 섞인 줄의 line_height를 TAC 높이/텍스트 높이로 축소"해 보았지만
  `question_marker_flow_drift`가 개선되지 않았고 일부 페이지 후보가 늘었다. 수식은 line_seg에
  속한 textRun/TAC로 보아야 하며, 문단 전체 높이를 일괄 보정하면 다시 overwrap으로 이어진다.

## 확인 결과

- `cargo fmt --all && cargo build --bin rhwp && cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 통과: 52 passed.
- `RHWP_ENDNOTE_ADVANCE_DEBUG=1 RHWP_ENDNOTE_LINE_DEBUG=1 target/debug/rhwp dump-pages samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp -p 11`
  - 12쪽 문15 후반부는 `line_seg.text_start`별 TAC 수식 소유권이 정상적으로 잡힌다.
  - 문15 ep15~ep17에는 같은 줄 TextRun에 `eqh2070` 수식이 포함되고, 원본 `line_seg.line_height`
    역시 27.6px로 커져 있다. 따라서 수식 높이를 별도 floating object처럼 보정하지 않는다.
  - 문16 ep0은 현재 단 높이 `915.73px`, 가용 높이 `1001.56px`에서 `advance_for_new_endnote=true`
    로 다음 단으로 이동한다. 제목 한 줄과 문16 전체는 현재 단에 들어갈 수 있으므로, 남은 수정은
    0/0/0 미주 profile의 새 미주 시작 advance 조건을 line_seg 기반 tail fit로 바꾸는 것이다.
