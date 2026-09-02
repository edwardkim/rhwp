---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5898 검토 - HWP 문단 stream 좌표 보존

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5898](https://github.com/edwardkim/rhwp/pull/5898) / `@Shadungi` |
| 관련 issue | closes #5894, #5895 |
| source head | `39b02a8c15c5d5453326270567e1f03115b695c3` |
| 상태 | non-draft, `CLEAN`, source CI 완료 |
| 적용 commit | `b509bdcbb4`, `c083fbbd0f` |

## 검토

- 탭을 HWP stream의 8-unit 확장 문자로 계산하고 astral UTF-16 폭과 함께 삽입, 삭제, 분할,
  병합에 일관되게 적용한다. 분할 문단에서는 PARA_HEADER instance ID만 0으로 초기화하고 변경
  추적 suffix는 보존한다.
- `A🙂\tB`의 좌표 `[0, 1, 3, 11]`, `char_count`, split/merge/delete와 raw header suffix를
  검증하는 `paragraph_stream_coordinates_contract` 2건을 통과했다.
- source CI의 Rust archive, Lint, Native Skia, Canvas visual diff, CodeQL, Adapter, Proptest가
  완료했다. 통합 전체 nextest와 clippy에서도 차단 결함을 발견하지 못했다.

## 판정

**통합 후보 수용.** 문단 내부 바이트·좌표 계약 변경이므로 fixture 시각 검증은 요구하지 않는다.
