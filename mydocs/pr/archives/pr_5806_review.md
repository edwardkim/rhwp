---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5806 검토 - HWPX container/OLE groupLevel 보존

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5806](https://github.com/edwardkim/rhwp/pull/5806) / `JamesPsh` |
| source head / 적용 commit | `27641468396bf8f3d522bb07608dab6bad1118a3` / `7dc434ea2` |
| 관련 issue | [#5716](https://github.com/edwardkim/rhwp/issues/5716) |
| GitHub 상태 | Open, non-draft, `MERGEABLE`; source CI 성공 |
| 라우팅 | `maintainer_general` + `intake_and_review` + `local_validation` + `multi_pr_update_branch` |

PR은 HWPX serializer가 container와 OLE `groupLevel`을 리터럴 `0`으로 쓰지 않고 IR 값을 쓰게 하고,
top-level OLE parser가 `groupLevel`을 읽도록 고친다.

## 메인터너 보정

검토 중 `parse_container_body`가 nested `hp:ole`를 dispatch하지 않아 `groupLevel`을 읽기 전에 요소 전체가
버려지는 인접 결함을 확인했다. PR 본문에도 범위 밖 간극으로 명시돼 있었지만, 원 변경의 "container/OLE
groupLevel 보존" 계약을 실제 nested OLE에 적용하려면 같은 통합 범위에서 보정해야 한다.

적용 commit `270d28ffd`는 `b"ole"` arm에서 기존 `parse_hp_ole_element`를 호출하고, 새 정적 test를
늘리지 않고 기존 `issue4669_parse_ole_preserves_shape_component_children_and_id`에 nested container
`hp:ole groupLevel="3"` 왕복 assertion을 추가했다. focused test 1건과 unit-tier check가 통과했다.

`node scripts/rust-test-suite-manifest.mjs --check`의 Cargo generated target drift는 기준 branch에도 있는
CI 생성물 불일치다. `tests/generated/regression_suite_*`, `tests/suites/manifest.json`은 CI 관리 대상이라는
운영 규칙에 따라 수정하거나 stage하지 않았다. generator Node test 18건과 unit-tier Node test 12건은 통과했다.

## 검증과 최종 권고

전체 nextest **8,059 passed**, native-Skia, fmt, clippy, standard WASM build가 통과했고 code candidate
GitHub CI도 성공했다. **메인터너 보정을 포함해 수용 권고.** merge 뒤 #5716의 close 상태를 확인하고,
중첩 OLE dispatch 보정을 포함한 이유를 contributor comment에 명시한다.
