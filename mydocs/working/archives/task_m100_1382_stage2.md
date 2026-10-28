# Task M100 #1382 — 2단계 완료 보고서 (파서 + serializer 2축 정정)

- 브랜치: `local/task1382`
- 작성일: 2026-06-13
- 수정 파일: `src/parser/hwpx/section.rs` (2a + 테스트 2종),
  `src/serializer/hwpx/section.rs` (2b + 테스트 3종)

## 1. 구현 내용

### 2a. 파서 — char_shapes 경계 축 정정

- `calc_utf16_len_from_parts`의 8유닛 분기에 `"\u{0012}"` 추가 (placeholder 포함
  8유닛 — offsets 축 정합). 동류 비일관 토큰 1:1 대조 결과 `\u{0012}` 단 1건
  (탭 8/서로게이트 2/일반 1 처리는 양 함수 동일).
- `char_count`는 offsets 축 산출(`utf16_pos + 1`)이라 본 정정의 영향 없음 —
  char_shape_changes 위치만 정밀 변경.

### 2b. serializer — 슬롯 시스템 autoNum placeholder 인지

- **추론** (`inferred_control_slot_count`): autoNum 1개당 두 축 잉여가 7로 측정되는
  규약을 보정 — gap 누적 후 `(gap + autonum_count) / 8`. placeholder 없는 합성
  IR(편집기 생성)은 보정 후에도 0 → 기존 mismatch 경로 유지 (회귀 없음).
- **방출** (`render_paragraph_parts` 메인 루프): AutoNumber 슬롯의 placeholder
  위치에서 ctrl 방출 + placeholder 문자 미방출 (한컴 원본 XML 동형). placeholder
  판별은 "공백 + 위치 일치 + **직후 offset +8 jump**" 3조건 — 일반 공백 오인을
  차단 (구현 중 ta-pic에서 1글자 조기 방출 검출 → jump 판별 추가로 해소).

## 2. 단위 테스트

| 테스트 | 검증 |
|--------|------|
| `task1382_calc_counts_autonum_as_8_units` (parser) | `[\u{0012}, " "]` → 9 |
| `task1382_autonum_run_boundary_on_offsets_axis` (parser) | 143E 패턴 XML → char_shapes `[(0,10),(9,11)]`, offsets `[0,8,9,10]` |
| `task1382_inference_counts_autonum_placeholder_slot` | 7-잉여 패턴 → 슬롯 1 |
| `task1382_autonum_slot_emitted_at_placeholder` | ctrl mid-text 방출 + placeholder 미방출 + 이중 방출 재발 금지 |
| `task1382_synthetic_autonum_without_placeholder_keeps_legacy_path` | 합성 IR → 추론 0 + 끝 방출 유지 |

`cargo test --lib serializer::hwpx` 175 passed / `--lib parser::hwpx` 79 passed /
`cargo fmt --check` 통과.

## 3. spot 검증 (실샘플 왕복 대칭)

임시 추적 테스트 실측 — **두 발현 샘플 완전 대칭** (text·char_count·char_offsets·
char_shapes 4축 전부 원본=재파싱):

- 143E 각주[0]: char_shapes `[(0,10),(9,11)]` 왕복 동일, offsets 8-jump 보존
- ta-pic 캡션[0]: offsets `[…3,4,12…]` 왕복 동일, RT XML이 한컴 원본 동형
  (`<hp:t>&lt;그림 </hp:t><hp:ctrl><hp:autoNum…`)

roundtrip CLI: 143E **PASS** (종전 IR_DIFF xfail) / footnote-01 PASS / ta-pic PASS
— 모두 2-round 안정 (r2=0).

## 4. 게이트 상태 (3단계 트리거 — 예측 적중)

`cargo test --test hwpx_roundtrip_baseline`에서 **가드 테스트가 설계대로 승격을
강제 중**:

```
XFAIL 샘플이 통과함: 143E433F503322BD33.hwpx — baseline 으로 승격하고 XFAIL 에서 제거하라
```

1단계 4절 예측(143E xfail 해소, 신규 xfail 0) 그대로이며, 3단계(xfail 승격)의
계획된 입력이다. **승인 즉시 3단계에서 제거·승격 처리한다** (타 테스트 3건은 통과).

## 5. 다음 단계

3단계 — 143E `XFAIL_1378_RECURSIVE` 제거 + #1387 캡션 테스트 trim_end 완화 제거·
완전 일치 승격 + 슬롯 위치 회귀 테스트.

승인 요청드립니다.
