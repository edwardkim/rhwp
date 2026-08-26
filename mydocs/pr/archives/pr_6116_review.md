---
kind: pr-review
status: corrections-complete-awaiting-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6116 self-review — 활성 페이지·눈금자 2D 정합성 (#6107)

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `rework_and_exceptions.md`의 대형 PR 경로
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와
  `docs_and_git_workflow.md`
- 작성자 본인 self-review이며, 외부 리뷰
  [pullrequestreview-5028382095](https://github.com/edwardkim/rhwp/pull/6116#pullrequestreview-5028382095)의
  10개 inline 항목을 독립 보정 gate로 처리했다.

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#6116](https://github.com/edwardkim/rhwp/pull/6116) |
| 이슈 | [#6107](https://github.com/edwardkim/rhwp/issues/6107) |
| 작성자 | `postmelee` |
| base / head | `devel` / `codex/issue-6107-active-page-ruler` |
| 원 review head | `6b0fa6ee9de406c9f9abf13ca8ab19bd277a1321` |
| 보정 code candidate | `f16b1fed8` |
| 원 review head 상태 | Open, non-draft, `MERGEABLE/CLEAN` |
| 원 review head 규모 | 19 files, +1,401 / -143 |

GitHub 상태는 변할 수 있으므로 push와 merge 판단 전에 다시 확인한다. 1,000줄을 넘는 변경은 활성 페이지
resolver, CanvasView, 키 이동, 눈금자, 테스트와 단계별 문서가 결합된 기능 단위다. 대형 PR 규칙에 따라
즉시 merge하지 않고 리뷰 보정, 최신 code head CI와 사용자 merge 판단을 별도 gate로 둔다.

## 결론

**리뷰 보정 완료 — 최신 code head CI와 merge 승인 조건부 수용 가능**.

실제 결함으로 판정한 2D 가시성, page count 경계, 다중 개체 선택 페이지, stale 개체 focus와 약한 테스트를
보정했다. active viewport와 마지막 편집 focus는 서로 다른 사용자 상태이므로 두 이벤트를 하나로 합치지
않았다. 한글 2024 대조와 사용자 확인으로 확정한 “눈금자는 마지막 클릭의 물리 페이지 좌표에 남는다”와
“가로 이동 PageUp/PageDown은 X축만 바꾼다” 계약도 유지했다.

## 외부 리뷰 판정

### 반영

- 모든 페이지 배치에서 X/Y viewport 교차를 함께 검사하고, viewport 중심 페이지도 2D 좌표로 판정한다.
- 눈금자 page index를 문서·레이아웃 page count의 공통 범위로 제한한다.
- 다중 그림 선택의 overlay 렌더 페이지와 편집 focus 페이지를 분리한다.
- 그림·표 선택 해제와 렌더 clear 경로가 stale editing page를 남기지 않게 한다.
- source 문자열 정규식 테스트를 실제 `VirtualScroll`·resolver·선택 helper 행위 테스트로 교체한다.
- PageUp assertion을 실제 X 변화량과 Y축 불변식으로 강화한다.

### 부분 반영

- `active-page-changed`는 가시 viewport, `focused-page-changed`는 마지막 클릭·편집 focus라 의미가 다르다.
  이벤트 병합은 하지 않고 눈금자의 문단 편집 문맥만 focus 페이지로 교정했다.
- 페이지 경계 loop에서 캐시된 `getTotalWidth()`를 한 번만 읽는다. 전체 탐색은 현재 O(n)이고 문서 topology
  변경 때만 준비되는 배열을 사용하므로, 복잡한 binary-search index는 profile로 병목이 확인될 때 다룬다.

### 미반영

- focus 페이지가 화면 밖이면 눈금자를 새 가시 페이지에 붙이는 제안은 #6107의 최종 UX 계약과 반대라
  적용하지 않았다. 순수 스크롤은 focus를 바꾸지 않으며, 새 페이지를 클릭해야 눈금자가 이동한다.
- 가로 PageUp/PageDown에서 남은 이동을 Y축 overflow로 보내는 제안은 모드의 축 예측 가능성을 깨므로
  적용하지 않았다. 가로 모드의 페이지 키는 X축 전용이고 Y축은 별도 native 스크롤 입력으로 접근한다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 보정 focused test | 36/36 통과 |
| Studio 전체 test | 1,151 pass, 1 skip, 실패 0 |
| TypeScript | 통과 |
| 프로덕션 build | 230 modules, 통과 |
| Chrome PageUp/PageDown E2E | 6쪽 TC1~TC7 전체 통과 |
| generated suite 준비 | 32 harnesses, 9 exceptions |
| `cargo fmt --all`·`--check` | 통과 |
| `git diff --check` | 통과 |

문서 내부 paint, PDF/SVG export와 WASM renderer는 바뀌지 않았다. 변경은 브라우저 페이지 가시 후보와 UI
focus routing이므로 실제 Chrome E2E와 함수 수준 2D 배치 테스트를 사용했다. 기존 #6107 수동 검증에는
세로·가로·두 쪽·맞쪽·여러 쪽에서 페이지 클릭 후 두 눈금자 이동과 순수 스크롤 focus 유지가 포함된다.
이번 보정은 그 시각 계약을 바꾸지 않아 별도 PDF visual sweep을 추가하지 않는다.

## 최종 권고

외부 리뷰의 재현 가능한 정확성·lifecycle 항목은 보정됐고, 두 의도된 계약은 사용자 혼란을 줄이는 기존
결정을 유지한다. 보정 code candidate와 이 review 기록을 push하고 10개 스레드에 항목별 근거를 남겨
resolve할 수 있다. 최신 head의 Frontend package gate와 required aggregate가 성공하고
`MERGEABLE/CLEAN`을 다시 확인하기 전에는 merge하지 않는다.
