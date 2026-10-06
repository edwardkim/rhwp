# Task M100 #1498 1단계 완료보고서 — chrome 신선도 가드

- 이슈: #1498
- 브랜치: `local/task1498`
- 작성일: 2026-06-24
- 단계: 1/3

## 변경 내용

`rhwp-chrome/sw/download-interceptor.js`:

1. `const seen = new Set();` 추가 — onCreated 로 관측한(SW 기동 이후 새로 시작된) 다운로드 id.
2. `onCreated`: `seen.add(item.id)` 후 기존 `processDownloadItem(item)` 호출.
3. `onChanged` 재판정 게이트: `seen.has(delta.id)` 조건 추가.
   - onChanged 단독(= onCreated 미관측, 과거 기록 항목)은 `search`/`openViewer` 경로에 진입하지 않는다.
4. cleanup: 종료 시 `handled` 와 함께 `seen` 도 정리(메모리 누수 방지).

## 핵심 불변식

onChanged 단독으로는 openViewer 를 호출하지 않는다. onCreated 는 새 다운로드에만
발화하므로 과거 기록 항목은 seen 에 없어 자동 제외된다.

## 검증

- `node --check`: 구문 OK.
- 기존 chrome SW 테스트: **6 passed / 0 failed** (회귀 없음).
  - "filename finalized in onChanged is rechecked" 통과 = onCreated 선행(seen 등록) 시나리오 정상.

## 다음 단계

2단계: firefox 동일 적용 + 과거 항목/새 항목 신선도 가드 테스트 추가.
