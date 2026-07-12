# PR #2057 리뷰 — 표/셀 속성 변경 undo 기록

- PR: #2057 `Task #2053: 표/셀 속성·셀 테두리/배경 다이얼로그를 편집 라우터 snapshot 으로 기록`
- URL: https://github.com/edwardkim/rhwp/pull/2057
- 기준 브랜치: `devel`
- head branch: `fix/table-props-undo`
- 작성자: @lpaiu-cs
- 관련 이슈: #2053
- 문서 작성 시점 참고값: 원 PR head `2284be8e0b4ef084363a27ba0ade85538a1b734b`, merge state `BEHIND`
- 처리 경로: 여러 PR 체리픽 누적 검토. 통합 PR #2062 에 기능 커밋을 포함하고, 본 review 문서를 같은 PR head 에 포함한다.
- 최종 merge 조건: 통합 PR #2062 최신 head 기준 GitHub Actions 통과 + 작업지시자 승인.

## 결론

merge 후보로 본다. 다이얼로그가 `wasm.setCellProperties`/`setTableProperties`/`setCellZoneProperties` 를 직접
호출해 undo stack 에 기록되지 않던 문제를 `executeOperation({ kind: 'snapshot', operationType: 'objectProps' })`
경로로 보낸다. 기존 그림 속성 다이얼로그의 편집 라우터 패턴과 일치하며, 실패 시 수동 복구가 어려운 표/셀
속성 일괄 변경에 필요한 수정이다.

원 PR 은 `BEHIND` 상태라 #2058, #2059 와 함께 `upstream/devel` 기준 체리픽 통합 PR #2062 로 처리한다.

## 변경 범위

- `rhwp-studio/src/ui/table-cell-props-dialog.ts`
- `rhwp-studio/src/ui/cell-border-bg-dialog.ts`
- `rhwp-studio/src/command/commands/table.ts`
- `rhwp-studio/src/command/commands/format.ts`
- `rhwp-studio/tests/table-props-undo.test.ts`

## 체리픽 기록

| 항목 | 값 |
|------|----|
| 원 커밋 | `2284be8e0b4ef084363a27ba0ade85538a1b734b` |
| 통합 PR 커밋 | `75ec1b50ef996a94b9562623b642143b0fa18082` |
| 체리픽 순서 | 1 / 3 |
| 충돌 | 없음 |
| 선행 PR 의존 | 없음. #2058/#2059 와 같은 통합 PR 에 포함 |

## 검증

| 검증 | 결과 |
|------|------|
| 원 PR GitHub Actions | CodeQL/CI/Render Diff 계열 통과. WASM Build 는 원 PR CI 에서 skip |
| `cd rhwp-studio && npm test` | 통과, 181 passed |
| `cd rhwp-studio && npm run build` | 통과 |
| 실제 앱 검증 | 새 문서에 표 생성 후 cell/table props 변경, undo/redo 복원 확인 |

실제 앱 검증 관측값:

```text
cell paddingLeft: 510 -> 870 -> undo 510 -> redo 870
table treatAsChar: false -> true -> undo false -> redo true
```

앱 로드 증적:

- `mydocs/pr/assets/pr_2057_rhwp_studio_loaded.png`

## visual sweep 판단

이 PR 은 렌더링 결과나 페이지네이션을 직접 바꾸는 PR 이 아니라 편집 히스토리 기록 경로를 수정한다. 따라서
한컴 기준 PDF visual sweep 은 필수 대상이 아니다. 대신 실제 `localhost:7700` 앱에서 `window.__inputHandler`
경유 undo/redo 동작을 확인했다.

## merge 후 처리

- #2062 merge 후 #2053 auto-close 여부를 확인한다.
- 원 PR #2057 에 통합 PR #2062 로 처리됐음을 남기고 close 한다.
