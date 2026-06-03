# 구현 계획서 — Task #1233: 수식 큰 연산자(Σ/∏/∫) 피연산자 간격

- **이슈**: #1233 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1233-eq-bigop-spacing` (base: `stream/devel` f83c43b5)
- **수행계획서**: `plans/task_m100_1233.md`
- **작성일**: 2026-06-02

## 설계 요지

`equation/layout.rs::layout_row` 는 형제 노드를 간격 0(`x += b.width`)으로 배치하므로,
큰 연산자(BigOp)와 그 피연산자 사이 간격은 **BigOp 박스 width** 가 전적으로 결정한다.
현재 limits(`layout_big_op`)·적분(`layout_integral`) 모두 **trailing 간격이 없어** 피연산자가
붙는다. 두 함수의 box width 에 `BIG_OP_TRAIL_PAD`(fs 비율) trailing 간격을 더한다.

인라인 수식은 `svg.rs` 에서 컨트롤 advance(tac_w)에 맞춰 가로 스케일되므로, width 증가는
오버플로 없이 흡수되고 간격만 비례적으로 생긴다(자기완결적, 문단 advance 무변경).

## 단계 구성 (3단계)

### Stage 1 — limits 큰 연산자(Σ/∏) trailing 간격 + 상수 + 단위 테스트

**목표**: `layout_big_op` 의 limits 스타일 BigOp width 에 trailing 간격 추가.

- 상수 추가: `const BIG_OP_TRAIL_PAD: f64 = 0.1;` (fs 비율, `layout_symbol` pad 관례 참고).
  초기값 0.1 로 두고 Stage 3 PDF 대조에서 미세 조정.
- `layout_big_op`(L668-671): `width: max_w` → `width: max_w + fs * BIG_OP_TRAIL_PAD`.
  sup/sub 중앙정렬은 기존 `max_w` 기준 유지(연산자는 좌측, 우측에 순수 trailing 공백).
- **단위 테스트**(`layout.rs mod tests`): `parse_and_layout("sum _{n=1} ^inf b_n", fs)` 류로
  BigOp 박스 width 가 `max_w + fs*PAD` 임을, 그리고 피연산자 x 가 그만큼 우측 이동함을 단언.
- **1차 시각 확인**: `export-svg 3-09월_교육_통합_2023 -p5` → 문26 ∑bₙ 간격 생성 확인.
- **검증**: `cargo test --lib equation` 통과.

### Stage 2 — 적분(∫) trailing 간격

**목표**: `layout_integral`(nolimits, 우측 첨자)의 width 에 동일 trailing 간격 추가.

- `layout_integral`(L728): `width: total_w` → `width: total_w + fs * BIG_OP_TRAIL_PAD`.
- 적분은 우측 첨자 구조이므로 trailing 이 첨자 뒤에 오는지 시각 확인(`int _a ^b f(x)`).
- **검증**: 적분 포함 수식 SVG 시각 확인 + `cargo test --lib equation`.

### Stage 3 — 시각 검증·다수식 표본·회귀·문서화

**목표**: PDF 대조로 pad 값 확정 + 회귀 없음 증빙.

- **PDF 대조**: 6쪽 문26 ∑bₙ 간격을 `pdf/3-09월_교육_통합_2023.pdf` 와 시각 대조,
  필요 시 `BIG_OP_TRAIL_PAD` 미세 조정(과압축/과간격 점검).
- **다수식 표본**: ∑/∏/∫ 포함 샘플 2~3종에서 피연산자 간격·오버플로(다음 글자 침범) 없음 확인.
- **레이아웃 불변**: 수식 외 본문 `dump-pages -p5` 전후 동일(수식 tac_w 스케일 → advance 무변경).
- **회귀**: 전체 `cargo test --lib` 통과.
- **문서화**: 최종 보고서 `report/task_m100_1233_report.md`.

## 단계별 커밋 정책

각 Stage 완료 시 소스 + `working/task_m100_1233_stage{N}.md` 동반 커밋. 무관 rustfmt diff 금지.

## 검증 도구 요약

| 항목 | 명령 |
|------|------|
| 시각 대조 | `rhwp export-svg samples/3-09월_교육_통합_2023.hwp -p 5` ↔ `pdf/…2023.pdf` |
| 단위/회귀 | `cargo test --lib equation`, `cargo test --lib` |
| 레이아웃 불변 | `rhwp dump-pages … -p 5` (전후 비교) |
