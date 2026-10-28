# Task M100 #1391 최종 보고서 — HWPX MEMO 필드 parameters·subList 직렬화

- 이슈: #1391 "HWPX serializer: MEMO 필드(fieldBegin) subList 미직렬화 — 메모 본문 소실"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1391`
- 작성일: 2026-06-14

## 1. 결함과 해소 (결손 2축)

PR #1405(physwkim, 어제 머지) 후에도 fieldBegin 태그만 보존되고 parameters·subList는
소실 잔존하던 것을 해소.

| 축 | 범위 | 종전 | 해소 |
|----|------|------|------|
| **parameters** | 전 fieldBegin 13건 공통 (MEMO/HYPERLINK/FORMULA/BOOKMARK) | IR 미보존(Command·Number만 추출), serializer 미방출 | `raw_parameters_xml` verbatim 보존 + start/end 태그 방출 |
| **MEMO 본문** | MEMO 2건 | `memo_paragraphs` 적재됐으나 미방출 | subList 공유 문단 경로 방출 |

`write_field_begin`이 `empty_tag` 자기닫힘이라 자식이 들어갈 자리가 없던 것이 본체 —
자식(parameters/memo) 있으면 start/end 태그로 전환.

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 전수 측정 (parameters 전 13건 공통 소실 확정) + verbatim 방식 확정 (구현계획서 0절) | `e7057cc1` |
| 2 | 모델+파서+serializer+게이트 + 테스트 7종 | `ec675b85` |
| 3 | 매뉴얼 + CI + 최종 보고서 | (본 커밋) |

수정 파일: `src/model/control.rs`, `src/parser/{hwpx/section,control}.rs`,
`src/document_core/queries/field_query.rs`, `src/serializer/hwpx/{field,section,
roundtrip}.rs` — 모델/파서/serializer/게이트 한정, 렌더러·레이아웃 무변경.

## 3. 검증

### 3.1 전수 배치 (`output/poc/task1391/`)

- PASS 49 / **IR_DIFF 0** (게이트 동승 후에도 차이 0) / SERIALIZE_FAIL 4(#1384) /
  PARSE_FAIL 1(제외) / ROUND2_DIFF 0
- **RT 가능 파일 전수 parameters: 7 → 7 (불일치 0)** + aift MEMO 2/2 본문 복원
  (나머지 13건 중 exam_kor 계열은 #1384 SERIALIZE_FAIL로 RT 부재 — 기존 xfail)
- baseline 4 passed — **신규 xfail 0**

### 3.2 통합 테스트 (`tests/issue_1391_memo_field_roundtrip.rs`)

- aift MEMO 2건 parameters+본문("기업 소개…") 보존 + 2-round 안정 / 143E HYPERLINK
  parameters 보존 — 2 passed.

### 3.3 CI급 검증 (release-test 프로필)

- `cargo test --profile release-test --tests` — **2317 passed, 0 failed** (기존 2310 + 신규 7: 게이트 3 + 방출 2 + 통합 2)
- `cargo fmt --check` 통과, clippy 경고 0

## 4. parameters verbatim 방식 (설계 기록)

전 fieldBegin 13건이 parameters 보유·소실이라 구조화(타입별 가변 파라미터) 대신
원문 verbatim(`raw_parameters_xml`)을 채택 — #1405의 부수 항목 verbatim 기조와
일치, 의미 비교 실익 낮음. 게이트는 문자열 비교(`FieldContent`)로 동승.
기존 Command/Number 추출(누름틀 안내문 등 렌더용)은 병행 유지.

## 5. 잔존 한계 (기지 이슈)

| 한계 | 이슈 |
|------|------|
| 표 pageBreak 일괄 TABLE 방출 | #1393 |
| borderFillIDRef SERIALIZE_FAIL 4건 | #1384 |
| hp:pic 크기 요소 IR 미반영 | #1389 |
| 열거 속성 표면 표기 정합 검사 | #1402 |
| newNum 슬롯 위치 + 143E RT 페이지 수 | #1407 |

신규 발견 없음.

## 6. 한컴 판정 (선택)

메모 본문은 편집기에서 메모 패널로 표시되는 콘텐츠이나, 본문 시각 렌더 영향은 없다
(전수 SVG 변동 없음·IR_DIFF 0). 필요 시 aift.rt를 한컴에디터에서 열어 메모 2건의
본문("기업 소개 및 본 과제 관련 기술력 소개" 등) 표시를 점검할 수 있으나 필수 아님.

## 7. 산출물

- 계획서: `mydocs/plans/task_m100_1391{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1391_stage2.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (게이트 항목 + #1391 해소)
- 검증 산출물: `output/poc/task1391/`
