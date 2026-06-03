# Task #229 Fix — 구현계획서

## 구현 방향 확정

`TextStyle` 에 플래그를 추가하는 대신, **`extra_char_spacing < 0.0` 일 때만 per-char 최소 advance 클램프 적용**.

근거:
- 음수 `extra_char_spacing` 은 `paragraph_layout.rs` 의 세 분기에서만 설정됨:
  1. Justify (내부 공백 없음) — 라인 911–946
  2. Distribute/Split — 라인 950–956
  3. **비정렬 오버플로우 압축** — 라인 957–966 (← 이번 버그의 원인)
  셋 다 음수 공통 의미는 "전체 폭을 줄이기 위한 압축" 이므로 "압축 중" 시그널로 재사용 가능.
- 기존 `letter_spacing`(문자 모양) 음수는 건드리지 않음 → 기존 렌더 동일.
- 새 필드 없음 → 직렬화/`Default`/호출부 변경 불필요.

## 단계 구성 (총 4단계)

### 단계 1 — 재현 테스트 및 베이스라인 고정

목표: 수정 전 버그 상태를 회귀 테스트로 포획.

작업:
- `tests/svg_snapshot.rs` 에 `table_text_page_0` 확인(이미 존재).
- `tests/golden_svg/table-text/page-0.svg` 는 버그 상태로 오늘 이미 생성됨 → **이 단계에서 기존 골든을 삭제하고 그대로 둠**(수정 후 재생성 예정).
- 추가 단위 테스트 (`tests/text_measurement_overflow.rs` 신규, 또는 기존 모듈 test 모듈 확장):
  - 케이스: `"65,063,026,600"` 을 `extra_char_spacing = -2.88`, font_size=12, ratio=1, 맑은 고딕으로 측정 → `compute_char_positions` 반환 벡터가 **단조 비감소**여야 함.
  - 현재는 실패(assertion fail) 해야 함 (Red).

검증:
- `cargo test --test text_measurement_overflow` → Red 확인
- `cargo test --release --lib` → 890 기존 테스트는 통과 유지

커밋 메시지: `Task #229 fix: 오버플로우 압축 셀 글자 역진 회귀 테스트 추가`

### 단계 2 — `EmbeddedTextMeasurer` / `estimate_text_width_unrounded` 조건부 클램프 복원

대상 파일: `src/renderer/layout/text_measurement.rs`

수정: `EmbeddedTextMeasurer::estimate_text_width`, `EmbeddedTextMeasurer::compute_char_positions`, `estimate_text_width_unrounded` 세 곳의 `char_width` 클로저에 다음 클램프 추가:

```rust
let mut w = base_w * ratio + style.letter_spacing + style.extra_char_spacing;
if c == ' ' { w += style.extra_word_spacing; }
// 오버플로우/Justify/Distribute 압축(extra_char_spacing < 0) 시에만
// per-char 최소 advance = base_w * ratio * 0.5 로 클램프하여
// narrow glyph(콤마/마침표)가 뒷 글자와 역진 겹침되는 것을 방지.
if style.extra_char_spacing < 0.0 {
    let min_w = base_w * ratio * 0.5;
    w = w.max(min_w);
}
w
```

검증:
- 단계 1의 실패 테스트가 Green
- `cargo test --release --lib` → 890+ 통과
- `samples/hwpx/form-002.hwpx` export-svg 출력이 커밋 `21a02ec` 직후와 바이트 동일 (비-overflow 회귀 없음 확인용)

커밋 메시지: `Task #229 fix: 네이티브 측정 경로에 overflow 한정 per-char 50% 클램프 복원`

### 단계 3 — `WasmTextMeasurer` 동일 반영

대상 파일: `src/renderer/layout/text_measurement.rs` (라인 509–, 581– 두 곳)

`WasmTextMeasurer::estimate_text_width`, `compute_char_positions` 클로저에도 동일한 `extra_char_spacing < 0.0` 가드 + 클램프 삽입.

검증:
- `cargo build --target wasm32-unknown-unknown --release -p rhwp`
- WASM 경로는 CI/스냅샷이 없으므로 빌드 성공만 검증 (원래 8c9b366 에서도 동일 정책).

커밋 메시지: `Task #229 fix: WASM 측정 경로에도 동일한 overflow 클램프 적용`

### 단계 4 — 골든 갱신 및 시각 검증

작업:
- `UPDATE_GOLDEN=1 cargo test --test svg_snapshot table_text_page_0` → 골든 덮어쓰기
- `cargo run --release --bin rhwp -- export-svg samples/hwpx/table-text.hwpx -o /tmp/verify` → x 좌표 단조성 스크립트로 확인
- 최종 보고서 `mydocs/working/task_m100_229_fix_report.md` 작성
- `mydocs/orders/20260422.md` 에 본 타스크 완료 기록

검증:
- `cargo test --test svg_snapshot` 전체 Green (기존 `form_002_page_0` 은 이번 수정으로 영향 없음 — 단 현재 실패 상태인지 사전에 확인 필요)
- SVG 육안 비교: `table-text-hwp.png` 레퍼런스 레이아웃과 근사
- `cargo test --release --lib` → 통과

커밋 메시지: `Task #229 fix: golden 갱신 및 최종 보고서`

## 단계별 완료 체크리스트

- [ ] 단계 1 — 회귀 테스트 Red 확인 후 보고서 `_stage1.md`
- [ ] 단계 2 — 네이티브 3곳 클램프 + Green 확인 후 `_stage2.md`
- [ ] 단계 3 — WASM 2곳 클램프 + 빌드 확인 후 `_stage3.md`
- [ ] 단계 4 — 골든/문서 정리 후 `_report.md`

## 롤백 전략

문제 발생 시 `git revert` 로 단계별 되돌리기. 단계 2, 3 은 `text_measurement.rs` 에 국한되어 격리가 깔끔함.

## 영향 범위 요약

| 영역 | 영향 |
|------|------|
| 비-overflow 셀 렌더 | 없음 (`extra_char_spacing == 0.0`) |
| Justify/Distribute | 기존 평균-기반 letter_spacing 클램프 + per-char 보조 클램프가 추가 보장 (기존 동작은 이미 50% 평균 클램프로 안전) |
| Overflow 압축 | 콤마/마침표 역진 해소 → 총 폭이 약간 증가할 수 있음 (셀 clip-path 로 시각적 overflow 는 방지됨) |
| WASM | 네이티브와 동일 |

## 미해결 사항 (별도 타스크 대상)

- `form_002_page_0` 스냅샷이 현재 실패 상태(이번 대화 시작 시점에 `page-0.actual.svg` 미커밋 존재). 이번 수정과는 **무관**하며 원인 조사는 별도 이슈로 분리.
