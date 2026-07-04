# Task M100 #1471 최종 결과 보고서

- 이슈: #1471 `onDeterminingFilename 인터셉터가 다른 확장의 chrome.downloads.download({filename}) 하위폴더 저장을 무효화`
- 브랜치: `local/task1471`
- 작성일: 2026-06-22
- 상태: 구현 및 검증 완료, 작업지시자 최종 승인 대기

## 1. 요약

rhwp Chrome 확장이 `chrome.downloads.onDeterminingFilename` 리스너를 상시 등록한 것만으로도 다른 확장의
`chrome.downloads.download({ filename })` 경로 결정에 개입하는 문제가 확인되었다.

Mac Chrome 최소 재현에서 rhwp ON 상태는 `Downloads/<blob-uuid>.pdf`로 저장되었고, rhwp OFF 상태는 요청한
`Downloads/Privacy Filter/Filtered/.../filtered_test.pdf` 경로로 정상 저장되었다.

수정은 Chrome 확장이 filename 결정 단계에서 완전히 빠지도록 `onDeterminingFilename` 리스너를 제거하고,
`downloads.onCreated` / `downloads.onChanged` 기반 관찰자로 전환하는 방식으로 진행했다.

## 2. 변경 내용

| 파일 | 변경 |
|------|------|
| `rhwp-chrome/sw/download-interceptor.js` | `onDeterminingFilename` 제거, `onCreated`/`onChanged` 관찰자 전환 |
| `rhwp-chrome/sw/download-interceptor.test.mjs` | Chrome downloads API mock 테스트 6개 추가 |
| `mydocs/plans/task_m100_1471.md` | 수행계획서 |
| `mydocs/plans/task_m100_1471_impl.md` | 구현계획서 |
| `mydocs/working/task_m100_1471_stage1.md` | Stage 1 보고서 |
| `mydocs/working/task_m100_1471_stage2.md` | Stage 2 보고서 |
| `mydocs/working/task_m100_1471_stage3.md` | Stage 3 보고서 |
| `mydocs/orders/20260622.md` | 오늘할일 상태 갱신 |

## 3. 동작 변화

### 수정 전

- rhwp Chrome 확장은 `onDeterminingFilename` 리스너를 등록했다.
- 비-HWP에서는 `suggest()`를 직접 호출하지 않았지만, 리스너 등록 자체가 Chrome filename 결정에 전역 영향을 주었다.
- 다른 확장의 blob PDF 다운로드가 요청 filename을 잃고 blob UUID 파일명으로 저장되었다.

### 수정 후

- rhwp Chrome 확장은 `onDeterminingFilename` 리스너를 등록하지 않는다.
- 비-HWP 다운로드는 filename 결정 단계에서 완전 무개입이다.
- HWP/HWPX 다운로드는 `onCreated`/`onChanged` 관찰자로 감지해 기존 자동 열기 흐름을 유지한다.
- `file://` HWP 중복 다운로드 억제는 HWP/HWPX로 판정된 경우에만 best-effort로 유지한다.

## 4. 검증

### 자동 테스트

| 명령 | 결과 |
|------|------|
| `node --test rhwp-shared/sw/download-interceptor-common.test.js` | 26 passed |
| `node --test rhwp-chrome/sw/download-interceptor.test.mjs` | 6 passed |
| `rg "onDeterminingFilename\\.addListener\|chrome\\.downloads\\.onDeterminingFilename" rhwp-chrome` | 결과 없음 |

### 빌드

| 명령 | 결과 |
|------|------|
| `cd rhwp-chrome && node build.mjs` | 통과 |
| `rg "onDeterminingFilename\\.addListener\|chrome\\.downloads\\.onDeterminingFilename" rhwp-chrome/dist` | 결과 없음 |

빌드 중 Vite의 기존 warning은 출력되었으나 빌드는 성공했다.

### 수동 검증

수정된 rhwp 테스트 빌드를 Mac Chrome에 압축해제 로드한 뒤 재현용 상대 확장에서 blob PDF 다운로드를 실행했다.

```text
[오후 8:17:49] request: Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test.pdf
[오후 8:17:49] complete: /Users/melee/Downloads/Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test (2).pdf
```

수정 전 실패 패턴인 `Downloads/<blob-uuid>.pdf`가 사라지고 요청한 하위폴더 경로가 유지되었다.

## 5. PR 제외 산출물

다음은 재현/검증용 로컬 산출물이며 PR에 포함하지 않는다.

- `output/poc/issue1471-download-repro-extension/`
- `rhwp-chrome/dist/`

테스트 빌드 준비 중 임시로 만든 `pkg`, `rhwp-chrome/node_modules`, `rhwp-studio/node_modules` symlink는 정리했다.

## 6. 남은 운영 작업

- 작업지시자 최종 승인
- PR #1480 최신 head 기준 GitHub Actions 통과 확인
- collaborator self-merge 후보 review 문서 PR diff 포함 확인
- GitHub 이슈 #1471에 원인/수정 방향/검증 결과 코멘트 등록
- PR merge 후 이슈 close는 작업지시자 승인 후 수행
