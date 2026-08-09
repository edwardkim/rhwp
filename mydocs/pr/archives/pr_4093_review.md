---
kind: pr_review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4093 검토 — VS Code 개요 번호 탐색 패널

## 결론

**기존 변경 요청 2건은 해소되었고, 최신 contributor head를 현재 `devel`에 합친 트리에서
로컬 코드·WASM·실제 브라우저 검증이 통과했다.** 첫 검토에 섞였던 Python binding 변경은
제거됐고, 빈 `closes #`도 정리됐다. 표 셀 번호 문단은 렌더러와 같은 순서로 카운터에
반영되며, 접기 버튼은 키보드 조작 뒤에도 초점을 보존한다.

최신 head의 GitHub Full CI, CodeQL, Render Diff도 모두 성공했고, 2026-08-08 작업지시자의
실제 VS Code 시각 판정도 통과했다. 두 변경 요청 thread를 해소하고 최신 head를 승인한 뒤,
작업지시자의 merge 승인에 따라 `devel`에 admin merge했다.

## 라우팅

base route는 `maintainer_general.md`다. 적용 보조 절차는 `intake_and_review.md`,
`local_validation.md`, `multi_pr_update_branch.md`, `visual_fixture_evidence.md`,
`rework_and_exceptions.md`, `review_only_fast_pass.md`다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4093](https://github.com/edwardkim/rhwp/pull/4093) / @walnutkim |
| 기여 이력 | 저장소 첫 PR, collaborator 아님 |
| contributor code head | `f2964508c0a0967fbab5669f1cc23d23091faf7e` |
| 최종 review head | `179a5e69ca5f5ace6937dab79ba9d6846de99713` |
| 최신 검토 기준 `devel` | `0fbfc24e6dd254a6df804f5282302885e8a2f837` |
| devel과 거리 | `devel` 139커밋 / PR 6커밋 |
| 최종 규모 | 1,637+ / 34-, 15파일, review 문서 commit 포함 |
| 로컬 review branch | `review/walnutkim-20260808` |
| 최종 합성·실제 merge tree | `508d28d637518ad0e1119ce89995eed176813cfc` |
| GitHub 상태 | Merged / merge commit `c119e5db5829df2e27850258500e5e2dd74a0015` |
| reviewer | @edwardkim `APPROVED`, @jangster77 요청 철회 |

이전 검토 branch `review/walnutkim-20260807`은 force-push 이전 기록 보존용으로 유지했다.
최종 head와 검토 기준 `devel`의 3-way merge tree는 충돌이 없고 `git diff --check`도 통과했다.

## 이전 변경 요청의 처리 확인

### 범위 정리

- 무관한 Python binding 커밋은 최신 PR history와 diff에서 제거됐다.
- 본문의 빈 `closes #`는 제거됐다.
- 기여자는 당시 `devel` `d634e608`에 rebase했다. 검토 시점에는 `devel`이 다시 139커밋
  전진했지만 current-base merge simulation은 clean이고, source·test·fixture의 고유 diff는
  변하지 않는다.

### 표 셀 번호 문단 이후의 개요 번호

`18e9dd2b`에서 표와 표 안 표의 셀 문단을 행 우선 순서로 순회하고 renderer와 같은
`NumberingState`, `resolve_numbering_id`, `expand_numbering_format`을 사용하도록 보정했다.
셀의 `Number`/`Outline` 문단은 탐색 목록에는 넣지 않되 뒤 개요의 번호 카운터에는 반영한다.

두 HWPX fixture의 통합 테스트는 query와 렌더된 SVG를 직접 대조한다.

- 최소 경계: 앞 개요 `1.` → 표 셀 Number `2.` → 뒤 개요 `3.`
- 패널 데모: 3수준 15개 항목, 3쪽, 표 셀 경계 뒤 `5. 부칙`

### 접기 버튼의 키보드 이벤트와 초점

`b6599864`에서 부모 항목의 keydown을 `event.target === item`일 때만 처리하고,
`dc20530a`에서 재렌더 뒤 같은 toggle로 초점을 복원했다. 추가된 방향키 이동은 본문을
움직이지 않고 보이는 항목 사이에서 초점만 옮기며, `Enter`/`Space`에서만 본문 위치로 간다.

소스 계약 테스트뿐 아니라 production webpack 산출물을 실제 headless Chromium에 로드해 다음을
확인했다.

- toggle `Enter` 연속 2회: 접힘·펼침과 초점 유지, 본문 scroll 없음
- `ArrowDown`: `0:1`에서 `0:4`로 초점 이동, 본문 scroll 없음
- `Enter` 이동: `0:16` 항목에서 2쪽, scrollTop 1268, 강조선 표시

방향키 기능은 개요 패널의 keyboard-only 접근성 범위와 응집돼 있고 실제 브라우저 검증을 통과해
별도 PR 분리를 요구하지 않는다.

## 알려진 범위 메모

글상자 안 표는 기존 renderer의 `layout_embedded_table()`이 구역 `outline_numbering_id` 대신
`0`을 전달하는 협소 경로다. 새 query도 최상위 문단의 직접 표만 카운터에 반영한다. 따라서 글상자
내 표 셀에 개요/번호 문단이 있고 뒤 본문 개요와 같은 numbering state를 공유하는 문서는 별도
정합성 과제로 남는다.

이 차이는 이 PR이 수정한 일반 표 경로에서 새로 만든 회귀가 아니며, 현재 fixture와 공개 기능의
수용을 막지는 않는다. renderer와 query 양쪽의 글상자 traversal·outline id 전달을 함께 고치는
후속 이슈로 분리하는 것이 적절하다.

## 로컬 검증

코드 검증 대상은 contributor code head를 당시 `devel`에 clean하게 합성한 tree
`fca45be81455b3f2f4b9b303f656ebf91ff6ac40`다. 이후 추가된 두 commit은 이 검토 문서만
변경하며, 최종 head와 merge 직전 `devel`의 tree는
`508d28d637518ad0e1119ce89995eed176813cfc`다.

| 게이트 | 결과 |
| --- | --- |
| 3-way merge tree / `git diff --check` | 충돌 없음 / 통과 |
| `cargo test --lib document_core::queries::navigation` | 4 passed |
| `cargo test --test outline_navigation_table_cell_number` | 2 passed |
| `node scripts/frontend-vscode-outline.test.mjs` | 5 passed |
| IR field sweep + baseline diff | 815샘플, 597경로, 112,314건 / 기준선 byte-identical |
| overflow-cell sweep + baseline diff | 678샘플, 20문서, 1,849줄 / 기준선 byte-identical |
| 새 `samples/pr4093` overflow | 0, baseline 갱신 불필요 |
| Docker `wasm-pack 0.15.0 build --target web --out-dir pkg` | 통과, 5분 8초 |
| 생성 WASM 선언 | `getOutlineNavigation()` 확인 |
| `npm --prefix rhwp-vscode run typecheck` | 통과 |
| `npm --prefix rhwp-vscode run compile` | 통과 |
| production webview + real WASM browser smoke | 통과, 3쪽·15개 항목·키보드·이동 검증 |

fresh `npm ci`는 성공했으나 현재 lockfile 기준 audit이 high 1건을 보고했다. 이 PR의 설치·컴파일
실패는 아니며 자동 `npm audit fix`는 수행하지 않았다.

GitHub Full CI 재사용 조건을 충족하므로 release-test 전체와 Native Skia 3종은 로컬에서 중복
실행하지 않았다. 정확한 code head `f2964508`의 broad run과 review 문서만 추가된 최종 head
`179a5e69`의 review-only fast-pass를 근거로 삼는다.

- [CI / Build & Test — success](https://github.com/edwardkim/rhwp/actions/runs/31152782217)
- [CodeQL — success](https://github.com/edwardkim/rhwp/actions/runs/31152782861)
- [Render Diff — success](https://github.com/edwardkim/rhwp/actions/runs/31152781488)
- [최종 head CI / Build & Test — success](https://github.com/edwardkim/rhwp/actions/runs/31244205082)
- [최종 head CodeQL — success](https://github.com/edwardkim/rhwp/actions/runs/31244205012)
- [최종 head Render Diff — success](https://github.com/edwardkim/rhwp/actions/runs/31244205019)

## 시각·조작성 증적

로컬 산출물은 gitignored `output/4093/`에 두었다.

| 파일 | SHA-256 | 판정 |
| --- | --- | --- |
| `pr4093-outline-panel-current-base.png` | `ce2beb6eb663716301fce13b79f4b72aeb916f234af5391fcb7c5fbefafbc217` | 3수준 계층·번호·들여쓰기·표 뒤 `5. 부칙` 확인 |
| `pr4093-outline-navigation-page2.png` | `2ec2592f790df4756eca0ec872c77ba8a25e6c22a8eaced7b803bc47d452383b` | 2쪽 이동·focus·문단 강조 확인 |
| `pr4093-browser-validation.json` | `5cf913d83fb3f651c2580506db101602306154e13652603b9ebb08c3590d13d9` | 실제 브라우저 측정값과 오류 0건 기록 |

에이전트 육안 검토에서는 겹침·잘림·tofu·사이드바 침범이 없었다. 작업지시자는 2026-08-08
같은 VS Code 결과를 확인하고 **시각 판정 통과**로 승인했다.

## 수용 및 병합 결과

**수용·병합 완료.** 검토 기록 commit `be0b4b28c`와 시각 승인 기록 commit `179a5e69c`를
contributor source branch에 반영했다. 기존 `CHANGES_REQUESTED`의 두 inline thread는 contributor
응답과 로컬·시각 재검증을 근거로 해소했고, 최신 head에 @edwardkim의 `APPROVED` review를 게시했다.
@jangster77 review 요청은 작업지시자가 수동으로 제외했다.

최종 head의 CI·CodeQL·Render Diff와 `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`을 재확인한
뒤 2026-08-08 15:59:37 KST에 작업지시자의 승인에 따라 admin merge했다. merge commit은
`c119e5db5829df2e27850258500e5e2dd74a0015`이며 원격 `devel`도 같은 SHA로 갱신됐다. PR 본문에
종료할 유효한 관련 이슈는 없다.

## 기존 GitHub 리뷰

- 게시 시각: 2026-08-07 11:57:57 KST
- 대상 head: `97eb4ad8b2ae741e685510183b173a0563f574ad`
- 판정: `CHANGES_REQUESTED`
- 리뷰: <https://github.com/edwardkim/rhwp/pull/4093#pullrequestreview-4879550740>
- inline thread 2건: contributor 수정과 재검증을 확인한 뒤 2026-08-08에 모두 resolved
- 최종 판정: 최신 head `179a5e69`에 @edwardkim `APPROVED`
