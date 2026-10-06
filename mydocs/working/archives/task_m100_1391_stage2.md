# Task M100 #1391 — 2단계 완료 보고서 (모델+파서+serializer+게이트)

- 브랜치: `local/task1391`
- 작성일: 2026-06-14
- 수정 파일: `src/model/control.rs`, `src/parser/{hwpx/section,control}.rs`,
  `src/document_core/queries/field_query.rs`, `src/serializer/hwpx/{field,section,roundtrip}.rs`

## 1. 구현 내용 (결손 2축)

### 2.1 모델

- `Field`에 `raw_parameters_xml: Option<String>` 추가 (verbatim 보존).

### 2.2 파서 (parameters verbatim 캡처)

- `parse_field_parameters`가 parameters 요소 원문을 재조립(`<hp:parameters>…
  </hp:parameters>`)하여 `raw_parameters_xml`에 저장 + 기존 Command/Number 추출 병행.
  자식이 stringParam/integerParam(name 속성 + 텍스트)만이라 이벤트 재방출 안전,
  텍스트·속성값은 `escape_xml_text`로 정확히 이스케이프.
- memo subList 본문은 기존 `memo_paragraphs` 적재(무변경).

### 2.3 serializer

- `field_begin_open_tag`(field.rs) 신설: 여는 태그 문자열(`/>` 없이).
- `render_control_slot`(section.rs) Field arm: parameters 또는 (MEMO && 본문)이
  있으면 **start/end 태그** → raw_parameters 방출 → memo subList(공유 문단 경로,
  `SUB_LIST_OPEN` 실물 속성) → 닫기. 없으면 기존 `write_field_begin`(자기닫힘) 유지.

### 2.4 게이트 (`FieldContent` 동승)

- `IrDifference::FieldContent` + Display. char_shapes·linesegs 재귀에 Field arm:
  raw_parameters_xml 비교 + memo_paragraphs 문단 수·내부 재귀(`field.memo.p[k]`).

## 2. 단위 테스트

| 테스트 | 검증 |
|--------|------|
| `task1391_memo_field_emits_parameters_and_sublist` (section) | MEMO start/end + parameters verbatim + 본문 + 순서 |
| `task1391_field_without_params_keeps_empty_tag` (section) | parameters/memo 없으면 자기닫힘 유지 (회귀 방지) |
| `task1391_field_parameters_loss_in_gate` (roundtrip) | parameters 소실 검출 |
| `task1391_memo_paragraph_loss_in_gate` (roundtrip) | 본문 수 불일치 검출 |
| `task1391_aift_memo_roundtrips` (roundtrip) | 실샘플 게이트 0 |

`cargo test --lib serializer::hwpx` 226 passed / `--lib parser::hwpx` 84 passed / fmt 통과.

## 3. spot·전수 검증

### 3.1 통합 테스트 (`tests/issue_1391_memo_field_roundtrip.rs`)

- aift MEMO 2건 parameters+본문("기업 소개…") 보존 + 2-round 안정 / 143E HYPERLINK
  parameters 보존 — 2 passed.

### 3.2 전수 배치 (`output/poc/task1391/`)

- PASS 49 / **IR_DIFF 0** (게이트 동승 후에도 차이 0) / SERIALIZE_FAIL 4(#1384) /
  PARSE_FAIL 1(제외) / ROUND2_DIFF 0
- **RT 가능 파일 전수 parameters: 원본 7 → RT 7 (불일치 0)** + aift MEMO 2/2 본문 복원
  (나머지 13건 중 exam_kor 계열은 #1384 SERIALIZE_FAIL로 RT 부재)
- baseline 4 passed — **신규 xfail 0**

## 4. 부수 효과 (이슈 범위 초과 해소)

이슈는 MEMO 본문 소실이 본질이었으나, parameters는 **전 fieldBegin 타입(13건)
공통 소실**이라 verbatim 방식으로 한 번에 해소 — HYPERLINK/FORMULA/BOOKMARK
parameters도 함께 복원된다 (1단계 측정 근거).

## 5. 다음 단계

3단계 — 매뉴얼 + CI급(release-test) + 최종 보고서.

승인 요청드립니다.
