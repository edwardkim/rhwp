---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4695 검토 - 저장 조각 폭 음수 방지

| 항목 | 기록 |
| --- | --- |
| PR | [#4695](https://github.com/edwardkim/rhwp/pull/4695) |
| 작성자 / 원 head | @planet6897 / `e26e3fd86da90e04cfa4f2ac6b5ffdd16a46d549` |
| 적용 commit | `1e11d5681` (`#4690`) |
| 통합 후보 | `c8f6a7dac` |

저장 조각의 시작점이 여백을 포함하지 않을 때 음수 폭의 줄 상자가 만들어지는 경로를 차단한다.
범위를 벗어난 줄을 억지로 그리지 않고 실제 content box로 제한하는 수정이다.

원 PR은 force-push 뒤 #4690 단일 commit으로 정리됐다. 새 원 head와 현재 후보의 적용 commit은
동일 patch-id `ca10b8f4d6ab05e289826749ad64624c3e6e9f97`이므로 재적용하지 않았다. 이전 source에
있던 #4088 한국어 절 경계 변경은 더 좁은 목적격 절 경계로 해결한
[PR #4700](https://github.com/edwardkim/rhwp/pull/4700)으로 대체했고, 동일 탐지기를 두 번 바꾸지 않았다.

## 완료한 검증

- 비공개 fixture의 저장 조각 렌더에서 page 3 밖쪽 text node가 0건임을 확인했다. 비공개 자료의 이름과 경로는 기록하지 않는다.
- 동등 patch의 앞선 누적 후보 전체 `nextest`는 5,923건 통과, 37건 제외, 실패 0건이었다.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check`를 통과했다.

**통합 수용 대상이다.**
