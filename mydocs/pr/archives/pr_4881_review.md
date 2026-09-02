---
kind: pr-review
status: pending-ci
pr: 4881
author: jangster77
base: devel
head: codex/4764-kopub-canvaskit-sfnt
last_verified: 2026-08-16
---

# PR #4881 자체검토 - CanvasKit KoPub SFNT 원본 사용

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#4881](https://github.com/edwardkim/rhwp/pull/4881) |
| 작성자 | `jangster77` (collaborator self-review) |
| base / head | `devel` / `codex/4764-kopub-canvaskit-sfnt` |
| code candidate | `13462cbaa` |
| 변경 범위 | CanvasKit 글꼴 계획, 해당 계약 테스트, stage 문서 |
| 관련 이슈 | [#4764](https://github.com/edwardkim/rhwp/issues/4764) |

collaborator 자체 PR이므로 reviewer를 지정하지 않았다. 이 문서는 review·오늘할일 trailing commit을 추가하는
중의 기록이며, merge 직전에 최신 head, GitHub Actions, mergeability를 다시 확인한다.

base route: `collaborator_self_merge.md`

modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`

loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_self_merge.md`,
`intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`

## 변경 검토

- CSS `FontFace`는 기존 KoPub WOFF/WOFF2 URL과 전송 형식을 그대로 유지한다.
- `resolveCanvasKitFontPlan`만 `canvasKitFile`을 선택해 KoPub은 TTF, KoPubWorld는 OTF SFNT 원본을 CanvasKit에
  전달한다.
- 같은 SFNT 원본을 공유하는 별칭을 URL 단위로 묶어 기존 중복 방지 계약을 보존한다.
- SFNT 원본 URL을 지정하는 KoPub·KoPubWorld 테스트를 추가했다.
- 페이지네이션, Rust, HWP/HWPX fixture, 기준 PDF는 변경하지 않았다. 따라서 이 PR은 #4764 전체를 닫지 않는다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| `cd rhwp-studio && npm test` | 936 passed, 0 failed, 1 skipped |
| `cd rhwp-studio && node --test tests/page-margin-guides.test.ts` | 3 passed |
| `cd rhwp-studio && node --test tests/canvaskit-font-plan.test.ts` | 4 passed |
| `cd rhwp-studio && npm run e2e:canvaskit-font-coverage` | 통과 |
| `cd rhwp-studio && npm run build` | 통과 |
| `git diff --check` | 통과 |
| `http://127.0.0.1:7700` browser shell | 기동 및 console error 0건 확인 |

브라우저 자동화의 숨김 file input 선택 이벤트 제한으로 기준 HWP를 자동 업로드해 캔버스 픽셀을 대조하지는 못했다.
이번 PR의 SFNT 선택은 단위 계약으로 검증했으며, #4764의 실제 문서 PDF 대조와 페이지 수 결론은 별도 근거로
계속 확인한다.

## GitHub CI 결과

기록 당시 code candidate `2102eead0`에서 GitHub Actions를 완료까지 관찰했다.

| CI | 결과 |
| --- | --- |
| CI preflight | 성공 |
| Frontend package gates | 성공 |
| Build & Test | 성공 |
| CodeQL JavaScript/TypeScript 분석 | 성공 |
| Canvas visual diff | 성공 (6분 24초) |
| Rust lint, Native Skia, WASM Build, Rust test shards | 프론트 변경 범위에 따른 정상 skip |

완료 시점 참고값은 `MERGEABLE`/`CLEAN`이었다. 이 문서 commit은 review·오늘할일만 변경하므로 새 trailing
head의 CI와 mergeability를 merge 직전에 다시 확인한다.

## 위험과 병합 조건

- CDN의 SFNT 원본 URL이 제공되는 동안에만 CanvasKit 직접 렌더링이 해당 원본을 사용한다. 기존 offline
  fail-closed 판정은 CSS URL과 CanvasKit URL을 모두 확인하도록 유지했다.
- CanvasKit에 전달하는 TTF/OTF는 CSS WOFF/WOFF2보다 크므로, 문서에서 해당 글꼴을 요청한 경우에만 기존 계획
  단계에서 fetch된다.
- review·오늘할일 trailing head의 GitHub Actions 통과, 최신 `MERGEABLE`/`CLEAN` 확인, 작업지시자 승인이
  필요하다.

## 최종 권고

**보류.** 로컬 프론트 검증과 code candidate GitHub CI는 통과했다. trailing 문서 commit의 최신 CI와
mergeability를 확인한 뒤 작업지시자 승인에 따라 병합한다.
