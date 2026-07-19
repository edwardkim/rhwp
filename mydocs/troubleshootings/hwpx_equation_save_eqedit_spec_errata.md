---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# HWPX 수식 저장 — EQEDIT spec errata + Stage 4-pivot oracle (Task #1061)

| 항목 | 내용 |
|------|------|
| 발견일 | 2026-05-22 |
| 이슈 | [#1061](https://github.com/edwardkim/rhwp/issues/1061) |
| 정답지 | `samples/math-001.hwp` (한컴 직접 저장) |
| HWPX 원본 | `samples/hwpx/math-001.hwpx` |
| 시각 정답지 | `pdf-large/hwpx/math-001.pdf` |
| 산출물 | `output/poc/issue_1061/repro_stage1.hwp` |
| 분석 도구 | `examples/dump_equation_records.rs`, `examples/dump_eqedit_raw.rs`, `examples/repro_1061_equation_save.rs` |
| 관련 | [`hwpx2hwp-rule.md`](hwpx2hwp-rule.md), [`hwp_spec_errata.md`](../tech/hwp_spec_errata.md), [`hwpx_footnote_save_5round_oracle_method.md`](hwpx_footnote_save_5round_oracle_method.md) |

## 증상

`samples/hwpx/math-001.hwpx` 를 rhwp-studio 에서 열어 HWP 로 저장 후 한컴 한글
2020 에서 열면:

1. **(Stage 1 전)** 한컴이 "파일 손상 / 파일을 열 수 없습니다" 또는 본문 깨짐
2. **(Stage 1 후, attr bit 27 보강만)** 한컴이 **수식만 표시하고 본문 텍스트 미표시**
3. **(Stage 2 후, unknown UINT2 정정)** 수식 + 본문 텍스트 모두 정상 표시 ✓

핵심: **rhwp-studio 자기 정합 정상** (모든 stage 에서) **vs 한컴 거부 단계적 회복**.
`feedback_self_verification_not_hancom` 의 결정적 입증.

## 방법론 — Stage 4-pivot oracle (Task #1058 패턴 재적용)

### 본질 식별 절차

1. 정답지 (한컴 직접 저장 HWP) vs 저장본 (rhwp HWP) 비교 도구 작성
2. record-level diff 분석 → 본질 후보 식별
3. 정정 + 시각 판정 게이트
4. 새 본질 (Stage 1 후 잔존) → raw byte 직접 분석 → 다음 본질 식별

### Stage 1 발견 — GenShape attr bit 27

`examples/dump_equation_records.rs` 의 IR diff 결과:

| 항목 | 정답지 | 저장본 (Stage 1 전) |
|------|--------|--------------------|
| common.attr | 0x0C2A2211 | 0x042A2211 |

→ bit 27 (0x08000000) 누락. Table 어댑터의 `HWPX_TABLE_NUMBERING_BIT` 와 동일 패턴.

**정정**: `src/document_core/converters/hwpx_to_hwp.rs::adapt_equation` 신규 —
`pack_common_attr_bits(common) | 0x08000000` + raw_ctrl_data clear.

**Stage 1 후 시각 판정**: 작업지시자 보고 "수식만 보이고 문자들이 안 보임".

### Stage 2 발견 — EQEDIT spec errata (진짜 본질)

Stage 1 단독으로 부족. **EQEDIT raw payload 직접 분석** (`examples/dump_eqedit_raw.rs`):

```
정답지 EQEDIT #1 (size=158):
0000: 00 00 00 00       attr (UINT4) = 0
      2A 00             script length = 42
      [script 84 byte]
0058: 4C 04 00 00       letter_size = 1100
      00 00 00 00       color = 0
      5D 00             baseLine = 93
      00 00             ← spec 표 105 누락! (UINT16 zero)
      13 00             version_info length = 19
      [Equation Version 60]
      07 00             font_name length = 7
      [HYhwpEQ]
```

**HWP5 spec 표 105 의 errata** — baseline 과 version_info 사이의 `UINT16 zero`
필드가 누락되어 있음.

**hwplib 검증** (`/home/edward/vsworks/shwp/hwplib`):

```java
// ForEQEdit.java
eqEdit.setBaseLine(sr.readSInt2());
eqEdit.setUnknown(sr.readUInt2());      // ← 스펙 누락 영역
eqEdit.getVersionInfo().setBytes(sr.readHWPString());
eqEdit.getFontName().setBytes(sr.readHWPString());
```

hwplib 가 spec 표 105 의 누락 영역을 `unknown` UINT2 로 정확 처리. write 측도 동일.

### 누락 시 증상의 메커니즘

unknown UINT2 를 read 하지 않으면:
- baseline 직후 `[00 00]` 을 version_info length 로 오인 → version_info = ""
- 그 다음 `[13 00]` 을 length 19 로 읽어 font_name = "Equation Version 60"
- → (version_info, font_name) 자리값 **swap**

이 swap 으로 EQEDIT 자체는 size 보존되어 한컴이 수식 표시. 하지만 EQEDIT 후속
record (PARA_TEXT 등) 의 byte align 이 깨져 **본문 텍스트 미표시**.

이는 단순 IR 차원 분석으로는 발견 불가능 — **raw byte 직접 분석** 필수.

## 핵심 학습

### 1. spec 만 따르면 안전하지 않음 — spec errata 의 결정성

HWP5 spec 은 한컴이 공식 공개한 권위 자료지만 errata 가 존재.
`mydocs/tech/hwp_spec_errata.md` 의 사전 검색이 결정적. 새 record 처리 시:

1. 공식 spec
2. hwplib (Java 권위 자료)
3. 실제 HWP fixture raw byte

**3 단계 교차 검증** 필수 (`hwp_spec_errata.md` 의 "검증 원칙" 정합).

### 2. IR 차원 정합 ≠ raw byte 정합

본 task 에서 Stage 1 단독 적용 후 IR 은 정답지와 (font_name, version_info) 자리값
swap 만 존재하고 다른 차이 없음. 그러나 한컴이 본문 미표시.

**raw byte 직접 분석** 까지 가야 본질 식별 가능. IR 차원에서 보이는 차이는 표면적
증상일 수 있음.

### 3. 단계적 시각 판정의 위험 — "수식만 보임" 의 정확한 해석

Stage 1 후 작업지시자 보고 "수식만 보임" 의 의미가 결정적:
- "수식 깨짐" 이면 EQEDIT 자체 결함
- "수식 정상 + 본문 미표시" 이면 **EQEDIT 후속 record byte align 손상**

후자가 단순 진단으로는 도달하기 어려운 본질 (record 간 byte align 손상). 작업지시자
보고 의 정확한 phrasing 이 진단의 결정적 단서.

### 4. inline 컨트롤 자기 정합 + paragraph contract 둘 다 보존

본 task 의 정답지 본문 paragraph 는 char_offsets=`[0,1,...,14, 23, 24,...]` 의 8 cu
jump 패턴. HWPX 파서가 이미 정확 생성 → 본문 paragraph 정정은 불필요. **수식 자체
record contract 만 정정**으로 충분.

이는 Task #1058 (footnote AutoNumber) 의 paragraph char_offsets contract 와 별개
영역 — 본 case 의 본질은 EQEDIT 자체에 있음.

### 5. hwplib 권위 자료의 결정성

본 errata 발견은 hwplib `ForEQEdit.java` 의 `readUInt2()` 한 줄이 결정적.
`reference_hwp2hwpx_library` 메모리 룰 정합 — 새 record 처리 시 hwplib 의 처리
패턴을 spec 보다 먼저 확인.

## 정정 영역 매트릭스

| 영역 | 파일 | 변경 |
|------|------|------|
| Equation IR | `src/model/control.rs` | `unknown: u16` 필드 추가 |
| HWP5 parser | `src/parser/control.rs::parse_equation_control` | baseline 후 `read_u16()` |
| HWP5 serializer | `src/serializer/control.rs::serialize_equation_control` | baseline 후 `write_u16(eq.unknown)` |
| HWPX parser | `src/parser/hwpx/section.rs::parse_equation` | unknown: 0 (default) |
| HWPX serializer test | `src/serializer/hwpx/mod.rs` | test fixture 갱신 |
| HWPX 어댑터 | `src/document_core/converters/hwpx_to_hwp.rs` | `adapt_equation` 신규 (attr bit 27 + raw_ctrl_data clear) |

## 회귀 가드

`tests/issue_1061_equation_serialize.rs` (7 가드):
- `issue_1061_equation_attr_bit27_materialized`
- `issue_1061_first_equation_attr_matches_oracle`
- `issue_1061_equation_unknown_field_parsed_correctly`
- `issue_1061_saved_equation_font_version_matches_oracle`
- `issue_1061_equation_count_preserved`
- `issue_1061_equation_script_preserved`
- `issue_1061_paragraph_char_offsets_with_inline_equation_preserved`

## 후속 작업

- 다른 수식 포함 HWP fixture (`samples/equation-lim.hwp`, `samples/exam_kor.hwp` 등)
  sweep 회귀 점검 — 본 task 의 unknown UINT2 정정이 다른 HWP 출처 fixture 의
  EQEDIT 라운드트립에 영향 없는지 확인 (회귀 가드는 본 fixture 한정)
- `hwpx2hwp-rule.md` 의 contract unit 항목에 본 task 추가 (다음 hwpx2hwp task 시)
- 광범위 HWPX → HWP 호환 (Task #178 영역) — 별도 task

## 관련 commits

Task #1061 (2026-05-22):
- Stage 1: HWPX 어댑터 Equation arm + attr bit 27
- Stage 2: EQEDIT unknown UINT2 + HWPX font 매핑 복원
- Stage 3: 회귀 가드 7
- Stage 4: 최종 보고서 + 트러블슈팅

## 관련 메모리 룰

- `feedback_self_verification_not_hancom` — Stage 1 후 rhwp 정상 / 한컴 본문 미표시
- `feedback_diagnosis_layer_attribution` — IR diff 한계 → raw byte 분석으로 본질
- `feedback_search_troubleshootings_first` — 본 문서 사전 검색 의무
- `feedback_visual_judgment_authority` — Stage 1 → 2 추가 본질 발견
- `feedback_hancom_compat_specific_over_general` — case-specific EQEDIT contract
- `reference_hwp2hwpx_library` — hwplib 결정적 근거 (spec errata 발견)
- `project_hwpx_to_hwp_adapter_limit` — 단순 어댑터 한계 점진 돌파
