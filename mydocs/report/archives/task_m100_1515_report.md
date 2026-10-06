# Task M100 #1515 최종 보고서 — 공통 다운로드 상태 머신 도입 및 Chrome 적용

- 이슈: #1515
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1515-download-state-machine`
- 작성일: 2026-06-24

## 1. 결론

#1513의 메모리 `seen` / `handled` 기반 긴급 완화 로직을 공통 다운로드 상태 머신과
`chrome.storage.session` 기반 adapter로 대체했다. 이제 Chrome/Edge 다운로드 관찰자는
service worker 재시작 후에도 저장된 download id 상태를 이어서 사용하고, 과거 다운로드 기록은
공통 상태 머신에서 보수적으로 무시한다.

`onDeterminingFilename`은 재도입하지 않았다. 따라서 #1471의 다른 확장 filename/subdirectory 훼손
회귀를 피하면서, #1498 계열의 과거 항목 재오픈과 동일 download id 중복 open을 더 강하게 막는다.

## 2. 변경 파일

| 파일 | 변경 |
|---|---|
| `rhwp-shared/sw/download-observer-state.js` | 브라우저 API 비의존 다운로드 상태 머신 추가 |
| `rhwp-shared/sw/download-observer-state.test.js` | 상태 머신 단위 테스트 14건 추가 |
| `rhwp-chrome/sw/download-observer-state.js` | shared 상태 머신 심볼릭 링크 |
| `rhwp-chrome/sw/download-interceptor.js` | `storage.session` 기반 Chrome adapter로 전환 |
| `rhwp-chrome/sw/download-interceptor.test.mjs` | storage mock, worker 재시작, 중복 이벤트 회귀 테스트 추가 |
| `mydocs/orders/20260624.md` | #1515 작업 기록 |
| `mydocs/plans/task_m100_1515*.md` | 수행/구현 계획서 |
| `mydocs/working/task_m100_1515_stage1.md` | 단계 완료보고서 |

## 3. 검증

| 명령 | 결과 |
|---|---|
| `node --test rhwp-shared/sw/download-observer-state.test.js` | 14 passed |
| `node --test rhwp-shared/sw/download-interceptor-common.test.js` | 26 passed |
| `node --test rhwp-chrome/sw/download-interceptor.test.mjs` | 13 passed |
| `node --check rhwp-shared/sw/download-observer-state.js` | 통과 |
| `node --check rhwp-chrome/sw/download-interceptor.js` | 통과 |
| `npm run build` (`rhwp-chrome`) | 통과 |
| `rg "onDeterminingFilename\\.addListener|chrome\\.downloads\\.onDeterminingFilename" rhwp-chrome/dist rhwp-chrome/sw` | 결과 없음 |

## 4. 수동 검증

작업지시자 환경에서 unpacked Chrome 확장으로 다음 케이스가 통과했다.

| 케이스 | 결과 |
|---|---|
| Kangnam `download.do?...` 링크 | URL 확장자 없는 다운로드 핸들러가 filename 확정 후 정상 처리 |
| GitHub raw `.hwp` 링크 | 직접 `.hwp` URL 정상 처리 |
| Chrome 재시작/확장 reload | 과거 HWP 탭 자동 재오픈 없음 |
| `autoOpen=false` | 다운로드 관찰자 경로 자동 열림 없음 |
| 같은 링크 반복 클릭 | 클릭 1회당 탭 1개 |

## 5. 영향

- Chrome/Edge 확장에서 새 HWP/HWPX 다운로드는 `onCreated` 또는 filename 확정 `onChanged` 경로로 정확히 1회 열린다.
- service worker 재시작 후에도 `chrome.storage.session`에 남은 추적 상태로 filename 확정 이벤트를 이어서 처리한다.
- 과거 완료 항목, storage 상태 없는 `onChanged` 단독 항목, 이미 handled인 download id는 뷰어를 열지 않는다.
- 비-HWP blob PDF 다운로드는 `onDeterminingFilename` 없이 관찰자 경로만 타므로 filename 결정 단계에 개입하지 않는다.
- `autoOpen=false`, `file://` HWP suppress, 대용량 경고 동작은 유지된다.

## 6. 후속

- Firefox 적용은 #1516에서 진행한다.
- #1471 blob PDF 재현 확장은 PR 리뷰/릴리스 전 필요 시 추가 확인한다.
- PR 생성 전 #1513 긴급 패치 배포 상태와 base branch를 다시 확인한다.
