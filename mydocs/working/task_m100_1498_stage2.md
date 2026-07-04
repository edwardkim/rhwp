# Task M100 #1498 2단계 완료보고서 — firefox 적용 + 신선도 가드 테스트

- 이슈: #1498
- 브랜치: `local/task1498`
- 작성일: 2026-06-24
- 단계: 2/3

## 1. firefox 동일 적용

`rhwp-firefox/sw/download-interceptor.js`:

- `seen` 집합 도입, onCreated 에서 `seen.add(item.id)`.
- onChanged 재판정을 `seen.has(delta.id)` 로 게이트.
- 종료 시 handled/seen cleanup.
- chrome 과 동형.

## 2. SW mock 테스트 추가

`rhwp-chrome/sw/download-interceptor.test.mjs` 신규 케이스 2건:

- **`past download (onChanged only, no onCreated) does not open the viewer`**
  - SW 재기동 후 과거 HWP 항목에 onChanged 만 발화 → `downloads.search` 미호출 + `tabsCreate`(뷰어) 미호출. **회귀 정확히 가드.**
- **`new download seen via onCreated is opened on onChanged recheck`**
  - onCreated 관측 후 onChanged 재판정 → 뷰어 정상 오픈.

## 3. 검증 결과

| 항목 | 결과 |
|---|---|
| chrome SW 테스트 (신규 2 + 기존 6) | **8 passed / 0 failed** |
| firefox SW 구문 체크 | OK |
| chrome 확장 빌드 | 통과 |
| firefox 확장 빌드 | 통과 |
| 기존 케이스(onCreated 즉시 판정, onChanged 재판정, file:// 억제) | 회귀 없음 |

## 4. 다음 단계

3단계: 최종 검증 + 보고서 + 오늘할일 갱신.
