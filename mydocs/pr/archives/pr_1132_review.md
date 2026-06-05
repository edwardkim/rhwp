# PR #1132 검토 보고서

- PR: `#1132`
- 제목: `fix(chrome): 로컬 file:// HWP 열기 시 권한 안내 + 중복 다운로드 억제 (#1131)`
- 기여자: `the0807`
- 대상 이슈: `#1131`
- 검토일: 2026-05-26

## 1. 검토 결론

권장안: **체리픽 수용**.

코드 변경 방향은 이슈 #1131의 원인과 맞다. Chrome 확장에서 `file://` 문서를 열 때
파일 URL 접근 권한이 꺼져 있으면 일반 `Failed to fetch` 대신 권한 안내를 표시하고,
로컬 파일을 뷰어로 여는 과정에서 중복 다운로드 항목을 best-effort로 제거한다.

단, PR 브랜치는 현재 `devel`보다 오래된 지점에서 갈라져 있으므로 직접 merge보다
현재 `local/devel` 위로 cherry-pick하는 방식이 안전하다.

## 2. 코드 검토

### `rhwp-studio/src/main.ts`

- `loadFromUrlParam()`의 실패 경로에서 `file://` + 파일 URL 접근 권한 미허용을 분리한다.
- `chrome.extension.isAllowedFileSchemeAccess()`는 현재 `@types/chrome` 기준 Promise 호출이 가능하다.
- API가 없거나 예외가 발생하면 `null`로 폴백하고 기존 `showLoadError()`를 유지한다.
- 권한 미허용인 경우 토스트에 원인과 설정 열기 액션을 제공한다.

판정: 코드 방향 적합.

### `rhwp-chrome/sw/download-interceptor.js`

- 원격 HWP/HWPX 다운로드는 기존처럼 `suggest({ filename })`를 유지한다.
- 로컬 `file://` HWP/HWPX는 뷰어를 열고 `chrome.downloads.cancel()` + `erase()`로 다운로드 항목을 정리한다.
- 실패해도 뷰어 동작에는 영향이 없도록 예외를 무시한다.

판정: 이슈 목적에 적합. 다운로드 억제는 Chrome 동작상 best-effort라는 한계를 문서에 명시해 두었다.

## 3. 검증 결과

PR 커밋 4개를 현재 `local/devel` 기준 임시 worktree에 cherry-pick해 검증했다.

```text
git cherry-pick a8f4f8b4 07085d99 fe13f2ee 1bc9045a
```

충돌: 없음.

실행 검증:

```text
node --check rhwp-chrome/sw/download-interceptor.js
node --test rhwp-shared/sw/download-interceptor-common.test.js
npx tsc --noEmit
npm run build        # rhwp-studio
npm run build        # rhwp-chrome
```

결과:

```text
success
```

GitHub Actions:

```text
no checks reported on the 'feature/1131-file-url-access-guidance' branch
```

## 4. 주의 사항

PR에 포함된 작업 문서 파일명은 현재 저장소 규칙과 맞지 않았다.

PR 파일:

```text
mydocs/plans/task_m100_1131.md
mydocs/plans/task_m100_1131_impl.md
mydocs/working/task_m100_1131_stage1.md
mydocs/working/task_m100_1131_stage2.md
mydocs/working/task_m100_1131_stage3.md
mydocs/report/task_m100_1131_report.md
```

저장소 규칙:

```text
task_{milestone}_{issue}.md
milestone = m{number}
```

따라서 수용 과정에서 `m07x` 파일명을 `task_m100_1131...` 형식으로 정리한다.

이번 PR은 첫 기여자의 PR이므로, 처리 코멘트에는 환영 메시지를 함께 남긴다.

## 5. PR 코멘트 초안

```text
the0807님, rhwp 첫 PR 감사합니다. 환영합니다!

검토 결과 #1131에서 보고된 `file://` 권한 안내와 중복 다운로드 억제 방향이 이슈 원인과 잘 맞습니다.
로컬 통합 검증에서도 cherry-pick 충돌 없이 적용되었고, 관련 JS 문법 검사, 공유 판정 테스트,
rhwp-studio/rhwp-chrome 빌드가 통과했습니다.

다만 저장소 내부 작업 문서 파일명은 `task_m{number}_{issue}` 규칙을 사용하고 있어,
maintainer 쪽에서 cherry-pick 시 문서명만 `task_m100_1131...` 형식으로 정리했습니다.
좋은 기여 감사합니다.
```

## 6. 권장 절차

```text
1. 사용자 승인
2. `local/devel`에 PR 커밋 cherry-pick
3. 작업 문서 파일명 정리 또는 제외
4. 검증: node check/test, rhwp-studio build, rhwp-chrome build
5. 커밋
6. PR #1132에 환영/수용 코멘트 작성
7. PR #1132 및 issue #1131 정리
```
