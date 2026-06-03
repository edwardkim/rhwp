# Task M100-1061 — HWPX 수식 저장 기능 구현 (구현 계획서)

- 이슈: [#1061](https://github.com/edwardkim/rhwp/issues/1061)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1061`
- 일시: 2026-05-22
- 수행 계획서: [`task_m100_1061.md`](task_m100_1061.md)

## 1. 본질 식별 (진단 도구 사전 분석 결과)

`examples/dump_equation_records.rs` 신규 작성 후 정답지 vs 저장본 IR 비교:

| 항목 | 정답지 (samples/math-001.hwp) | 저장본 (saved/111math-001.hwp) | 비고 |
|------|------------------------------|---------------------------------|------|
| Equation 개수 | 3 | 3 ✓ | |
| script | "sqrt {3} of {5} ..." | 동일 ✓ | |
| font_size | 1100 | 동일 ✓ | |
| color | 0x00000000 | 동일 ✓ | |
| baseline | 93 | 동일 ✓ | |
| **common.attr** | **0x0C2A2211** | **0x042A2211** | **bit 27 (0x08000000) 누락** |
| **font_name** | "Equation Version 60" | "HYhwpEQ" | HWP5 의미 swap |
| **version_info** | "" | "Equation Version 60" | (font_name 와 swap) |
| raw_ctrl_data[3] (attr byte 3) | 0x0C | 0x04 | 동일 본질 (bit 27) |
| common.size/pos/margin/treat_as_char | 동일 ✓ | | |

→ **본질 3 가지**:

1. **GenShape attr bit 27 (0x08000000)** 누락 — Table 어댑터 (`HWPX_TABLE_NUMBERING_BIT`) 와
   동일 패턴, Equation 어댑터에서 누락
2. **font_name ↔ version_info swap** — HWPX `<hp:equation>` 의 `font="HYhwpEQ"` +
   `version="Equation Version 60"` 속성 매핑이 HWP5 EQEDIT byte order
   (HWP5 spec 표 105: 첫 string=version, 둘째=font) 와 의미적으로는 일치하지만
   정답지의 byte order 는 (첫 string="Equation Version 60", 둘째="") 이므로 spec 의 자리값
   기준 정답지는 `version_info="Equation Version 60", font_name=""`. parser 가 정확히 동일 IR 을
   재구성해야 함.
3. **raw_ctrl_data 보존** — common.attr 정정 시 raw_ctrl_data 도 함께 갱신 필요 (직렬화 시
   raw 우선)

## 2. 단계 구성 (4 단계)

### Stage 1 — HWPX 어댑터 Equation arm 추가 + GenShape attr bit 27 보강

**파일**: `src/document_core/converters/hwpx_to_hwp.rs`

- `adapt_paragraph` 의 control match 에 `Control::Equation(eq)` arm 추가
- 신규 함수 `adapt_equation(eq, report)`:
  - `eq.common.attr |= HWPX_EQUATION_NUMBERING_BIT` (`0x08000000`, bit 27)
  - `eq.common.raw_extra` 처리 — Equation 은 raw_extra 없으나 안전 처리
  - `raw_ctrl_data` 가 비어있지 않으면 byte 3 도 동기화 (attr byte 3)
  - 또는 raw_ctrl_data 를 clear 하여 직렬화기가 common 으로 재합성하도록
- `AdapterReport` 에 `equation_attr_materialized` counter 추가

**검증**: Stage 1 단독으로 `examples/repro_1061_equation_save.rs` 실행 → 어댑터 후
common.attr 가 정답지와 동일 (`0x0C2A2211`) 인지 확인.

### Stage 2 — HWPX parser 의 font_name/version_info 매핑 정정

**파일**: `src/parser/hwpx/section.rs::parse_equation`

- HWPX `<hp:equation>` 의 `version` 속성 → `version_info` (현재 동일)
- HWPX `<hp:equation>` 의 `font` 속성 → `font_name` (현재 동일)

하지만 정답지 HWP5 의 EQEDIT 의 의미는 spec 표 105 와 같음:
- 첫 string = version 정보 (보통 "Equation Version 60")
- 둘째 string = font 이름 (보통 "")

HWPX parser 가 `version="Equation Version 60"` 를 `version_info` 로 정확 매핑.
HWPX 의 `font="HYhwpEQ"` 는 HWP5 의 어느 자리에도 없음 — **삭제 또는 안전 처리**.

후보 (a): `font_name = ""` 강제 (정답지 정합)
후보 (b): `font_name = ""` if 정답지 패턴, 그렇지 않으면 보존

→ 권장: **(a)** 정답지 정합 우선. HWPX `font` 속성은 한컴이 EQEDIT 에 저장하지 않는 정보로
판단.

**검증**: 저장본 재파싱 후 font_name="", version_info="Equation Version 60" 확인.

### Stage 3 — 회귀 가드 + sweep 회귀 점검

**파일**:
- `tests/issue_1061_equation_serialize.rs` 신규 — equation contract 검증:
  - `issue_1061_equation_attr_bit27_set` — common.attr 의 bit 27 set
  - `issue_1061_equation_font_version_swap_normalized` — font_name="", version_info="Equation Version 60"
  - `issue_1061_equation_count_preserved` — 3 equation 보존
  - `issue_1061_equation_script_preserved` — script 보존
- 광범위 sweep: 기존 12 fixtures (HWP/HWPX) 회귀 부재 확인 (수식 없는 fixture 영향 없음 확인)

### Stage 4 — WASM Docker 빌드 + 작업지시자 한컴 시각 판정 + 보고서

- WASM Docker 빌드 → `pkg/` → `rhwp-studio/public/` 동기화
- 산출물 `output/poc/issue_1061/repro.hwp` 작업지시자 시각 판정 게이트
- 통과 시 Stage 보고서 → 머지 → push → 이슈 close

## 3. 작업 절차 (각 Stage 마다)

1. 구현
2. `cargo test --release --lib` + `--tests` + clippy + fmt (`feedback_push_full_test_required`)
3. 자동 reproduce (`examples/repro_1061_equation_save.rs`) → IR 재파싱 정합 확인
4. Stage 보고서 (`mydocs/working/task_m100_1061_stageN.md`) 작성 + 작업지시자 승인 요청
5. 승인 후 다음 Stage

## 4. 위험 분석

| 위험 | 영향 | 완화 |
|------|------|------|
| HWPX `font` 속성 정보 손실 | font_name 정보 사라짐 | 정답지 정합 — 한컴 자체가 font 정보 EQEDIT 에 저장 안 함 |
| 다른 HWPX 출처 fixture 회귀 | 표/그림과 동일 영역 보강 패턴이지만 Equation 외 영향 부재 (수정 영역 한정) | sweep 12 fixtures 회귀 가드 |
| Stage 1 단독으로 한컴 호환 미달성 | 추가 라운드 필요 | Stage 2 (font swap) 동반 필수 |
| raw_ctrl_data 와 common.attr 의 일관성 | 직렬화기가 raw 우선이면 common.attr 갱신 무효 | raw_ctrl_data 도 함께 갱신 또는 clear |

## 5. 산출물

- `examples/dump_equation_records.rs` (이미 작성)
- `examples/repro_1061_equation_save.rs` (Stage 1 작성)
- `src/document_core/converters/hwpx_to_hwp.rs` (Stage 1)
- `src/parser/hwpx/section.rs::parse_equation` (Stage 2)
- `tests/issue_1061_equation_serialize.rs` (Stage 3)
- `mydocs/working/task_m100_1061_stage{1..4}.md`
- `output/poc/issue_1061/` 산출물
- 트러블슈팅 (필요 시): `mydocs/troubleshootings/hwpx_equation_save_*.md`
- 최종 보고서: `mydocs/report/task_m100_1061_report.md`

## 6. 작업지시자 승인 요청 사항

1. 본 구현 계획 (4 단계) 승인 여부
2. Stage 2 의 font_name 처리 방향 (a 강제 "" / b 조건부 보존) 중 (a) 권장 수용 여부
3. raw_ctrl_data 처리 방향 (clear 후 재합성 vs byte 3 patch) — clear 권장 수용 여부
