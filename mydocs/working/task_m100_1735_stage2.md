# Stage 2 완료보고 — 폭 정합 (M100 #1735)

- **이슈**: [edwardkim/rhwp#1735](https://github.com/edwardkim/rhwp/issues/1735)
- **브랜치**: `local/task1735`

## 목표
측정-렌더 폭 정합. 방점은 좁은 가운데 점(·)으로 렌더되므로 측정 폭도 narrow(0.3em)로 분류(기존 0.5em 기본 폴백 방지).

## 설계 교정
당초 구현계획서는 방점을 **전각**으로 가정했으나, Stage 1 시각 검증(export-pdf) 결과 0.5em 도 이미 한컴과 근접하며 **전각은 오히려 `각` 앞 간격을 늘려 회귀**임을 확인. 렌더가 방점을 좁은 · 로 그리므로 측정도 **narrow** 로 맞추는 것이 정합 방향임을 실측으로 교정.

## 변경
- `src/renderer/layout/text_measurement.rs`
  - `is_narrow_punctuation` 에 `U+302E`/`U+302F` 추가 → 미등록 폰트 폴백 폭 0.5em → 0.3em.
- 테스트: `test_narrow_glyph_tone_marks` 추가(TDD): `가〮나`/`가〯나` 의 방점 advance ≤ 0.35em.

## TDD
- RED: 방점 advance 8.33px(0.5em) > 5.83px(0.35em) → 실패.
- GREEN: narrow 분류 후 통과.

## 검증
- 재렌더 결과 SVG `<circle cx=134.84>`·`각 x=144.386` 으로 Stage 1 과 **동일**(회귀 없음). 본 문서는 단일 run 이라 렌더는 확장 텍스트 내부 위치를 사용하므로 측정 변경은 시각 무영향이나, justify/줄바꿈 등에서 측정-렌더 일관성을 확보.

## 게이트
- `cargo test --lib`: 2046 passed, 0 failed.
- `cargo clippy --lib`: 클린.
- `rustfmt --check`(수정 파일): 클린.
