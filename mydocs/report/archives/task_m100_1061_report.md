# Task M100-1061 — [hwpx2hwp] 수식과 글자 조합된 문서 저장 구현 (최종 보고서)

- 이슈: [#1061](https://github.com/edwardkim/rhwp/issues/1061) (CLOSED)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1061`
- 일시: 2026-05-22
- 작업지시자 한컴 한글 2020 시각 판정: 통과

## 1. 개요

HWPX 문서의 수식(`<hp:equation>`)과 글자 조합 저장 기능 구현. Task #1050 (각주) →
#1052 (글상자 안 각주) → #1058 (각주 contract 5 라운드) 누적 후 HWPX → HWP
어댑터의 다음 누락 영역 처리.

작업지시자 자료:
- `samples/hwpx/math-001.hwpx` — HWPX 원본 (수식 44개)
- `samples/math-001.hwp` — 정답지 (한컴 직접 저장)
- `pdf-large/hwpx/math-001.pdf` — 시각 정답지
- `saved/111math-001.hwp` — 본 task 출발점 (rhwp-studio 정정 전 저장본)

## 2. 본질 식별 — 2 단계 oracle 분석

### Stage 1: IR diff — common.attr bit 27 누락

`examples/dump_equation_records.rs` 진단:

| 항목 | 정답지 | 저장본 (정정 전) |
|------|--------|------------------|
| common.attr | 0x0C2A2211 | 0x042A2211 (bit 27 0x08000000 누락) |

→ HWPX 어댑터에 `Control::Equation` arm 누락. Table 어댑터 패턴 정합 정정.

**Stage 1 후 시각 판정**: "수식만 보이고 문자들이 안 보임" (작업지시자 보고).

### Stage 2: raw byte diff — EQEDIT spec errata

`examples/dump_eqedit_raw.rs` 신규 — EQEDIT raw payload 직접 분석:

```
정답지: ...baseLine 2byte | [00 00] ← spec 표 105 누락 | version_info...
rhwp:   ...baseLine 2byte | [version_info length 으로 오인] ...
```

**HWP5 spec 표 105 의 errata 발견** — baseline 과 version_info 사이의 UINT16 zero
필드 누락. hwplib `ForEQEdit.readUInt2()` 정확 처리.

→ rhwp parser/serializer 가 spec 만 따라 unknown UINT2 read 안 함. byte align 한
글자 밀려 (version_info, font_name) swap → 후속 record (PARA_TEXT 등) align 손상
→ 한컴 본문 미표시.

**Stage 2 후 시각 판정**: "본문이 잘 보입니다" ✓.

## 3. 정정 영역 매트릭스

### 3.1 `src/document_core/converters/hwpx_to_hwp.rs` (Stage 1)

- `AdapterReport`: 신규 필드 2 (`equation_ctrl_header_attr_materialized`,
  `equation_font_version_normalized`)
- `adapt_paragraph` match 에 `Control::Equation(eq)` arm
- 신규 함수 `adapt_equation`:
  ```rust
  fn adapt_equation(eq: &mut Equation, report: &mut AdapterReport) {
      const HWPX_EQUATION_NUMBERING_BIT: u32 = 0x0800_0000;
      let before = eq.common.attr;
      eq.common.attr = pack_common_attr_bits(&eq.common) | HWPX_EQUATION_NUMBERING_BIT;
      let raw_was_present = !eq.raw_ctrl_data.is_empty();
      eq.raw_ctrl_data.clear();
      if eq.common.attr != before || raw_was_present {
          report.equation_ctrl_header_attr_materialized += 1;
      }
  }
  ```

### 3.2 `src/model/control.rs::Equation` (Stage 2)

신규 필드:
```rust
pub baseline: i16,
pub unknown: u16,  // 신규 — HWP5 spec 표 105 errata, hwplib ForEQEdit.readUInt2() 정합
pub version_info: String,
pub font_name: String,
```

### 3.3 `src/parser/control.rs::parse_equation_control` (Stage 2)

```rust
equation.baseline = r.read_i16().unwrap_or(0);
equation.unknown = r.read_u16().unwrap_or(0);  // 신규
if let Ok(ver) = r.read_hwp_string() { equation.version_info = ver; }
```

### 3.4 `src/serializer/control.rs::serialize_equation_control` (Stage 2)

```rust
w.write_i16(eq.baseline).unwrap();
w.write_u16(eq.unknown).unwrap();  // 신규
w.write_hwp_string(&eq.version_info).unwrap();
```

### 3.5 `src/parser/hwpx/section.rs::parse_equation` (Stage 2)

- HWPX `font="HYhwpEQ"` → `equation.font_name` 매핑 (Stage 2 첫 가설 후 복원)
- Equation 생성자에 `unknown: 0` 추가

### 3.6 `src/serializer/hwpx/mod.rs` (Stage 2)

test fixture 의 Equation 생성자에 `unknown: 0` 추가.

## 4. 정량 입증

### 4.1 IR record-level 정합

| Equation 항목 | 정답지 | Stage 1+2 저장본 |
|---------------|--------|----------------|
| common.attr | 0x0C2A2211 | 0x0C2A2211 ✓ |
| font_name | "HYhwpEQ" | "HYhwpEQ" ✓ |
| version_info | "Equation Version 60" | "Equation Version 60" ✓ |
| script / font_size / color / baseline | 동일 | 동일 ✓ |
| common.size / pos / margin | 동일 | 동일 ✓ |
| unknown | 0 | 0 ✓ |

### 4.2 본문 paragraph contract 정합

| 본문 paragraph (예: pi 5) | 정답지 | Stage 1+2 저장본 |
|--------------------------|--------|------------------|
| text | "첫째항과 공비가 모두 양수 인 등비수열 이 만족시킬 때, 의 값은? [3점]" | 동일 ✓ |
| char_offsets | `[0,1,...,14, 23, 24,...]` (수식 inline 8 cu jump) | 동일 ✓ |
| char_count | 67 | 67 ✓ |

### 4.3 회귀 가드 7/7 통과

`tests/issue_1061_equation_serialize.rs`:

| Test | 검증 영역 |
|------|----------|
| `issue_1061_equation_attr_bit27_materialized` | HWPX 모든 Equation attr bit 27 |
| `issue_1061_first_equation_attr_matches_oracle` | 첫 수식 attr (0x0C2A2211) |
| `issue_1061_equation_unknown_field_parsed_correctly` | 정답지 parse 결과 정합 |
| `issue_1061_saved_equation_font_version_matches_oracle` | 저장본 font/version |
| `issue_1061_equation_count_preserved` | 수식 개수 보존 |
| `issue_1061_equation_script_preserved` | 첫 수식 script 보존 |
| `issue_1061_paragraph_char_offsets_with_inline_equation_preserved` | 본문 inline char_offsets |

### 4.4 CI 패턴

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1323 passed / 0 failed / 6 ignored** |
| cargo test --release --tests | 54 test files / 0 FAILED |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |
| WASM Docker 빌드 (4.91 MB) | success |
| WASM 동기화 (rhwp-studio/public/) | done |

### 4.5 작업지시자 한컴 한글 2020 시각 판정 2 라운드

| Round | 영역 | 결과 |
|-------|------|------|
| Stage 1 | attr bit 27 보강만 | "수식만 보이고 본문 안 보임" (잔존) |
| Stage 2 | EQEDIT unknown UINT2 추가 | **"본문이 잘 보입니다"** ✓ |

추가: rhwp-studio WASM 동작 판정 통과.

## 5. 메모리 룰 정합

- ✅ `feedback_visual_judgment_authority` — Stage 1/2 2 라운드 시각 판정 게이트
- ✅ `feedback_diagnosis_layer_attribution` — IR diff 한계 → raw byte 분석 본질 식별
- ✅ `feedback_self_verification_not_hancom` — Stage 1 후 rhwp 정상 / 한컴 본문 미표시
- ✅ `feedback_hancom_compat_specific_over_general` — case-specific EQEDIT contract
- ✅ `feedback_push_full_test_required` — lib + tests + clippy + fmt + WASM 모두 통과
- ✅ `feedback_search_troubleshootings_first` — 본 task 시작 시 troubleshootings 사전 검색
- ✅ `reference_hwp2hwpx_library` — hwplib 결정적 근거 (spec errata 발견)
- ✅ `project_hwpx_to_hwp_adapter_limit` — Task #1050 → #1058 → 본 task 누적

## 6. 산출물

| 위치 | 내용 |
|------|------|
| `mydocs/plans/task_m100_1061.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1061_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_1061_stage1.md` | Stage 1 보고서 |
| `mydocs/working/task_m100_1061_stage2_3.md` | Stage 2+3 보고서 |
| `mydocs/report/task_m100_1061_report.md` | 최종 보고서 (본 문서) |
| `mydocs/troubleshootings/hwpx_equation_save_eqedit_spec_errata.md` | 트러블슈팅 |
| `mydocs/tech/hwp_spec_errata.md` | EQEDIT unknown UINT2 항목 추가 |
| `tests/issue_1061_equation_serialize.rs` | 회귀 가드 7 |
| `examples/dump_equation_records.rs` | IR diff 진단 도구 |
| `examples/dump_eqedit_raw.rs` | raw payload 진단 도구 |
| `examples/repro_1061_equation_save.rs` | reproduce 도구 |
| `output/poc/issue_1061/repro_stage1.hwp` | 시각 판정 산출물 |

## 7. 후속

- 다른 수식 fixture (`samples/equation-lim.hwp`, `samples/exam_kor.hwp`) sweep
  회귀 점검 — 본 task 의 unknown UINT2 정정이 영향 없는지 확인
- `hwpx2hwp-rule.md` 의 contract unit 누적 (다음 hwpx2hwp task 시)
- 광범위 HWPX → HWP 호환 (Task #178 영역) — 별도 task

## 8. `hwpx2hwp-rule.md` contract unit 누적

Task #1050 → #1052 → #1058 의 contract 에 본 task 추가:

- **HWPX `<hp:equation>` → HWP CTRL_HEADER attr bit 27 (0x08000000)** 보강
  (HWPX_EQUATION_NUMBERING_BIT, Table 패턴 정합)
- **HWP5 EQEDIT spec 표 105 errata** — baseline 과 version_info 사이 UINT2 zero
  (hwplib ForEQEdit.readUInt2() 정합)

작업지시자 통찰 Stage 4-pivot oracle 방법론의 결정성 — 본 task 에서도 정답지 vs
저장본 raw byte 분석으로 spec errata 의 본질 정확 식별.
