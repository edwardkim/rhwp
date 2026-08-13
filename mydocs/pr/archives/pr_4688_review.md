---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4688 검토 - HWP5 저장 `LINE_SEG` 본문 보존

| 항목 | 기록 |
| --- | --- |
| PR | [#4688](https://github.com/edwardkim/rhwp/pull/4688) |
| 작성자 / 원 head | @planet6897 / `72c3aa8ebf20a2940dd8f34c119c90c28ed79aa5` |
| 적용 범위 | 원 PR의 처음 세 serializer commit |
| 통합 후보 | `c7cfaefb9` 위 `8335162b3`, `4bfd989c3`, `a36a9d756` |

원 PR의 후속 세 commit은 이번 누적 후보에 넣지 않았다. 본 검토 범위는 본문에 존재하지 않는
`LINE_SEG` 제거, end-exclusive 범위 판정, `PARA_TEXT`가 없는 문단의 실제 방출 글자 수 정합이다.

## 완료한 검증

- 한컴 Office 2020 MCP로 `hwp3-sample10-hwp5`를 PDF로 변환해 763쪽 A4 출력과 `PrintToPDFEx`,
  `PrintMethod=0`, 본문 검증 성공을 확인했다.
- 증적 PDF는 [hwp3-sample10-hwp5-lineseg-normalized-2020.pdf](../../../pdf/pr-review-planet6897-20260813/hwp3-sample10-hwp5-lineseg-normalized-2020.pdf)에 보존했다.
  SHA-256은 `a11af9041a0a116395f7647f8476d63f2da8e4c42277a895b6046db3042bf3f0`이다.
- 누적 후보에서 실행한 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`는 5,923건 통과, 37건 제외, 실패 0건이었다.

원본에 이미 손상된 `LINE_SEG` 꼬리 네 건은 저장 과정의 정규화 대상이며, 본문 보존 실패로 판정하지 않았다.

**통합 수용 대상이다.** GitHub 상태는 merge 직전에 최신 head로 재확인한다.
