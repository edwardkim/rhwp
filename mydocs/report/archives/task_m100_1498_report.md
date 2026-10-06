# Task M100 #1498 최종 보고서 — 확장 다운로드 관찰자 신선도 가드

- 이슈: #1498 "[Bug] 확장 0.2.6: 과거 다운로드 기록 접근으로 미사용 문서 뷰어 창 다발 열림"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1498`
- 작성일: 2026-06-24

## 1. 개요

확장 0.2.6 업데이트 후, 과거에 다운로드한 HWP/HWPX 문서들의 뷰어 창이 다수 자동으로
열리는 회귀가 발생했다. #1471/#1480 에서 다운로드 가로채기를 `onDeterminingFilename` →
`onCreated`/`onChanged` 관찰자로 전환하면서, "새로 시작된 다운로드만 처리" 필터가 누락된 것이
원인이다.

## 2. 원인

`download-interceptor.js`(chrome/firefox)의 `onChanged` 리스너가 `!handled.has(id) &&
filename 확정` 이면 `downloads.search({id})` 로 재조회하여 처리했다. service worker 재기동 시
in-memory `handled` 가 초기화되면, 브라우저에 쌓인 과거 다운로드 항목에 onChanged 가 발화할 때
onCreated 를 거치지 않았음에도 확장자/MIME 매치만으로 `openViewer` 가 호출되었다.
공통 판정 `shouldInterceptDownload` 는 확장자/MIME 만 보고 다운로드 시점(신선도)을 보지 않는다.

## 3. 변경

### `rhwp-chrome/sw/download-interceptor.js` · `rhwp-firefox/sw/download-interceptor.js`

- `seen` 집합 도입: onCreated 로 관측한 id(= SW 기동 이후 새로 시작된 다운로드).
- onChanged 재판정을 `seen.has(delta.id)` 로 게이트 → onCreated 미경유(과거 기록) 항목은
  `search`/`openViewer` 경로에 진입하지 않는다.
- 종료(complete/error) 시 `handled` 와 `seen` 모두 cleanup.

**핵심 불변식**: onChanged 단독으로는 openViewer 를 호출하지 않는다. onCreated 는 새
다운로드에만 발화하므로 과거 기록 항목은 seen 에 없어 자동 제외된다.

공통 판정 로직(`shouldInterceptDownload`)·WASM·viewer-launcher 무변경.

### `rhwp-chrome/sw/download-interceptor.test.mjs` (신규 케이스 2건)

- 과거 항목(onChanged 단독) → search/뷰어 미호출.
- 새 항목(onCreated 관측 후 onChanged 재판정) → 뷰어 정상 오픈.

## 4. 검증 결과

| 항목 | 결과 |
|---|---|
| chrome SW 테스트 (신규 2 + 기존 6) | 8 passed / 0 failed |
| firefox SW 구문 체크 | OK |
| chrome / firefox 확장 빌드 | 통과 |
| 기존 케이스(onCreated 즉시 판정·onChanged 재판정·file:// 억제) | 회귀 없음 |

## 5. 영향

- 확장에서 SW 재기동 후 과거 다운로드 항목이 뷰어로 열리지 않는다.
- 새 HWP/HWPX 다운로드의 뷰어 자동 열기는 그대로 동작한다.
- chrome/firefox 공통. 라이브러리·공통 판정 무변경.

## 6. 후속

- 본 정정의 확장 반영: 0.2.6 은 스토어 미배포 상태이므로, 재패키징 시 포함하거나 0.2.7 로
  올릴지는 릴리즈 단계에서 작업지시자와 결정.
- 실제 브라우저 동작(과거 항목 미오픈)은 작업지시자 확장 테스트로 최종 확인.

## 7. 산출물

- 수행계획서: `mydocs/plans/task_m100_1498.md`
- 구현계획서: `mydocs/plans/task_m100_1498_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1498_stage1.md`, `_stage2.md`
- 최종 보고서: 본 문서
- 소스: `rhwp-chrome/sw/download-interceptor.js`, `rhwp-firefox/sw/download-interceptor.js`,
  `rhwp-chrome/sw/download-interceptor.test.mjs`
