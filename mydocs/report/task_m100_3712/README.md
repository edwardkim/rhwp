---
kind: report
status: active
canonical: mydocs/report/task_m100_3712/README.md
last_verified: 2026-08-01
---

# #3712 처리 기록 — changedPages (#3630 P3, 내성 4종 완결)

## 문제

에이전트는 편집 결과를 눈으로 못 본다. 어느 쪽이 바뀌었는지 표면이 지정하지
않으면 전 페이지를 렌더하거나(비용 폭발) 검증을 건너뛴다(#3630 실패 유형 3).

## 구현

- 코어 신규 pub 질의 `DocumentCore::pages_covering_paragraphs(&mut self, targets)`
  ([changed_pages.rs](../../../src/document_core/queries/changed_pages.rs)):
  - grep 페이지 인덱스와 같은 순회(PageItem 걷기)를 재사용하되, 첫 쪽만이 아니라
    **그 문단이 걸친 모든 쪽**(분할 표 PartialTable 포함)을 담는다 — 누락은 거짓
    통과를 만들지만 상위집합은 렌더 한 번 더일 뿐.
  - 대상 문단이 하나라도 커버리지에 없으면 None → 봉투 null (**부분 목록 금지**).
  - 진입 시 `paginate_if_needed()` — 편집 커맨드는 recompose 로 dirty 만 남기므로
    저장 직전 조판으로 맞춘 뒤 걷는다(#3704 세션 재조판과 같은 전제).
- 변경 문단 추적 (새 편집 로직 0):
  - fill: `collect_all_fields` 순회에서 FieldLocation(section·para)을 계수와 함께 수집
  - replace: **치환 전** grep 매치 주소 — 문자열 치환은 문단 인덱스를 밀지 않는다
  - set-cell: `resolve_table_cell` 의 호스트 문단
  - run 저널: step 합집합(fill 위치 + replace 매치 + set_checkbox n번째 □ + set_cell)
- 봉투: `edit fill-fields/replace-text/set-cell --json` + `run` 저널에
  `changedPages:[n,…]|null`. dry-run·치환 0건(무산출)은 null.
- capabilities outputFields 5곳 동기화 (fill/replace/set_checkbox/set_cell/run_plan).

## 실측 (evidence.txt 원문)

1. fill: `changedPages:[0]` — 회사명 필드가 1쪽(0 기준)에 있음.
2. replace: `changedPages:[0]` — 매치 문단 페이지.
3. dry-run: `changedPages:null` — 예측 목록으로 오인 금지.
4. run 저널: 2 step 합집합 `changedPages:[0]` + verify identical:true.

## 검증

- 신규 `changed_pages_contract` 5건 green (fill/replace/set-cell 범위 유효 + 비어
  있지 않음 / dry-run null / run 합집합).
- 무회귀: cli_json 22 · run_plan 6 · edit_verify 4 · fill 7 · replace 4 · set-cell 5.
- clippy `-D warnings` 0 · fmt clean.

## 남은 것

- 세션 편집 도구(hwp_doc_fill_fields 등)의 changedPages 는 **#3704(세션 재조판)
  머지 후 후속 적층** — 재조판 시점 규약을 #3704 와 맞춘다(이슈 본문에 명시).
- #3630 내성 4종(P1 did-you-mean · P2 --verify · P3 changedPages · P4 nextCall) 완결.
