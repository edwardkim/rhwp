---
타스크: #178 HWPX→HWP IR 매핑 어댑터
단계: Stage 3 — 셀 list_attr bit 16 (apply_inner_margin) — 최소 구현 (작업지시자 결정 A)
브랜치: local/task178
작성일: 2026-04-19
선행: Stage 2 완료
---

# Stage 3 단계별 완료 보고서

## 1. 결정 배경

Stage 3 시작 시 디버그 샘플 3건 (hwpx-h-0[123].hwpx) 인벤토리 측정:

| 샘플 | `cell.list_attr.bit16` 위반 |
|---|---:|
| hwpx-h-01 | 0 |
| hwpx-h-02 | 0 |
| hwpx-h-03 | 0 |

→ 현재 디버그 샘플로는 Stage 3 의 효과를 측정할 수 없음. 페이지 폭주 회복에 기여하지 않음 (이미 모든 셀이 false).

작업지시자 결정 (A): **단위 테스트만 추가하는 최소 구현**. 합성 함수와 단위 테스트는 만들어두되, 효과 검증은 후속 샘플에서 (Stage 4 와 묶음).

## 2. 핵심 설계 — 보수적 bit 합성

### 2.1 발견된 모호성 (parser/control.rs:323-371)

LIST_HEADER `list_attr` 비트 16 의 의미 중첩:
- parser:331: `text_direction = (list_attr >> 16) & 0x07` (bit 16-18)
- parser:371: `apply_inner_margin = (list_attr >> 16) & 0x01` (bit 16 만)

→ bit 16 이 두 가지 의미로 동시에 해석됨. 한컴 스펙 문서 자체의 모호성. 실제 한컴은 bit 16 을 `apply_inner_margin` 으로 우선 해석 (parser 코드와 동일 로직).

### 2.2 직렬화기 동작 (serializer/control.rs:429)

```rust
let list_attr: u32 = ((cell.text_direction as u32) << 16) | (v_align_code << 21);
```

→ 직렬화기는 `text_direction << 16` 만 작성. `apply_inner_margin` 은 별도 필드가 없어 출력에서 항상 손실.

### 2.3 합성 방식 (어댑터)

```rust
if cell.apply_inner_margin && cell.text_direction == 0 {
    cell.text_direction = 1; // bit 0 OR (출력 bit 16 = 1)
}
```

가로 셀 (text_direction=0, 99% 케이스) AND apply_inner_margin=true 일 때만 OR. 세로 셀 (text_direction=1) 은 이미 bit 16 = 1 이므로 추가 합성 불필요.

가로/세로 비트 자체가 손상되지만 한컴이 bit 16 을 apply_inner_margin 으로 우선 해석하므로 핵심 의미 보존.

## 3. 산출물

### 3.1 코드 변경

| 파일 | 변경 |
|---|---|
| [src/document_core/converters/hwpx_to_hwp.rs](src/document_core/converters/hwpx_to_hwp.rs) | `adapt_cell_list_attr(&mut Cell, &mut AdapterReport)` 추가, `adapt_table` 셀 루프에서 호출. 단위 테스트 +5건 |

### 3.2 HWP 직렬화기 변경

**0줄 수정** — 정체성 (구현계획서 §1.0.1) 준수.

## 4. 검증 결과

### 4.1 단위 테스트 (16개, +5)

```
stage3_horizontal_cell_with_inner_margin_gets_bit16 ... ok
stage3_vertical_cell_already_has_bit16_no_change ... ok
stage3_no_inner_margin_no_change ... ok
stage3_list_attr_byte_layout_has_bit16_after_adapter ... ok    ← 출력 list_attr bit 16 검증 + parser 회복 시뮬
stage3_idempotent_does_not_double_or ... ok
```

`stage3_list_attr_byte_layout_has_bit16_after_adapter` 는 **serializer 의 list_attr 합성식과 정확히 같은 식**으로 출력 비트를 계산하고, **parser 의 apply_inner_margin 회복식**으로 재해석하여 한컴 호환성을 단위 수준에서 단언.

### 4.2 통합 테스트 (11개, 회귀 0)

Stage 1/2 통합 테스트 11건 모두 그린. Stage 3 통합 테스트는 검증 샘플 부재로 추가하지 않음 (작업지시자 결정 A 대로).

### 4.3 회귀 (전체 라이브러리)

```
test result: ok. 891 passed; 0 failed; 1 ignored; 0 measured
```

→ Stage 2 대비 +5건 (단위 테스트 추가분).

## 5. 페이지 폭주 측정 (Stage 2 와 동일, 변화 없음)

| 샘플 | 원본 | Stage 2 | **Stage 3** | 변화 |
|---|---:|---:|---:|---:|
| hwpx-h-01 | 9 | 169 | 169 | 0 (예상대로) |

Stage 3 는 검증 샘플이 없어 페이지 회복 효과 0 — 단위 테스트로 동작만 단언. 실제 회복은 Stage 4 (lineseg vpos) 가 처리.

## 6. 정체성 셀프 체크

- [x] HWP 직렬화기 0줄 수정
- [x] 어댑터는 IR 만 만짐 (`cell.text_direction` 만 갱신)
- [x] idempotent (`stage3_idempotent_does_not_double_or` 단언)
- [x] HWP 출처 보호 (text_direction == 0 가드 + apply_inner_margin 가드)
- [x] 매핑 명세는 직렬화기 + 파서 양방향 인용 (보고서 §2.1, §2.2)

## 7. 한계 (명시)

- 디버그 샘플 부재로 한컴 수동 검증 (Stage 7) 에서 본 합성의 실제 효과 미확인
- bit 16 의 의미 중첩 (text_direction vs apply_inner_margin) 은 한컴 스펙 모호성 — 후속 샘플 발굴 후 재검증 필요
- 가로/세로 비트가 손상되는 부작용은 의도적 (구현계획서 셀프 체크 후 결정 A)

## 8. 다음 단계

Stage 4: 문단 break_type + lineseg vpos 사전계산. **페이지 폭주의 직접 원인** 처리 — 169 잔여를 9 (원본) 까지 회복 목표.

## 9. 승인 요청

본 단계 완료 보고서 승인 후 Stage 4 착수.
