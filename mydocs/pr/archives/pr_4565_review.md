---
kind: pr-review
status: pending-full-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4565 리뷰 - 임베드 호스트 UI 프로파일

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md
```

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| 원 PR | [#4565](https://github.com/edwardkim/rhwp/pull/4565) |
| 작성자 / source | @JamesPsh / `issue-4564-embed-chrome-profile` |
| base / 원 code head | `devel` / `eaa7aac9f30ea9bcf010ca19ef1b9a90fef050b6` |
| 원 PR 규모 | 5 files, +443/-26, 2 commits |
| 최신 `devel` 기준 | `296d1fb3fcd06a4f5acab9437356a01b0cddb3dd` |
| 가시성 보정 브랜치 | `review/jamespsh-4565-20260812` |
| 최신 `devel` merge | `efbdd88db104ef84f6af66dd37facc903201340a` |
| 최종 code candidate | `4c511dd8578867649eade06880f8ce1d83a07e7d` |
| 원 source 수정 권한 | `maintainerCanModify=true` |
| mergeable | 문서 작성 시점 `MERGEABLE` / `CLEAN`; merge 전 재확인 필요 |

원 PR은 [#4564](https://github.com/edwardkim/rhwp/issues/4564)의 `?chrome=embed` opt-in UI
프로파일을 구현한다. `maintainerCanModify=true`이므로 collaborator 외부 PR 절차 9.3.1.4에 따라
원 source `eaa7aac9f`를 첫 부모로 유지하고, `efbdd88db`에서 최신 `devel`을 일반 merge했다. 원
contributor history는 재작성하지 않는다.

## 검토와 메인터너 보정

embed 프로파일은 문서 수명주기 명령을 등록하지 않고, 메뉴·도구막대·명령 팔레트·단축키가 공통으로
지나는 command registry 필터에서 막는다. `file:page-setup`과 `file:about`은 유지한다.

메인터너 검토에서 확인한 우회 경로는 contributor의 두 번째 commit에서 반영됐다.

- `edit:compare-documents`를 embed 명령 등록과 모든 UI 표면에서 제외했다. 비교 실행이 현재 문서를
  호스트가 감지하지 못하게 교체할 수 있기 때문이다.
- embed의 unsaved dialog에서는 registry를 우회하는 직접 로컬 저장을 비활성화했다. 자동 discard는
  추가하지 않고 사용자가 저장 안 함 또는 취소를 고르게 한다.
- 문서를 열기 전에도 `Ctrl+S`, `Ctrl+Shift+S`, `Ctrl+P`가 브라우저 저장·인쇄 대화상자로 빠지지 않게
  capture 단계 전역 리스너에서 흡수한다.

최신 `devel` merge 시 [#4602](https://github.com/edwardkim/rhwp/pull/4602)의 Subsecond 진단 코드가
`init()` 반환값의 `memory`를 읽으면서 `tsconfig.ci-unit.json`에서 `TS18046`을 냈다. 이는 원 PR의
변경이 아니라 최신 기준선 호환 결함이다. 메인터너 보정 `4c511dd85`은 `init()` 결과를
`{ memory?: WebAssembly.Memory }`로만 좁혀, 개발 glue의 memory 부재는 그대로 허용하면서 타입 오류를
해소한다. renderer, 명령 동작, contributor source는 변경하지 않는다.

## 완료한 검증

- 원 PR head `eaa7aac9f`의 GitHub Actions에서 Frontend package gates, Canvas visual diff,
  JavaScript/TypeScript·Python·Rust CodeQL 분석이 성공했다. Rust·Native·WASM lane은 Studio 변경 범위에
  따라 skipped였으며, 이 결과는 source 보정 candidate의 최종 CI를 대체하지 않는다.
- `node --test tests/chrome-mode.test.ts`: 12건 통과.
- `npm test`: 859건 통과.
- `./node_modules/.bin/tsc -p tsconfig.ci-unit.json --noEmit`: 통과. 보정 전 최신 `devel` 통합 tree에서
  재현된 `TS18046`도 해소됐다.
- `npm run build`: `tsc && vite build` 통과. Vite의 CanvasKit `fs`·`path` browser externalization과
  500 KiB chunk 경고만 출력됐고 빌드는 성공했다.
- Chrome DevTools Protocol 실브라우저 점검: `chrome=embed`에서 파일 수명주기 10종과
  `edit:compare-documents`의 표면 수가 모두 0이고, page setup/about은 각각 1개로 유지됐다.
  문서가 없는 상태의 `Ctrl+S`, `Ctrl+Shift+S`, `Ctrl+P` dispatch는 모두 `defaultPrevented=true`였다.
  dirty 문서에 host RPC `loadFile`을 보내면 저장 비활성 unsaved dialog가 열리고, 취소 시 dirty 상태와
  기존 문서는 보존됐다. `chrome=full`에서는 저장 표면 1개와 비교 표면 3개가 유지됐다.
- `git merge-tree --write-tree upstream/devel upstream/pr4565-head`, `git diff --check upstream/devel...HEAD`:
  통과. source와 최신 `devel`은 모두 최종 candidate의 조상이고, 검증했던 `14e9e9380` tree와
  `4c511dd85` tree가 동일함을 확인했다.

## 범위와 남은 조건

- 이번 source 보정은 Studio UI 프로파일과 최신 `devel`의 TypeScript 호환 보정만 포함한다. host RPC의 최근 문서
  기록, embed 단축키의 진단 로그, 별도 문서 세션 비교는 원 PR이 명시한 범위 밖으로 유지한다.
- 최신 `devel` merge와 타입 보정 code commit이 포함됐으므로 review-only fast-pass 대상이 아니다. #4565의
  최신 source head에서
  Frontend gate, CodeQL, Render Diff 및 branch protection aggregate를 다시 확인해야 한다.

## 최종 권고

**수용 권고.** source 보정 candidate는 로컬 TypeScript·프런트엔드 테스트·프로덕션 빌드와 실제 embed
브라우저 경로를 통과했다. #4565 최신 CI와 작업지시자 merge 승인이 충족되면 #4564의 종료 상태를 확인한다.
