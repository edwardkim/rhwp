# Task M100-1061 Stage 2~3 — EQEDIT unknown UINT2 field + 회귀 가드

- 이슈: [#1061](https://github.com/edwardkim/rhwp/issues/1061)
- 단계: Stage 2 + Stage 3
- 브랜치: `local/task1061`
- 일시: 2026-05-22

## 1. 본질 식별 진화 — 작업지시자 추가 보고

Stage 1 단독 적용 후 작업지시자 시각 판정:
> "저장된 hwp 를 한컴편집기로 열면 수식만 보이고 문자들이 안 보인다는 겁니다."

→ Stage 1 의 attr bit 27 보강은 **필요 조건이지만 충분 조건 아님**. 추가 본질 영역 식별 필요.

### Stage 4-pivot 패턴 재적용 — EQEDIT raw payload 직접 분석

`examples/dump_eqedit_raw.rs` 신규 — 정답지의 EQEDIT raw byte 분석:

```
정답지 samples/math-001.hwp EQEDIT #1 (size=158):
  0000: 00 00 00 00       attr (UINT4) = 0
        2A 00             script length = 42
        [script 84 byte UTF-16LE]
  0058: 4C 04 00 00       letter_size = 1100
        00 00 00 00       color = 0
        5D 00             baseLine = 93
        00 00             unknown (UINT2) = 0  ← spec 표 105 누락!
        13 00             version_info length = 19
        [Equation Version 60]
        07 00             font_name length = 7
        [HYhwpEQ]
```

**진짜 본질**: HWP5 spec 표 105 가 EQEDIT 의 `baseline` 과 `version_info` 사이의
**UINT16 zero** 필드를 누락. hwplib `ForEQEdit.read()` 는 `readUInt2()` 정확 처리.

→ rhwp parser/serializer 가 spec 만 따르고 unknown UINT2 처리 안 한 결과, 정답지의
byte align 이 한 글자만큼 밀려 (version_info, font_name) 자리값 swap. 한컴이
잘못된 byte order 의 EQEDIT 를 읽으면 EQEDIT 그 자체 (수식) 는 표시하지만 후속
record (PARA_TEXT 등) 의 byte align 이 깨져 본문 텍스트 미표시.

## 2. 정정 영역 (Stage 2)

### 2.1 `src/model/control.rs::Equation`

신규 필드 `unknown: u16` 추가 (HWP5 spec errata 영역):

```rust
pub struct Equation {
    ...
    pub baseline: i16,
    pub unknown: u16,  // 신규 — hwplib ForEQEdit.readUInt2() 정합
    pub version_info: String,
    pub font_name: String,
    ...
}
```

### 2.2 `src/parser/control.rs::parse_equation_control`

baseline 후 unknown UINT2 read:

```rust
equation.baseline = r.read_i16().unwrap_or(0);
equation.unknown = r.read_u16().unwrap_or(0);  // 신규
if let Ok(ver) = r.read_hwp_string() { equation.version_info = ver; }
```

### 2.3 `src/serializer/control.rs::serialize_equation_control`

baseline 후 unknown UINT2 write:

```rust
w.write_i16(eq.baseline).unwrap();
w.write_u16(eq.unknown).unwrap();  // 신규
w.write_hwp_string(&eq.version_info).unwrap();
```

### 2.4 `src/parser/hwpx/section.rs::parse_equation`

- HWPX `font="HYhwpEQ"` → `equation.font_name` 매핑 **복원** (Stage 2 첫 시도에서 강제
  "" 했던 가설이 잘못 — 실제 본질은 EQEDIT byte order swap)
- Equation 생성자에 `unknown: 0` 추가

### 2.5 `src/serializer/hwpx/mod.rs` 의 test 정정

Equation 생성자에 `unknown: 0` 추가.

## 3. 정량 입증

### 3.1 정답지 vs Stage 1+2 저장본 IR 완전 정합

| Equation 항목 | 정답지 | Stage 1+2 저장본 |
|---------------|--------|----------------|
| attr | 0x0C2A2211 | 0x0C2A2211 ✓ |
| font_name | "HYhwpEQ" | "HYhwpEQ" ✓ |
| version_info | "Equation Version 60" | "Equation Version 60" ✓ |
| script | 동일 | 동일 ✓ |
| font_size / color / baseline | 동일 | 동일 ✓ |
| common.size / pos / margin | 동일 | 동일 ✓ |

### 3.2 본문 paragraph contract 정합

| 본문 paragraph (예: pi 5) | 정답지 | Stage 1+2 저장본 |
|--------------------------|--------|------------------|
| text | "첫째항과 공비가 모두 양수 인 등비수열 이 만족시킬 때, 의 값은? [3점]" | 동일 ✓ |
| char_offsets (수식 jump 8 cu) | `[0,1,...,14, 23, 24,...]` | 동일 ✓ |
| char_count | 67 | 67 ✓ |
| control_mask | 0x800 | 0x800 ✓ |

### 3.3 작업지시자 한컴 한글 2020 시각 판정 통과

> "output/poc/issue_1061/repro_stage1.hwp 도 본문이 잘 보입니다."

수식 + 본문 텍스트 모두 정상 표시.

## 4. 회귀 가드 (Stage 3)

`tests/issue_1061_equation_serialize.rs` 신규 7 가드:

| Test | 검증 |
|------|------|
| `issue_1061_equation_attr_bit27_materialized` | HWPX 출처 모든 Equation 의 attr bit 27 set |
| `issue_1061_first_equation_attr_matches_oracle` | 첫 수식 attr 정답지 정합 (0x0C2A2211) |
| `issue_1061_equation_unknown_field_parsed_correctly` | 정답지 parse 결과 font_name/version_info 정합 |
| `issue_1061_saved_equation_font_version_matches_oracle` | 저장본 첫 수식 font/version 정합 |
| `issue_1061_equation_count_preserved` | 수식 개수 보존 |
| `issue_1061_equation_script_preserved` | 첫 수식 script 보존 |
| `issue_1061_paragraph_char_offsets_with_inline_equation_preserved` | 본문 inline 수식 paragraph char_offsets 정합 |

**7/7 통과**.

## 5. CI 패턴

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | **1323 passed / 0 failed** |
| cargo test --release --tests | 54 test files / 0 FAILED |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |
| WASM Docker 빌드 (4.91 MB) | success |
| WASM 동기화 (rhwp-studio/public/) | done |

## 6. 메모리 룰 정합

- `feedback_visual_judgment_authority` — 작업지시자 한컴 시각 판정 게이트 (Stage 1 → 2 추가 본질 발견)
- `feedback_diagnosis_layer_attribution` — Stage 1 만으로 부족, EQEDIT raw byte 분석으로 진짜 본질 (unknown UINT2) 식별
- `feedback_self_verification_not_hancom` — rhwp 자기 정합 (raw_ctrl_data 보존) ≠ 한컴 호환
- `feedback_hancom_compat_specific_over_general` — case-specific contract (EQEDIT unknown UINT2 spec errata)
- `feedback_push_full_test_required` — lib + tests + clippy + fmt + WASM 모두 통과
- `project_hwpx_to_hwp_adapter_limit` 정합 + **단순 어댑터 한계 점진 돌파** (Task #1050 → #1058 → 본 task)
- `reference_hwp2hwpx_library` — hwplib 권위 자료가 spec errata 정확 식별의 결정적 근거

## 7. `hwp_spec_errata.md` 추가 후보 (Stage 4 에서 반영)

- **EQEDIT (HWP5 spec 표 105) — baseline 과 version_info 사이 UINT2 zero 누락**
  - hwplib `ForEQEdit.readUInt2()` 정합
  - 누락 시 (version_info, font_name) 자리값 swap → 한컴 본문 미표시

## 8. 작업지시자 승인 요청

Stage 2~3 완료 (시각 판정 통과 + 회귀 가드 7 통과 + WASM 동기화). Stage 4 (최종 보고서 +
트러블슈팅 + commit/merge/push + 이슈 close) 진행 승인 여부.
