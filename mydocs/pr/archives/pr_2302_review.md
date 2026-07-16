# PR #2302 검토 - #2301 Studio undo 계약 실동작 e2e

- 검토일: 2026-07-16
- 대상: [PR #2302](https://github.com/edwardkim/rhwp/pull/2302), [Issue #2301](https://github.com/edwardkim/rhwp/issues/2301)
- 작성자: `lpaiu-cs`
- 최종 검토 head: `fffba325854b431bf730e2d5ed6694f17b8e0f81`
- merge SHA: `d144badd68b48e490c0e69e94a103716d13dba45` (squash)
- 규모: 2 files, +364/-0
- reviewer: 미지정 (검토 시점)

## PR 주장과 범위

PR은 기존 Studio undo 계약 테스트 다섯 종이 소스 문자열 검사에 머물던 한계를 보완한다. 새
`rhwp-studio/e2e/undo-contracts.test.mjs`는 다음 흐름을 브라우저 런타임에서 실행하고 IR 조회로 결과를
확인한다.

1. 찾아 바꾸기 모두 바꾸기 후 `Ctrl+Z` 복원
2. 그림 속성의 `쪽 영역 안으로 제한` 변경 후 undo 복원
3. 수식 속성 글자 크기 변경 후 undo 복원
4. 표/셀 속성의 `쪽 영역 안으로 제한` 변경 후 undo 복원
5. `Through` 배치 개체를 속성창에서 확인만 했을 때의 보존

변경은 e2e 파일 신설과 `npm run e2e:undo` 스크립트 추가뿐이다. Studio 런타임 소스, WASM, 기존
소스 검사 테스트, CI workflow는 바꾸지 않는다. 따라서 이 PR에는 시각 출력이나 문서 조판 변경이 없으며
visual sweep은 적용 대상이 아니다.

## 검증

- `git diff --check`: 통과
- `node --test tests/*.test.ts`: 280/280 통과
- `npx tsc --noEmit`: 통과
- `VITE_URL=http://localhost:7700 npm run e2e:undo -- --mode=headless`: 통과
  - Chromium headless에서 5개 흐름과 18개 assertion이 모두 통과했다.
  - 앱 소스 diff가 없으므로, 기존 7700 Vite가 제공한 Studio 런타임은 PR head와 동일하다. PR에서 새로
    추가한 e2e 파일과 package script는 분리 worktree의 최종 head로 실행했다.
- GitHub Actions: `CI preflight`, `Build default-feature tests`, `Build & Test`, `Native Skia tests`,
  `Frontend package gates`, `CodeQL`, `Canvas visual diff`를 포함한 완료 check가 성공했다. `WASM Build`는
  변경 경로 조건에 따라 skipped다.

## Findings

### P2 - 개체 속성 세 흐름은 키보드 단축키 자체를 검증하지 않는다

PR 본문은 다섯 흐름을 "undo"로 설명하지만 실제 키보드 `Ctrl+Z` 입력은 찾아 바꾸기 흐름만 사용한다.
그림, 수식, 표 속성 흐름은 선택 상태를 해제한 뒤 `InputHandler.performUndo()`를 직접 호출한다. 따라서
세 흐름은 실제 속성 대화상자의 apply와 CommandHistory 스냅샷 복원은 검증하지만, 선택 상태에서의 키보드
단축키 dispatch까지 독립적으로 검증하지는 않는다.

이는 [Issue #2301](https://github.com/edwardkim/rhwp/issues/2301)의 핵심인 런타임 스냅샷 복원 회귀를
막는 데 충분하며 merge 보류 사유는 아니다. 다만 PR 설명과 테스트 case 이름의 `Ctrl+Z` 표현을
`performUndo()` 기반 스냅샷 복원으로 정정하거나, 후속으로 선택 개체 한 종류의 실제 `Ctrl+Z` smoke
test를 추가하면 검증 범위가 더 명확해진다.

## 최종 권고

**Accept / merge 완료.** 새 e2e는 기존 문자열 배선 가드와 상보적으로 실제 대화상자 apply 이후의
history 복원을 확인하며, 로컬 Chromium headless 다섯 흐름과 최종 원격 CI가 모두 성공한 뒤
[PR #2302](https://github.com/edwardkim/rhwp/pull/2302)를 squash merge했다. P2는 테스트 설명의
정확성 및 키보드 경로 범위 보완 사항일 뿐, [Issue #2301](https://github.com/edwardkim/rhwp/issues/2301)의
런타임 undo 복원 검증 목표를 막지 않는다.

사용자 지시에 따라 이 archive 문서와 오늘할일은 원 코드 PR에 추가하지 않고, merge 뒤 docs-only
후속 PR로 보존한다. 해당 후속 PR의 CI가 완료되면 원 PR에는 P2 보완 요청과 검토 기록 링크를 남기고,
이슈 close 상태를 확인한다.
