# 최종 결과보고서 — Task #1556

## 제목
[HWPX/직렬화] 컨트롤 슬롯 8유닛 시프트 — 실문서 char_offset 변위 (hwpx-roundtrip IR_DIFF)

## 1. 결론
HWPX serializer 의 "8유닛 시프트" 잔여 결함의 **근본 원인은 다단락(문단 경계를 넘는)
필드의 고아 `<hp:fieldEnd>`** 였다. begin/end 가 다른 문단인 누름틀(CLICK_HERE) 필드에서,
종료 마커 문단이 8유닛 슬롯을 IR 로 표현하지 못해 직렬화 시 소실됐다. 고아 fieldEnd 를
IR(`Paragraph.orphan_field_ends`)에 기록·복원하여 해소했다.

## 2. 근본 원인
- 파서(`src/parser/hwpx/section.rs`)의 `field_stack` 은 **문단 단위**로 동작한다.
  begin·end 가 같은 문단이면 `FieldRange` 로 연결되지만, 다단락 필드는 연결되지 않는다.
- **end 문단**: 고아 fieldEnd → `\u{0004}`(8유닛)만 차지하고 `Control`·`FieldRange`
  둘 다 미생성 → IR 에 8유닛 슬롯을 표현할 산출물이 전무.
- 직렬화기는 `controls` + `field_ranges` 로부터 슬롯을 복원하므로 fieldEnd 를 방출하지
  못함 → char_count/char_offset 이 8 감소.
- 대표 증거(`…dt2854`): 문단 0.16 `cc 38→30`, `cs[1].pos 37→29` (정확히 −8),
  `controls=0` (= 컨트롤 아닌 fieldEnd 슬롯), `ir-diff` 가 controls 차이 미보고와 정합.

## 3. 해결
1. **IR** (`src/model/paragraph.rs`): `OrphanFieldEnd { char_idx, begin_id_ref, field_id }`
   + `Paragraph.orphan_field_ends`.
2. **파서** (`src/parser/hwpx/section.rs`): fieldEnd 의 `beginIDRef`/`fieldid` 포착,
   `field_stack` 이 빈 fieldEnd 를 고아로 기록(`parse_field_end_attrs` 헬퍼,
   `parse_ctrl` 시그니처 확장).
3. **직렬화기** (`src/serializer/hwpx/section.rs`, `field.rs`):
   `write_field_end_full`(beginIDRef+fieldid) / `emit_orphan_field_end` 추가,
   메인 루프·post-loop·빈 문단·mismatch 경로에서 고아 fieldEnd 방출.
   **핵심**: `inferred_control_slot_count` 에서 `orphan_field_ends.len()` 도 차감해
   슬롯 추정을 정합시켜 메인 경로로 진입(누락 시 무효과).

## 4. 검증 (수용 기준 대비)

| 수용 기준 | 결과 |
|----------|------|
| 영향 실문서 `hwpx-roundtrip` IR diff=0 | ✅ 대표 4건(dt2854/2906/2952/3004) diff=1→0 |
| 회귀 가드(샘플/게이트) | ✅ 합성 roundtrip 테스트 + 실문서 `samples/hwpx/field-multipara-clickhere.hwpx` |
| `samples/hwpx/` 전수 회귀 | ✅ `hwpx_roundtrip_baseline` 무회귀 |
| 전체 `cargo test` | ✅ 무회귀 (exit 0) |

- **코퍼스 전수**: 2023-01-05 폴더 60건 `IR_DIFF 24 → 1`. 다른 날짜 폴더 표본에서도
  잔여 IR_DIFF 는 전부 동일 패턴.
- **잔여 IR_DIFF (범위 밖)**: `Picture 직렬화 실패: BinDataContent 누락` (그림 BinData
  드롭). `ir-diff` 텍스트/오프셋 비교는 0건. 이는 **#1552 F1(HWP5/그림 BinData) 계열의
  별개 결함**으로, 본 이슈(HWPX 컨트롤 슬롯 8유닛 시프트)와 무관 → 후속 분리.
- 단위 테스트 5건(`cargo test --lib task1556`): 파서 2 + 직렬화 3 (합성 roundtrip 포함).
- fmt(변경 파일 한정) clean, clippy(lib) 무경고.

## 5. 변경 파일
- `src/model/paragraph.rs` — `OrphanFieldEnd` + 필드.
- `src/parser/hwpx/section.rs` — 고아 fieldEnd 기록.
- `src/serializer/hwpx/section.rs` — 고아 fieldEnd 방출 + 슬롯 카운팅 정합.
- `src/serializer/hwpx/field.rs` — `write_field_end_full`.
- `samples/hwpx/field-multipara-clickhere.hwpx` — 회귀 샘플 (신규).

## 6. 분리/후속
- HWP5 F1(#1554)·F2'(#1555) 와 무관(본 건 HWPX serializer 전용).
- 코퍼스 잔여 그림 BinData 드롭(`Picture 직렬화 실패`)은 별도 이슈 권고.
- (편집 경로) 문단 분할 시 `orphan_field_ends` 는 `field_ranges` 와 동일하게 비움 —
  roundtrip 보존 범위 밖, 필요 시 후속.

## 7. 범위 준수
하이퍼-워터폴 절차(이슈→브랜치→수행계획서→구현계획서→단계별 구현·보고→최종 보고서)
준수. HWP3/HWP5 공통 모듈 무수정. 기능 변경과 포맷 변경 분리.
