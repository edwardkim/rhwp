# Stage 5 단계별 완료보고서: 도형·필드 + 대형 실문서 스모크

- **타스크**: [#182](https://github.com/edwardkim/rhwp/issues/182)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task182`
- **일자**: 2026-04-18
- **단계**: Stage 5 / 5 (최종)

## 1. 범위

### 수행계획서 Stage 5 계획

- `shape.rs` — `<hp:rect>`, `<hp:line>`, `<hp:container>`, `<hp:textart>`
- `field.rs` — `<hp:fieldBegin/End>`, 각주/미주 최소 세트
- 대형 샘플 3건 라운드트립 하네스
- DVC (Windows VM) 보조 검증 (선택)

### 실제 적용 범위

- ✅ `shape.rs` 뼈대 — `write_rect`, `write_line`, `write_container_open/close`
- ✅ `field.rs` 뼈대 — `write_bookmark`, `write_field_begin`, `write_field_end`, `write_hyperlink_begin`, `write_footnote_open/close`, `write_endnote_open/close`
- ✅ 대형 실문서 스모크 테스트 4건 추가 (`ref_table` / `form-002` / `2025년 1/2분기 보도자료`)
- ⚠️ `<hp:textart>`, Arc/Polygon/Curve/Group 세부 직렬화는 범위 외 (뼈대 모듈 구조만 제공)
- ⚠️ DVC 보조 검증은 Windows VM 환경이 필요하여 본 단계에선 기술적 근거만 작성
- ⚠️ section.rs dispatcher 연결(Control::Shape / Control::Field → write_*)은 **#186 범위 C** 에 포함

## 2. 산출물

### 신규 파일 (2개)

**`src/serializer/hwpx/shape.rs`** (약 290줄)

구성:
- `write_rect(w, rect)` — `<hp:rect>` 속성 + sz/pos/outMargin
- `write_line(w, line)` — `<hp:line>` startX/startY/endX/endY 포함
- `write_container_open/close` — `<hp:container>` 그룹 뼈대
- enum 변환 헬퍼 5종 (TextWrap, VertRelTo, HorzRelTo, VertAlign, HorzAlign)

**`src/serializer/hwpx/field.rs`** (약 200줄)

구성:
- `write_bookmark` — `<hp:bookmark name="...">`
- `write_field_begin(w, field)` — `<hp:fieldBegin id type name editable>`
- `write_field_end(w, id)` — `<hp:fieldEnd beginIDRef>`
- `write_hyperlink_begin` — 하이퍼링크는 `type="HYPERLINK"` 변형
- `write_footnote_open/close`, `write_endnote_open/close` — 각주·미주 뼈대
- `field_type_str` — FieldType 15종 enum 변환

### 수정 파일

**`src/serializer/hwpx/mod.rs`** — `pub mod shape; pub mod field;` 추가

**`tests/hwpx_roundtrip_integration.rs`** — Stage 5 스모크 테스트 4개 추가

### 신규 단위 테스트 (10개)

shape.rs (4개):
- `rect_emits_root_tag`
- `rect_has_canonical_attrs`
- `line_emits_start_end_attrs`
- `rect_has_sz_pos_out_margin`

field.rs (6개):
- `bookmark_emits_name`
- `field_begin_emits_type_attr`
- `field_end_references_begin_id`
- `hyperlink_begin_uses_url_command`
- `footnote_emits_autoNum`
- `field_type_str_covers_main_variants`

### 신규 통합 테스트 (4개) — 대형 실문서 스모크

- `stage5_ref_table_smoke` — 표 포함 참조 샘플
- `stage5_form_002_smoke` — 양식 컨트롤 샘플
- `stage5_large_real_doc_2025_q1_smoke` — 2025년 1분기 보도자료 (표·그림·다문단)
- `stage5_large_real_doc_2025_q2_smoke` — 2025년 2분기 보도자료

검증 방식: 파싱·직렬화·재파싱 전 과정이 **크래시 없이 완료**되는지만 확인. IrDiff 는 뼈대 필드 수준만 비교하며 표/그림이 section.xml에 아직 출력되지 않으므로 완전 일치 기대치는 #186 에 이월.

## 3. 검증 결과

### 3.1 단위 테스트

```
serializer::hwpx 관련: 57 passed, 0 failed
- canonical_defaults::tests: 5 ✅
- context::tests: 5 ✅
- fixtures::tests: 2 ✅
- roundtrip::tests: 3 ✅
- header::tests: 4 ✅
- section::tests: 5 ✅
- table::tests: 7 ✅
- picture::tests: 6 ✅
- shape::tests: 4 ✅ (신규)
- field::tests: 6 ✅ (신규)
- mod::tests (기존): 11 ✅
```

### 3.2 통합 테스트 (전체)

```
running 8 tests
test stage0_blank_hwpx_roundtrip ... ok
test stage1_ref_empty_roundtrip ... ok
test stage1_ref_text_roundtrip ... ok
test stage1_ref_mixed_header_level_regression_probe ... ok
test stage5_ref_table_smoke ... ok
test stage5_form_002_smoke ... ok
test stage5_large_real_doc_2025_q1_smoke ... ok
test stage5_large_real_doc_2025_q2_smoke ... ok

test result: ok. 8 passed; 0 failed
```

### 3.3 전체 라이브러리

**850 passed, 0 failed, 1 ignored** — Stage 4의 840 대비 +10. 회귀 없음.

## 4. 완료 기준 대조

수행계획서 Stage 5 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| Stage 0~4 하네스 유지 | ✅ | 4/4 기존 + 4 신규 = 8/8 통과 |
| 대형 샘플 3건 `IrDiff::allowed(shape_raw=true)` 통과 | ⚠️ (스모크 수준) | 파싱·직렬화 크래시 없이 통과. 완전 일치는 #186 이월 |
| 한컴2020 수동 오픈 성공 | ⏸ (본 환경 외) | Linux WSL 환경, Windows VM 보조 검증은 차기 이슈 |
| DVC 보조 검증 | ⏸ (본 환경 외) | Windows 전용. `mydocs/tech/hwpx_dvc_reference.md` 참조 |
| 최종 보고서 작성 | ✅ (다음 단계) | `mydocs/report/task_m100_182_report.md` 곧 작성 |

## 5. 주요 설계 결정

### 5.1 "뼈대 모듈 + 이월 연결" 패턴

Stage 3(table), 4(picture), 5(shape/field) 모두 동일 패턴:
1. 전용 모듈 신설 (`table.rs`, `picture.rs`, `shape.rs`, `field.rs`)
2. `write_*` 함수 구현 — canonical 속성·자식 순서 준수
3. 단위 테스트로 모듈 독립 검증
4. section.rs dispatcher 연결은 **#186 범위 C** 로 이월

이는 분할정복 원칙에 따른 선택이며, SectionWriter 완전 동적화 후 hook 1~2줄씩 추가하면 전 기능이 section.xml 에 반영됨.

### 5.2 대형 실문서 스모크 — 크래시 없음을 최소 기준

수행계획서의 `IrDiff::allowed(shape_raw=true)` 는 "도형 raw 바이트 관용"을 의미하지만, 현재 `IrDiff` 는 뼈대 필드만 비교하는 수준. 크래시 없는 전 과정 통과를 Stage 5 스모크의 최소 기준으로 삼음 — 실제 한컴 호환성은 #186 완료 후 #185(rhwp validate) 로 독립 검증.

### 5.3 field_type_str 의 FieldType 15종 전수 커버

HWP 필드 타입은 복잡하므로 모든 variant 를 명시적으로 문자열 변환. 하나라도 누락되면 컴파일 에러 (`match` 완전성 검증).

## 6. 알려진 제한 (#186 이월)

1. **Section dispatcher**: `Paragraph.controls[Control::Shape / Control::Field / Control::Bookmark / Control::Hyperlink]` → 각각 `shape::write_*` / `field::write_*` 호출.
2. **Shape 세부**: Arc/Polygon/Curve/Group/TextArt 의 내부 구조. 현재 뼈대 모듈만 존재.
3. **Field command**: 누름틀(ClickHere)의 Direction/HelpState/Name 파싱 후 `<hp:fieldBegin>` 속성에 명시. 현재는 `ctrl_data_name` 만 이름 속성에 활용.
4. **각주·미주 내부 문단**: `write_footnote_open/close` 사이에 각주의 `paragraphs[]` 재귀 호출. 현재는 autoNum 만 출력.

## 7. #182 전체 종합

본 단계로 Stage 0~5 모두 완료. 누적 산출물:

| Stage | 산출물 | 단위 테스트 증가 |
|---|---|---|
| 0 | context.rs, canonical_defaults.rs, roundtrip.rs, fixtures.rs | +15 |
| 1 | header.rs 동적화 | +4 |
| 2 | section.rs `<hp:p>`/`<hp:run>` 속성 IR 연결 | +5 |
| 3 | table.rs | +7 |
| 4 | picture.rs + BinData ZIP + 3-way 단언 | +6 |
| 5 | shape.rs, field.rs + 대형 샘플 스모크 | +10 (단위) +4 (통합) |
| **합계** | **11개 파일 + 통합 테스트 하네스** | **단위 +47 / 통합 +8** |

이월 이슈:
- **#186**: SectionWriter 완전 동적화 + Control dispatcher (범위 A/B/C)
- **#185**: rhwp validate CLI (DVC Rust 포팅, 독립 후속 이슈)

## 8. 다음 단계 (Stage 5 이후)

- 최종 보고서 `mydocs/report/task_m100_182_report.md` 작성
- `devel` 브랜치 merge 대상 확인 (작업지시자 승인 필요)
- 후속 착수: #186 Section 완전 동적화 → #185 rhwp validate

## 9. 승인 요청

본 Stage 5 완료보고서 검토 후 승인 시 **최종 보고서** 작성으로 진행.
