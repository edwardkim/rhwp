# PR #1480 리뷰 기록 - Chrome download filename 인터셉터 부작용 제거

- PR: https://github.com/edwardkim/rhwp/pull/1480
- 작성일: 2026-06-22
- 작성자: collaborator self-merge 후보 경로
- 문서 작성 시점 참고 head: `e358b8fd602cd63f77f41107ab2e99d27977e73b`
- base: `devel`
- head: `postmelee:task_m100_1471`

## 1. PR 메타

| 항목 | 확인 내용 |
|------|-----------|
| 작성자 | `postmelee` |
| PR 상태 | open, draft 아님 |
| merge 상태 | 문서 작성 시점 `MERGEABLE`, `BLOCKED` |
| 관련 이슈 | `Refs #1471` |
| 규모 | 문서 작성 시점 9 files, +790 / -13 |
| 커밋 수 | 5개 + upstream merge commit + 본 self-merge review 문서 커밋 예정 |

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 이 문서는 작성 시점 값을 참고로만 기록한다.
최종 merge 판단은 merge 직전 최신 PR head 기준으로 다시 확인한다.

## 2. 문제 확인

이슈 #1471은 rhwp Chrome 확장이 `chrome.downloads.onDeterminingFilename` 리스너를 등록한 상태에서,
다른 확장의 `chrome.downloads.download({ filename })` 하위폴더 저장 경로가 무효화되는 문제다.

Mac Chrome 최소 재현 확장으로 실제 재현을 확인했다.

rhwp ON, 수정 전:

```text
[오후 7:48:34] request: Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test.pdf
[오후 7:48:34] blob: blob:chrome-extension://mjimojgmodpggnfbinamgnbeefamkmdd/5ec9ae5a-5bcb-46d8-8d0a-4f4e0d652dbd
[오후 7:48:34] complete: /Users/melee/Downloads/5ec9ae5a-5bcb-46d8-8d0a-4f4e0d652dbd.pdf
```

rhwp OFF:

```text
[오후 7:48:09] request: Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test.pdf
[오후 7:48:09] blob: blob:chrome-extension://mjimojgmodpggnfbinamgnbeefamkmdd/290d6ebf-3a82-4599-a452-4886c7ddd773
[오후 7:48:09] complete: /Users/melee/Downloads/Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test (1).pdf
```

따라서 이 문제는 HWP 판정 함수 오탐이 아니라, `onDeterminingFilename` 리스너 등록 자체가 Chrome filename
결정 절차에 전역으로 참여하는 부작용으로 판단했다.

## 3. 변경 범위

### 3.1 Chrome filename 결정 단계에서 rhwp 제거

- `rhwp-chrome/sw/download-interceptor.js`에서 `chrome.downloads.onDeterminingFilename` 리스너 등록을 제거했다.
- 비-HWP 다운로드와 다른 확장이 시작한 blob 다운로드에 대해 rhwp가 filename 결정 단계에 참여하지 않는다.
- `rhwp-chrome` 소스와 빌드 산출물 기준으로 `onDeterminingFilename.addListener` 잔여가 없음을 확인했다.

### 3.2 HWP/HWPX 자동 열기 관찰자 전환

- 기존 filename 결정 이벤트 대신 `chrome.downloads.onCreated`와 `chrome.downloads.onChanged`를 사용한다.
- 다운로드 생성 시점에 item 정보가 충분하면 즉시 `shouldInterceptDownload(item)`으로 HWP/HWPX를 판정한다.
- 생성 시점 정보가 부족한 경우 `onChanged`에서 완료 상태를 감지한 뒤 `chrome.downloads.search({ id })`로 다시 조회한다.
- 같은 다운로드 id는 `handled` set으로 한 번만 처리한다.

### 3.3 기존 동작 유지 범위

- HWP/HWPX로 판정된 다운로드는 기존 `openViewer(item.url)` 흐름을 유지한다.
- `autoOpen=false`이면 HWP/HWPX 다운로드도 열지 않는다.
- `file://` HWP/HWPX 중복 다운로드 억제는 HWP/HWPX 판정 후 best-effort로 `cancel`/`erase`를 호출하는 방식으로 유지한다.
- Chrome 확장 변경에 한정되며 Firefox/Safari download interceptor에는 변경이 없다.

