---
kind: pr-review
status: accepted-with-ci-condition
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6464
author: t2c-lab
---

# PR #6464 review - Gmail 서명 HTML raw tag 방지

## 검토 기준

- 원 PR head: `024cf9dea0bd161eb8e78a904e2d5fba916709cd`
- 통합 적용 commit: `aa9a2386a`, `6184ef986`
- 기준 base: `upstream/devel@19b89d967b1505cd4bdcdbba7d1f1413f32a1505`
- 작성 시점 원 PR은 Open/non-draft였고 최신 source head의 Build & Test와 CodeQL은 성공했다
  (CodeQL aggregate는 neutral). 최종 통합 PR 직전에 상태를 다시 확인한다.

## 변경과 메인터너 보정

- top-level `span`, nested `strong`/`u`, `ul`/`li`가 raw HTML text가 아니라 문단과 inline shape으로
  들어가도록 HTML import 경로를 보완한다. 관련 원인은 [#6463](https://github.com/edwardkim/rhwp/issues/6463)에
  기록돼 있다.
- 원 PR의 서명 shape 결과를 고정하는 새 Rust contract가 없었다. 메인터너 보정은 production logic을
  바꾸지 않고 public `paste_html_native` 결과로 다음을 고정했다: top-level span의 tag 미노출, 붙여넣은
  문단에 적용된 bold/underline shape, 두 list item의 bullet text. 붙여넣기 이후의 run 시작 위치는 내부
  document offset으로 재계산되므로 UTF-16 offset을 외부 API 계약으로 고정하지 않는다.
- #6486 초기 CI는 source-side `#[cfg(test)]` 증가를 거부했다. 네 계약을
  `tests/cases/html_import_paste_contract.rs`로 옮긴 뒤 source module test `2 passed`, public contract
  `4 passed`, RustUnitTier base `4,221`을 확인했다.

## 판단

**수용 권고.** 이슈의 raw tag와 서명 서식 재현 경로를 직접 assertion으로 묶었다. 실제 서명 원본은
개인정보가 될 수 있어 장기 fixture로 포함하지 않았으므로, 최소 대표 markup contract로 검증 범위를
명시한다. 통합 branch의 최종 head Full CI와 mergeability 통과가 merge 전 조건이다.
