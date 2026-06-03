---
타스크: #178 HWPX→HWP IR 매핑 어댑터
단계: Stage 2 — table.raw_ctrl_data 합성 + table.attr 재구성
브랜치: local/task178
작성일: 2026-04-19
선행: Stage 1 완료 (mydocs/working/task_m100_178_stage1.md)
---

# Stage 2 단계별 완료 보고서

## 1. 목표 (구현계획서 §2 Stage 2)

표가 HWP 직렬화기에서 정상 인식되도록 `raw_ctrl_data` (CommonObjAttr 직렬화) 합성 + `table.attr` IR 일관성 동기화.

## 2. 산출물

### 2.1 신규 코드

| 파일 | 변경 |
|---|---|
| [src/document_core/converters/common_obj_attr_writer.rs](src/document_core/converters/common_obj_attr_writer.rs) | placeholder → 본격 직렬화기 (158줄). `serialize_common_obj_attr(&CommonObjAttr) -> Vec<u8>` + `pack_common_attr_bits` 비트 합성 + 6개 enum→비트 헬퍼 |
| [src/document_core/converters/hwpx_to_hwp.rs](src/document_core/converters/hwpx_to_hwp.rs) | no-op 골격 → 표 어댑터 본체. `adapt_paragraph` + `adapt_table` 추가, 셀 내부 문단 재귀 처리 |

### 2.2 수정 파일

없음 (Stage 1 산출물에 본격 코드 추가만).

### 2.3 HWP 직렬화기 변경

**0줄 수정** — 정체성 (구현계획서 §1.0.1) 준수.

## 3. 검증 결과

### 3.1 단위 테스트 (11개, +5)

```
common_obj_attr_writer:
  preserves_existing_attr_when_nonzero ... ok
  produces_min_43_bytes ... ok
  roundtrip_default ... ok                  ← parse(serialize(x)) == x
  roundtrip_treat_as_char ... ok            ← treat_as_char + BehindText
  roundtrip_with_description ... ok         ← UTF-16LE 한글 보존
  synthesizes_attr_when_zero ... ok         ← HWPX 출처 (attr=0) 복원

hwpx_to_hwp:
  empty_doc_no_change ... ok
  hwp_source_no_op_via_filter ... ok
  idempotent_when_called_twice ... ok
```

### 3.2 통합 테스트 (11개, +5)

```
stage2_raw_ctrl_data_synthesized_for_hwpx_h_01 ... ok    ← 26개 표 모두 ctrl_data 채움
stage2_diagnostics_no_longer_flag_table_ctrl_data ... ok ← 진단 도구 위반 0건
stage2_idempotent_does_not_double_synthesize ... ok      ← 2차 호출 합성 0
stage2_hwp_source_unchanged ... ok                       ← HWP 출처 raw_ctrl_data 비변경
stage2_page_count_after_adapter_hwpx_h_01 ... ok         ← 페이지 수 회귀 없음
adapter_deterministic_across_clones ... ok               ← Stage 1 의 idempotent 갱신
```

### 3.3 회귀 (전체 라이브러리)

```
test result: ok. 886 passed; 0 failed; 1 ignored; 0 measured
```

→ Stage 1 대비 +5건 (단위 테스트 추가분).

## 4. 페이지 폭주 진척 측정

| 샘플 | 원본 | Stage 1 (어댑터 X) | **Stage 2 (어댑터 ○)** | 개선 |
|---|---:|---:|---:|---:|
| hwpx-h-01 | 9 | 200 | **169** | -31 (약 16% 감소) |

→ 표 26개 ctrl_data 합성만으로 일부 페이지 폭주 회복. 나머지 (160 페이지 초과분) 는 Stage 4 의 lineseg vpos 사전계산 + Stage 3 셀 list_attr bit 16 가 처리할 영역.

## 5. 핵심 설계 결정

### 5.1 작성기는 parser 의 정확한 역방향

`parse_common_obj_attr` (parser/control/shape.rs:247) 의 필드 읽기 순서·바이트 크기를 1:1 대칭으로 작성. 라운드트립 테스트 4개로 단언:

- `roundtrip_default`: 모든 비-string 필드
- `roundtrip_treat_as_char`: 비트 필드 (treat_as_char + text_wrap)
- `roundtrip_with_description`: HWP UTF-16LE 문자열
- `preserves_existing_attr_when_nonzero`: HWP 출처 (attr 비-0) 보존

### 5.2 attr 비트 합성 — HWPX 출처 (attr=0) 전용

HWPX 파서는 `table.common.text_wrap` 등 enum 만 채우고 `common.attr = 0` 으로 둠. 작성기는 `attr != 0` 이면 그대로 사용 (HWP 출처 보존), `attr == 0` 이면 enum 으로부터 비트 재구성. `synthesizes_attr_when_zero` 테스트가 검증.

### 5.3 셀 내부 문단 재귀

`adapt_table` 마지막 단계에서 `cell.paragraphs` 를 재귀 처리. 중첩 표 (표 안의 표) 도 자동 대응.

### 5.4 idempotent 보장

`if table.raw_ctrl_data.is_empty()` 가드로 2차 호출 시 합성 0건. `stage2_idempotent_does_not_double_synthesize` 테스트.

## 6. 정체성 셀프 체크

- [x] HWP 직렬화기 0줄 수정 (`git diff src/serializer/` 출력 0줄)
- [x] 어댑터는 IR 만 만짐 (`&mut Document` 단일 진입점)
- [x] idempotent (Stage 2 통합 테스트로 단언)
- [x] HWP 출처 보호 (`stage2_hwp_source_unchanged` 통합 테스트로 단언)
- [x] 매핑 명세는 직렬화기 가정 기준 (`serializer/control.rs:349` 인용)

## 7. 다음 단계

Stage 3: 셀 `list_attr bit 16` (apply_inner_margin) — 셀 안 여백 지정 비트 보강.

Stage 3 시작 전 `serializer/control.rs:429` 가 작성하는 LIST_HEADER 바이트 레이아웃 검증 (옵션 A vs B 결정) 단위 테스트 우선 작성 예정 (구현계획서 §4 위험 1).

## 8. 승인 요청

본 단계 완료 보고서 승인 후 Stage 3 착수.
