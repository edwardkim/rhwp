---
kind: pr-review
status: self-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6115 self-review — 기본 도구 상자 접기/펴기

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `7c4fffa48d9a55da7acde3b8b916b84c3163ace3`

`visual_fixture_evidence.md`는 적용하지 않는다. 이 PR은 Studio 메뉴와 도구 상자 DOM 표시 상태를 바꾸지만
문서 renderer·layout·typeset·paint, HWP/HWPX fixture와 페이지 출력 경로를 바꾸지 않는다. 실제 브라우저의
버튼·단축키·리로드·첫 페인트를 확인하는 기능 E2E가 이 변경의 직접 증거다.

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#6115](https://github.com/edwardkim/rhwp/pull/6115) |
| 작성자 | `postmelee` |
| 관련 이슈 | [#6112](https://github.com/edwardkim/rhwp/issues/6112), 선행 [#5738](https://github.com/edwardkim/rhwp/issues/5738) |
| base / head | `devel` / `codex/issue-6112-toolbox-collapse` |
| 규모 | 21 files, +696 / -47, 4 commits |
| 상태 | Open, non-draft, `MERGEABLE`, `mergeStateStatus=CLEAN` |
| base SHA | `upstream/devel@6b5c4f871972380c0866e2a8d27ac2bc67d257e6` |

작성 시점에 branch는 최신 base보다 `0 behind / 4 ahead`다. GitHub 상태는 변할 수 있으므로 push와 merge
직전에 다시 확인한다. 별도 `pr_6115_review_impl.md`는 만들지 않는다. 외부 PR 보정, 복수 PR 통합 또는
source 충돌 해결이 없고 self-review에서 제품 정정이 필요하지 않았다.

## 목적과 변경 범위 정합성

#5738에서 이미 제공한 `view:toolbox-basic`, `userSettings`, 루트 `data-toolbox-*`, 첫 페인트 초기화와
보기 메뉴를 단일 상태 경로로 재사용했다.

- 신규·미설정 상태의 `toolbarBasic`만 `false`로 바꾸고 `toolbarFormat=true`는 유지했다.
- 저장값이 명시적으로 `true` 또는 `false`인 기존 사용자는 그 값을 그대로 복원한다.
- 메뉴바 우측 버튼과 `보기 > 도구 상자 > 기본`, `Ctrl+F1`은 모두 `view:toolbox-basic`을 실행한다.
- 편집 textarea는 기존 InputHandler가, 그 밖의 포커스는 전역 경로가 단축키를 소유해 한 이벤트가 두 번
  실행되지 않는다.
- 버튼은 `aria-controls`, `aria-expanded`, 동적 이름과 툴팁으로 현재 상태와 다음 동작을 알린다.
- 별도 이미지 자산을 추가하지 않고 기존 색상·radius 토큰과 CSS 화살표를 사용한다.
- 서식 도구 상자 소형화, 새 toolbar framework, renderer 변경은 범위에 포함하지 않았다.

## self-review findings

### 추가 blocker 없음

- `theme-init.js`와 `defaultSettings()`의 신규 기본값이 모두 `toolbarBasic=false`로 일치한다. 첫 페인트
  스크립트는 저장된 `true`만 기본값 위에 덮고, 저장된 `false`는 동일한 숨김 상태를 유지한다.
- `applyToolboxVisibility()`는 같은 커맨드를 가진 메뉴 체크 항목과 우측 버튼을 구분해 각각
  `aria-checked`와 `aria-expanded`를 갱신한다.
- 메뉴 이벤트 위임 범위를 `data-cmd`가 있는 드롭다운 항목과 직접 버튼으로 한정해 다른 메뉴 요소를
  오발동시키지 않는다.
- `Ctrl+F1`은 기존 matcher 계약에 따라 macOS에서는 Command 조합도 수용하며, 브라우저 기본 동작을
  `preventDefault()`한다.
- 기존 명시적 표시값, 서식 도구 상자 기본값, 저장·리로드와 숨김 첫 페인트 회귀를 테스트가 고정한다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Studio focused·전체 unit | 1,138 tests, 1,137 pass / 1 skip / 0 fail |
| TypeScript·Vite production build | 통과 |
| 실제 브라우저 E2E | 버튼 클릭, 버튼 포커스 `Ctrl+F1`, 저장·리로드 통과 |
| 첫 페인트 | 숨김 상태 visible frame `0/37` |
| `cargo fmt --all` / `-- --check` | 통과 |
| Markdown 링크 | 603문서, 상대 링크 이상 없음 |
| `git diff --check` | 통과 |

Rust source와 새 integration test source가 없어 `cargo test`, clippy, Rust unit tier와 generated suite
manifest 변경은 범위 밖이다. 전체 문서 metadata 검사는 이번 변경과 무관한 기존 `mydocs/tech` 문서의
16건을 보고했으며, 새 계획·working·report·feedback 문서는 해당 오류 목록에 없다.

## GitHub Actions

code candidate `7c4fffa48`의 [CI run 32922305619](https://github.com/edwardkim/rhwp/actions/runs/32922305619)은
Frontend package gate와 Build & Test aggregate가 성공했다. 변경 경로 밖의 Rust·archive·WASM 작업은 정책에
따라 skip됐다. [CodeQL 32922305491](https://github.com/edwardkim/rhwp/actions/runs/32922305491),
[Proptest 32922305496](https://github.com/edwardkim/rhwp/actions/runs/32922305496),
[Adapter inter-diff 32922305508](https://github.com/edwardkim/rhwp/actions/runs/32922305508),
[Render Diff 32922305503](https://github.com/edwardkim/rhwp/actions/runs/32922305503)도 같은 SHA에서 성공했다.
최종 집계는 13 success, 12 policy skip, 0 failure, 0 pending이다.

이 문서와 상태 현행화만 담은 변경은 녹색 code candidate 뒤의 `mydocs/` 한정 single-parent trailing
commit이다. push 뒤 review-only fast-pass가 정확한 후보를 재사용하고 최신 required aggregate가 성공하는지
다시 확인한다.

## 최종 권고

기존 자산을 재사용하면서 신규 사용자의 세로 편집 공간을 확보하고, 저장된 사용자 선택과 서식 도구 상자
정책을 보존한다. 상태 경로·단축키 소유권·접근성·첫 페인트에서 추가 blocker는 발견하지 않았다.

self-review는 **완료 / 조건부 merge 권고**다. code candidate의 GitHub Actions 성공, review-only trailing
head의 fast-pass, 최신 `MERGEABLE/CLEAN`과 작업지시자의 별도 merge 승인을 확인하기 전에는 merge하지
않는다.
