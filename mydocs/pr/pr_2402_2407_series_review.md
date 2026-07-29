# kevin9327 studio 연작 검토 — #2402 → #2404~#2407

문서 기여(#2380/#2381)에서 studio 코드 기여로 확장한 연작 5건. 전부 독립
파일축(공유: mutation-routing-guard baseline·command.ts 2건 인접).

## #2402 — 계산식 쉼표 겹침 (closes #2367)

- 원인 정확: evaluateTableFormula(write_result=true)가 원시 결과 선기록 →
  쉼표 포맷이 offset 0 삽입으로 겹침("6,9126912"). #2344 "delete 누락"
  동형으로 delete→insert, 같은 commit() 클로저 = snapshot 원자화 유지.
- 원장 3→4 사유 주석 — 이관 연작 규율을 외부 기여자가 그대로 따름.
- 재실증: tsc OK / npm test 367/367 / red→green(원복 시 2 FAIL) / CI green.
- **merge 권고.**

(후속 4건은 순서대로 개별 검증 후 이 문서에 추가)

## #2404 — hwpctl SetCellText/GetCellText (자체 발견 2건)

Set 누적 삽입 → delete-후-insert (Set=replace 계약 메인테이너 확정), GetCellText
시그니처 불일치(항상 빈 문자열, 저장소 유일 path 형식) → 인덱스 API 정합.
red→green 2건. **merge 완료.**

## #2405·#2406 — command.ts undo 쌍 (결합 검증)

- #2405: charCount() 헬퍼(#2337 산물)의 유일 누락 지점(InsertTextCommand.undo)
  적용 — astral 문자 undo 인접 소실 해소. UTF-16 커서 관례 경계 유지.
- #2406: 3문단+ 선택 삭제 undo 분할점 미갱신 — 잠복 이유(2문단 반복 1회
  무증상) 분석 동봉.
- 결합 트리 tsc/369, red→green 4건. **둘 다 merge 완료.**

## #2407 — 표 리사이즈 hover marker 도달 불가

pageHint 유일 대입 지점 미기록 → 캐시 비교 항상 참 → marker 갱신 사문.
red→green 2건. **merge 완료 — 연작 5건 완결.**

## 총평 (2026-07-19)

kevin9327: 문서 기여 → 코드 연작 5건(전건 red→green·원장 규율·경계 명시,
자체 발견 4건) 확장 완결. 완결 코멘트 게시.

## 2차 연작 (#2410/#2411/#2413, 2026-07-19 오후) — 전건 merge

- #2410: 중첩 셀 병합 undo 길이 조회 축 불일치 (cursor.ts 동형 정합)
- #2411: 셀 안 Edit 필드 flat 호출 침묵 실패 → setFormValueInCell + record
  경로 inCell 정합 (#2375 상호작용, 원장 26→27)
- #2413: 표 구조 편집 후 리사이즈 캐시 신선도 맹점 (#2407 후속 완결)
- 결합 트리 tsc/415, red→green 9건, CI 전건 green. 하루 2사이클 8건 완결.
