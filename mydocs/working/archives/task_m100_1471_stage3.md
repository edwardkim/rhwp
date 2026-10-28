# Task M100 #1471 Stage 3 완료 보고서

- 이슈: #1471 `onDeterminingFilename 인터셉터가 다른 확장의 chrome.downloads.download({filename}) 하위폴더 저장을 무효화`
- 브랜치: `local/task1471`
- 작업일: 2026-06-22
- 단계: Stage 3 — 빌드와 Mac Chrome 수동 재현 검증

## 1. 자동 검증

Stage 2에서 추가한 테스트와 기존 공통 판정 테스트를 다시 실행했다.

| 명령 | 결과 |
|------|------|
| `node --test rhwp-shared/sw/download-interceptor-common.test.js` | 26 passed |
| `node --test rhwp-chrome/sw/download-interceptor.test.mjs` | 6 passed |

## 2. Chrome 테스트 빌드

수정된 Chrome 확장을 압축해제 로드할 수 있도록 `rhwp-chrome/dist` 테스트 빌드를 생성했다.

| 명령 | 결과 |
|------|------|
| `cd rhwp-chrome && node build.mjs` | 통과 |
| `rg "onDeterminingFilename\\.addListener|chrome\\.downloads\\.onDeterminingFilename" rhwp-chrome/dist` | 결과 없음 |

빌드 중 Vite의 기존 경고가 출력되었지만 빌드는 완료되었다.

## 3. Mac Chrome 수동 재현 검증

작업지시자가 수정된 rhwp 테스트 빌드와 재현용 상대 확장을 Chrome에 압축해제 로드한 뒤 `Download 1`을 실행했다.

검증 로그:

```text
[오후 8:17:49] Ready. Compare with rhwp enabled and disabled.
[오후 8:17:49] request: Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test.pdf
[오후 8:17:49] blob: blob:chrome-extension://mjimojgmodpggnfbinamgnbeefamkmdd/017d391e-5695-49f2-9c0e-57b0f94c71ad
[오후 8:17:49] download id: 104
[오후 8:17:49] complete: /Users/melee/Downloads/Privacy Filter/Filtered/Session_2026-06-22_1434/filtered_test (2).pdf
```

기존 실패 패턴은 `Downloads/<blob-uuid>.pdf`였으나, 수정 빌드에서는 요청한 하위폴더 경로가 유지되었다.

따라서 #1471 최소 재현은 수정 빌드에서 해소되었다.

## 4. 임시 산출물 정리

테스트 빌드 준비를 위해 임시로 만든 untracked symlink는 정리했다.

- `pkg`
- `rhwp-chrome/node_modules`
- `rhwp-studio/node_modules`

`rhwp-chrome/dist`와 `output/poc/...`은 `.gitignore` 대상이며 PR에 포함하지 않는다.

## 5. 남은 작업

다음 Stage 4에서 보고서와 최종 정리를 진행한다.

- 최종 결과 보고서 작성
- 오늘할일 상태 갱신
- 필요 시 이슈 코멘트/PR 본문 초안 정리
- 최종 검증 명령 재확인 후 커밋 준비

Stage 4 승인 전에는 최종 정리와 커밋 단계로 진행하지 않는다.
