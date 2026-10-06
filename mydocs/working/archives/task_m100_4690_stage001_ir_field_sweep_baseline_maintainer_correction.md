# #4690 Stage 1: IR 필드 스윕 baseline 메인터너 보정

## 목적

PR #4735가 추가한 `issue4690/30098_indent_over_stored_cs.hwp` fixture 때문에
기존 HWP 재구축 왕복의 `list_header_width_ref` 발산이 CI에 새로 노출됐다.
새 렌더링 회귀 가드는 유지하면서, 이미 존재하는 직렬화 한계를 baseline에 명시한다.

## 관찰

- GitHub Actions run `31702315505`, shard 2/3는 IR 필드 스윕에서만 실패했다.
- 실패값은 `sections[].paragraphs[].controls[].cells[].list_header_width_ref: 0 -> 124`다.
- PR #4735의 변경은 fixture와 `LINE_SEG.column_start` 렌더링 회귀 테스트뿐이며,
  파서나 HWP 직렬화 경로는 변경하지 않는다.

## 판정과 변경

이 값은 새 fixture가 드러낸 기존 HWP 재구축 왕복 발산이다. 렌더러 구현이나
fixture의 회귀 가드를 우회하지 않고, `hwp5rb` lane의 정규화된 경로와 실제 건수
124를 baseline에 추가한다.

## 검증

`cargo test --profile release-test --test ir_field_sweep_baseline -- --nocapture`로
전체 스윕이 baseline 증가 없이 통과해야 한다.
