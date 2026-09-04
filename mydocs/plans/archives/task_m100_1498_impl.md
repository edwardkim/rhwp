# Task M100 #1498 구현계획서 — 확장 다운로드 관찰자 신선도 가드

- 이슈: #1498
- 브랜치: `local/task1498`
- 작성일: 2026-06-24
- 수행계획서: `mydocs/plans/task_m100_1498.md`

## 구현 개요

`onCreated` 로 관측한 다운로드 id 집합 `seen` 을 도입한다. `onChanged` 의 재판정 경로를
`seen` 에 있는 id 에 한해서만 수행하여, service worker 기동 이전의 과거 다운로드 항목이
뷰어로 열리지 않게 한다. chrome/firefox 동일 적용.

핵심 불변식: **onChanged 단독(= onCreated 미관측)으로는 절대 openViewer 를 호출하지 않는다.**
onCreated 는 새 다운로드에만 발화하므로 과거 기록 항목은 seen 에 없다.

---

## 1단계 — chrome 신선도 가드

**대상**: `rhwp-chrome/sw/download-interceptor.js`

- `const seen = new Set();` 추가 (onCreated 로 관측한 id).
- `onCreated`: `seen.add(item.id)` 후 기존 `processDownloadItem(item)` 호출.
- `onChanged` 재판정 조건을 `seen.has(delta.id)` 로 게이트:
  ```js
  if (seen.has(delta.id) && !handled.has(delta.id) && shouldRecheckDownload(delta)) {
    const [item] = await chrome.downloads.search({ id: delta.id });
    processDownloadItem(item);
  }
  ```
  → onCreated 를 거치지 않은(과거) 항목은 search/openViewer 경로에 진입하지 않는다.
- cleanup: 완료/종료 시 `handled` 와 함께 `seen` 도 정리(메모리 누수 방지).

**완료 기준**: 빌드 통과. 과거 항목(onCreated 미발화) onChanged → 뷰어 미오픈, 새 항목 정상.

## 2단계 — firefox 동일 적용 + SW mock 테스트

**대상**: `rhwp-firefox/sw/download-interceptor.js` (동일 `seen` 가드)

- chrome 과 동형으로 `seen` 도입, `onChanged` 재판정을 `seen.has(id)` 로 게이트.

**테스트**: `rhwp-chrome/sw/download-interceptor.test.mjs` 에 신선도 가드 케이스 추가:
- 과거 항목 시나리오: onCreated 없이 onChanged(filename 확정)만 발화 → `openViewer` 미호출 + `search` 호출 안 함(또는 호출돼도 미오픈).
- 새 항목 시나리오: onCreated 발화(판정 불가) → onChanged 재판정 → `openViewer` 호출.
- 기존 케이스(onCreated 즉시 판정, file:// 억제 등) 회귀 없음.

**완료 기준**: SW 테스트 통과(신규 + 기존), firefox 빌드 통과.

## 3단계 — 최종 검증 + 보고서

- chrome/firefox 확장 빌드(`npm run build`) 통과.
- 최종 보고서, 오늘할일(#1498) 갱신.
- (별개) 확장 재패키징·버전(0.2.7 여부)은 릴리즈 단계에서 결정.

**완료 기준**: 빌드 통과 + 보고서/오늘할일 커밋.

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `rhwp-chrome/sw/download-interceptor.js` | `seen` 집합 + onChanged 게이트 (~10줄) |
| `rhwp-firefox/sw/download-interceptor.js` | 동일 (~10줄) |
| `rhwp-chrome/sw/download-interceptor.test.mjs` | 신선도 가드 케이스 |
| `mydocs/working/task_m100_1498_stage{N}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_1498_report.md` | 최종 보고서 |

## 위험 / 주의

- onCreated 가 발화하지 않는 정상 경로가 있는지 확인(있다면 startTime 보조 가드 필요).
  일반적으로 새 다운로드는 항상 onCreated 발화 → seen 기반으로 충분.
- 버전 정책: 0.2.6 은 미배포(스토어 업로드 전)이므로, 본 정정을 0.2.6 에 포함해 재패키징할지
  0.2.7 로 올릴지는 릴리즈 단계에서 작업지시자와 결정.