## 4. 리스크

| 리스크 | 판단 |
|--------|------|
| HWP 자동 열기 타이밍 변경 | filename 결정 단계가 아니라 다운로드 생성/완료 관찰 단계에서 처리하므로 약간 늦어질 수 있다. 대신 비-HWP filename 부작용을 근본적으로 제거한다. |
| 생성 시점 item 정보 부족 | `onChanged` 완료 이벤트에서 `chrome.downloads.search({ id })` 재조회 경로를 추가하고 테스트로 고정했다. |
| 중복 openViewer 호출 | `handled` set으로 다운로드 id당 1회만 처리하며 테스트로 고정했다. |
| 비-HWP blob 다운로드 회귀 | 비-HWP blob PDF는 `openViewer`, `cancel`, `erase`를 호출하지 않는 테스트와 사용자 수동 검증으로 확인했다. |
| 로컬 file:// 억제 회귀 | HWP/HWPX로 판정된 `file://` 다운로드에서만 `cancel`/`erase`를 호출하는 테스트를 추가했다. |

## 5. 검증

로컬 자동 검증:

```bash
node --test rhwp-shared/sw/download-interceptor-common.test.js
node --test rhwp-chrome/sw/download-interceptor.test.mjs
rg "onDeterminingFilename\\.addListener|chrome\\.downloads\\.onDeterminingFilename" rhwp-chrome
cd rhwp-chrome && node build.mjs
rg "onDeterminingFilename\\.addListener|chrome\\.downloads\\.onDeterminingFilename" rhwp-chrome/dist
```

검증 결과:

- `node --test rhwp-shared/sw/download-interceptor-common.test.js`: 26 passed
- `node --test rhwp-chrome/sw/download-interceptor.test.mjs`: 6 passed
- `rg "onDeterminingFilename\\.addListener|chrome\\.downloads\\.onDeterminingFilename" rhwp-chrome`: 결과 없음
- `cd rhwp-chrome && node build.mjs`: pass
- `rg "onDeterminingFilename\\.addListener|chrome\\.downloads\\.onDeterminingFilename" rhwp-chrome/dist`: 결과 없음

사용자 수동 검증:

수정된 rhwp 테스트 빌드를 Mac Chrome에 압축해제 로드한 뒤, 재현용 상대 확장에서 blob PDF 다운로드를 실행했다.

```text
[오후 8:17:49] Ready. Compare with rhwp enabled and disabled.
[오후 8:17:49] request: Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test.pdf
[오후 8:17:49] blob: blob:chrome-extension://mjimojgmodpggnfbinamgnbeefamkmdd/017d391e-5695-49f2-9c0e-57b0f94c71ad
[오후 8:17:49] download id: 104
[오후 8:17:49] complete: /Users/melee/Downloads/Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test (2).pdf
```

수정 전 실패 패턴인 `Downloads/<blob-uuid>.pdf`가 사라지고, 다른 확장이 요청한 하위폴더 경로가 유지되었다.

GitHub Actions 작성 시점 참고값:

- Build & Test: in progress
- Analyze (javascript-typescript): pass
- Analyze (python): pass
- Analyze (rust): in progress
- WASM Build: skipped
- CodeQL: neutral

본 review 문서 커밋 push 후 GitHub Actions가 다시 실행되므로, merge 전 최신 head 기준으로 위 상태를 재확인한다.

## 6. 판단

작성 시점 기준으로 #1471의 핵심 요구인 "rhwp가 다른 확장의 blob 다운로드 filename/subdir 결정을 깨지 않아야 한다"가
PR 범위에서 해결되어 있다.

최종 조건:

1. 본 review 문서 2건이 PR head에 포함된다.
2. push 후 최신 PR head 기준 GitHub Actions가 통과한다.
3. merge 직전 `MERGEABLE` 상태를 다시 확인한다.
4. 작업지시자 승인 상태가 유지된다.

위 조건 충족 시 collaborator self-merge 후보로 merge 수용한다.
