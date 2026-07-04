# 구현계획서 — 방점(U+302E/U+302F) 렌더링 정합 (M100 #1735)

- **이슈**: [edwardkim/rhwp#1735](https://github.com/edwardkim/rhwp/issues/1735)
- **브랜치**: `local/task1735`
- **수행계획서**: [task_m100_1735.md](task_m100_1735.md)
- **작업지시자 확인**: 스튜디오 스크린샷상 방점 자리에 dotted-circle(◌)+점 결합 아티팩트 재현됨(함초롬바탕 20pt).

## 설계 결정

- **치환 위치**: 렌더 전용 확장 `expand_pua_display_text`([composer.rs:1691](../../src/renderer/composer.rs#L1691)). 모든 렌더 표면 + paint/json 스튜디오 계약이 이 함수를 경유하므로 단일 수정으로 전 표면 정합. IR 텍스트는 불변 → 캐럿/오프셋/측정 원본/텍스트추출 보존.
- **치환 글리프**:
  - `U+302E → U+00B7`(· MIDDLE DOT) — 한컴 PDF의 `·각` 시각과 직접 일치. (대안 U+318D ㆍ; 세로 위치 불만 시 Stage 2에서 교체.)
  - `U+302F → U+205A`(⁚ TWO DOT PUNCTUATION, 세로 두 점) — best-effort. 현재 샘플 미포함 → 정답지 없이 미검증, 보고서에 명시.
- **폭 일관성**: 측정은 IR 원본 U+302E/U+302F에 대해 수행되므로, 이 2자를 full-width로 분류해 치환 글리프(가운데 점, 1셀)와 advance 일치. `is_fullwidth_symbol`([text_measurement.rs:1866](../../src/renderer/layout/text_measurement.rs#L1866))에 2자 추가. (시각 검증 후 반각이 더 맞으면 조정.)

## 단계 (3단계)

### Stage 1 — 렌더 치환 (dotted-circle 제거)
- `composer.rs`에 톤마크 → 가운데 점 치환 추가. `pua_plain_text_display`와 동일 패턴의 소헬퍼 `tone_mark_display(ch) -> Option<char>` 신설, `expand_pua_display_text` 루프에서 PUA 분기 이전에 적용.
- **테스트 우선(TDD)**: `expand_pua_render_text("〮 각")` → 선두가 U+00B7, U+25CC 미포함. `U+302F → U+205A` 단위 테스트.
- **검증**: `cargo build` 후 `export-svg` → SVG `<text>`에 U+302E/U+25CC 미출현, U+00B7 출현.
- 커밋 + `working/task_m100_1735_stage1.md`.

### Stage 2 — 폭 분류 일관화 + 시각 정합
- `text_measurement.rs` `is_fullwidth_symbol`에 U+302E/U+302F 추가.
- **테스트**: 측정 헬퍼로 U+302E 폭이 full-width(=font_size)인지 assert.
- **시각 검증**: 렌더 산출을 한컴 정답지 `samples/unicode/각 항목에 명시되어 있는_유니코드.pdf`와 비교(점 위치·크기, `각`과의 간격, 겹침/과대공백 없음). 필요 시 치환 글리프/폭 미세 조정.
- 커밋 + `working/task_m100_1735_stage2.md`.

### Stage 3 — 통합 검증 + 게이트 + 보고
- `cargo fmt`(수정 파일 한정) → `cargo test` → `cargo clippy` 통과.
- 최종 시각 비교 산출물 `output/poc/task1735/` 정리, 작업지시자 시각 판정 요청.
- `report/task_m100_1735_report.md` 작성. fork push → upstream `devel` PR(`Refs #1735`).

## 리스크 / 대응
- U+302F 정답지 부재 → best-effort 구현, 미검증 명시(후속 이슈 여지).
- 치환-측정 폭 불일치로 겹침/공백 → Stage 2 시각 검증으로 수렴.
- 방점 뒤 source 공백(`〮 각`)에 의한 미세 간격차 → 주 증상(아티팩트) 해결 후 판단, 필요 시 별도 처리.

---
**승인 요청**: 본 구현계획서 승인 후 Stage 1부터 진행합니다.
