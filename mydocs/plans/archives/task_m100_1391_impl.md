# Task M100 #1391 구현계획서 — HWPX MEMO 필드 subList·parameters 직렬화

- 수행계획서: `mydocs/plans/task_m100_1391.md` (승인 완료)
- 브랜치: `local/task1391`
- 작성일: 2026-06-14
- 단계: 3단계

## 0. 사전 조사 확정 (1단계 측정 완료)

### 0.1 전수 분포

- fieldBegin: FORMULA 4 / HYPERLINK 4 / MEMO 2 / BOOKMARK 1 / (type 미상) 2 — 총 13건.
- **13건 전부 `hp:parameters` 보유** — 그리고 **전부 RT에서 소실** (143E HYPERLINK
  원본 1 → RT 0 실측). parameters 소실은 MEMO 한정이 아니라 **전 fieldBegin 공통 결손**.
- MEMO 2건은 추가로 `hp:subList`(메모 본문) 보유 — 둘 다 소실.

### 0.2 결손 2축 + 보존 방식 확정

| 축 | 범위 | IR | 방식 |
|----|------|-----|------|
| **parameters** | 전 fieldBegin 13건 공통 | 미보존 (Command·Number만 추출) | **(A) verbatim** — `Field.raw_parameters_xml: Option<String>` 추가, 파서가 `<hp:parameters>…</hp:parameters>` 원문 저장, serializer 그대로 방출. 전 타입 1지점 해소 |
| **memo subList 본문** | MEMO 2건 | ✓ `memo_paragraphs` 적재됨 | serializer 방출만 (MEMO 전용) |

verbatim 채택 근거: 13건 전부 parameters 보유·소실이라 구조화(B)는 타입별 가변
파라미터 처리 부담이 크고, #1405의 "원본 부수 항목 verbatim" 기조와 일치.
파서의 기존 Command/Number 추출(렌더용)은 **유지** — raw 저장과 병행(추출값은
누름틀 안내문 등에서 사용).

### 0.3 serializer 결손 본체

- `write_field_begin`(field.rs:36) `empty_tag` 자기닫힘 → 자식 자리 없음.
  parameters(전 타입) 또는 memo subList(MEMO) 있으면 **start/end 태그로 전환**,
  없으면 기존 empty_tag 유지 (비-parameters 필드 회귀 방지).
- 호출부(section.rs:660): memo subList 문단 방출에 `ctx` 필요 →
  `write_field_begin(w, f, ctx)` 시그니처 확장 (subList 공유 경로).

### 0.4 게이트

- `IrDifference::FieldMemo`(또는 ObjectComment류) — memo subList 문단 수·내부 재귀.
- parameters는 verbatim 문자열 비교(raw_parameters_xml) 동승 — `FieldParameters` 차이.
- 비교 지점: 문단 컨트롤 재귀에 Field arm 신설.

## 1단계 — (측정 완료) 설계 확정 보고

0절이 1단계 측정 결과. 보고서에 분포·방식 확정 기재 → 승인 요청.
(코드 수정 없음)

## 2단계 — 모델+파서+serializer+게이트

### 2.1 모델

- `Field`에 `raw_parameters_xml: Option<String>` 추가.

### 2.2 파서

- `parse_field_begin`: `b"parameters"` arm에서 요소 원문(여는 태그~닫는 태그)을
  `raw_parameters_xml`에 저장 + 기존 `parse_field_parameters`(Command/Number 추출) 병행.
  (원문 캡처는 reader 위치 기반 또는 재조립 — 1지점)

### 2.3 serializer

- `write_field_begin(w, f, ctx)`:
  - parameters 또는 (MEMO && memo_paragraphs 비어있지 않음)이면 start 태그 →
    raw_parameters_xml 방출 → (MEMO면) memo subList 방출 → end 태그.
  - 아니면 기존 empty_tag.
- memo subList: `<hp:subList …>` 래퍼(실물 고정 속성) + `memo_paragraphs`를 공유
  문단 경로(render_paragraph_parts + ctx.next_para_id)로 방출.

### 2.4 게이트

- Field arm: raw_parameters_xml 비교(`FieldParameters`) + memo_paragraphs 재귀
  (char_shapes/controls/linesegs, 경로 `field.memo.p[k]`).

### 2.5 테스트

- MEMO fieldBegin: start/end + parameters verbatim + subList 본문 방출
- HYPERLINK: parameters verbatim 방출 (start/end 전환, subList 없음)
- parameters·subList 없는 필드: 기존 empty_tag 무변화
- 게이트: 본문/parameters 소실 검출
- 실샘플(aift MEMO 2건 + 143E HYPERLINK) 왕복

## 3단계 — 전수 검증 + 문서

1. `hwpx-roundtrip --batch` 전수 → `output/poc/task1391/` (parameters 13건·MEMO 본문 복원)
2. baseline + CI급 (release-test + fmt + clippy)
3. 매뉴얼 갱신 (#1391 해소 + 게이트 항목)
4. 최종 보고서

## 위험 관리

| 위험 | 단계 | 대응 |
|------|------|------|
| start/end 전환이 비-parameters 필드 회귀 | 2 | parameters/subList 유무 분기 + 기존 field 테스트 보존 |
| raw 캡처가 namespace/이스케이프 변형 | 2 | 원문 그대로 저장(재파싱 안 함), 재파싱 대칭 테스트로 보증 |
| memo para id 충돌 | 2 | ctx.next_para_id 공유 경로 (#1379/#1387 선례) |
| #1405 영역 충돌 | 3 | 머지 직전 rebase + 통합 트리 CI ([[project_pr_merge_collaborator]]) |
