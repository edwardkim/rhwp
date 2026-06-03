# Task #229 Fix — 오버플로우 압축 셀에서 narrow glyph 역진 겹침 수정

## 배경

Task #229 1차 작업(커밋 `8c9b366`)에서 `text_measurement.rs` 5곳에 per-char 50% 최소폭 클램프(`w.max(base_w * ratio * 0.5)`)를 추가하여 음수 `letter_spacing` 압축 시 글자 겹침을 방지했음.

후속 커밋 `21a02ec` 에서 "오버플로우와 무관한 측정에까지 영향을 줘서" 해당 클램프 5곳을 모두 제거.
→ 정상 셀의 측정은 복구되었으나, **오버플로우 압축 셀에서 narrow glyph(콤마/마침표)가 뒷 글자에 역진 겹침**되는 회귀가 발생.

## 재현

- 샘플: `samples/hwpx/table-text.hwpx`
- 명령: `cargo run --release --bin rhwp -- export-svg samples/hwpx/table-text.hwpx`
- 증상: `"65,063,026,600"` 셀에서 각 콤마 다음 숫자가 콤마보다 x 좌표가 **작은** 위치에 배치 → 콤마 위에 숫자가 겹침.

계산 근거: 12pt 기준 숫자 advance ≈ 6.6px, 콤마 base advance ≈ 2.61px, 압축 `letter_spacing` ≈ −2.88px → 콤마 effective advance = 2.61 + (−2.88) = **−0.27px** (음수).

## 목표

오버플로우 압축이 적용된 셀에서만 per-char 최소 advance를 `base_w * ratio * 0.5` 로 클램프하여 글자 역진 방지. 정상(non-overflow) 셀의 측정·배치 동작은 `21a02ec` 상태 그대로 유지.

## 수용 기준

1. `samples/hwpx/table-text.hwpx` → `export-svg` 출력 SVG에서 모든 text 요소의 x 좌표가 단조 증가(같은 line 내).
2. 렌더 결과가 레퍼런스(`table-text-hwp.png`)의 시각적 레이아웃과 근사(각 숫자가 셀 경계 안).
3. SVG 스냅샷 테스트 (`cargo test --test svg_snapshot`) — 신규 `table-text/page-0.svg` 골든을 수정 반영으로 갱신, `form-002` 기존 골든은 변경 없음.
4. `cargo test --release --lib` 890+ 테스트 통과 유지.
5. 변경 없음 확인: 비-오버플로우 셀의 `compute_char_positions` 결과는 `21a02ec` 와 바이트 동일.

## 범위

**포함**
- `src/renderer/layout/text_measurement.rs` — 오버플로우 경로에서만 per-char 클램프 적용하도록 경로 구분
- `TextStyle` 에 플래그(예: `clamp_min_advance: bool`) 추가 또는 동등 효과 파라미터
- 압축 자간을 계산·적용하는 호출부에서 플래그 설정
- 테스트·골든 갱신

**제외**
- 표 패딩 축소 로직(`shrink_cell_padding_for_overflow`) 수정
- Justify/Distribute 경로의 평균-기준 letter_spacing 클램프 정책 변경
- 폰트 메트릭 DB 변경

## 리스크 / 주의사항

- `TextStyle` 은 여러 측정 함수가 공유. 플래그 기본값은 `false` 로 두어 비-오버플로우 경로 영향 0 을 보장.
- WASM 경로(`WasmTextMeasurer`)도 동일하게 반영 필요(원래 8c9b366 에서 4곳 + estimate_unrounded 1곳).
- 골든 `table-text/page-0.svg` 는 이번 수정으로 재생성(UPDATE_GOLDEN=1).

## 산출물

- 구현계획서: `mydocs/plans/task_m100_229_fix_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_229_fix_stage{N}.md`
- 최종 보고서: `mydocs/working/task_m100_229_fix_report.md`
