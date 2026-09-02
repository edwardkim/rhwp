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
| 적용 범위 | 원 PR의 여섯 serializer/converter/HWPX commit |
| 통합 후보 | `c8f6a7dac` 위 `8335162b3`~`c8f6a7dac` |

본문에 존재하지 않는 `LINE_SEG` 제거, end-exclusive 범위 판정, `PARA_TEXT`가 없는 문단의 실제
방출 글자 수 정합을 먼저 적용했다. 원 head가 뒤이어 추가한 세 commit도 최신 상태로 반영했다.

- 캡션 달린 묶음 개체는 한컴 번호 범주 bit 29와 캡션 문단 header tail을 사용한다.
- HWPX bookmark는 HWP5 제어문자처럼 8-unit 위치 슬롯을 점유한다.
- Ruby는 `tdut` control character와 그 CTRL_HEADER를 짝으로 방출한다. Ruby 내용 자체의 HWP5
  레코드 완성은 이번 범위가 아니라 #4397 소관이다.

## 완료한 검증

- 한컴 Office 2020 MCP로 `hwp3-sample10-hwp5`를 PDF로 변환해 763쪽 A4 출력과 `PrintToPDFEx`,
  `PrintMethod=0`, 본문 검증 성공을 확인했다.
- 증적 PDF는 [hwp3-sample10-hwp5-lineseg-normalized-2020.pdf](../../../pdf/pr-review-planet6897-20260813/hwp3-sample10-hwp5-lineseg-normalized-2020.pdf)에 보존했다.
  SHA-256은 `a11af9041a0a116395f7647f8476d63f2da8e4c42277a895b6046db3042bf3f0`이다.
- 앞선 누적 code head에서 실행한 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`는 5,923건 통과, 37건 제외, 실패 0건이었다.
- 추가 세 commit의 원 head는 GitHub Full CI·CodeQL·Render Diff·Native Skia를 모두 통과했다.
  통합 후보에서도 `captioned_group_gets_hancom_numbering_bit_and_visited_caption`,
  `task1593_first_para_same_para_field_end_preserved`,
  `task1591v2_first_para_hidden_slot_char_shape_position`, `issue_3915_verify_axes_both_reported`를 실행해
  총 6개 test case를 통과했다.

원본에 이미 손상된 `LINE_SEG` 꼬리 네 건은 저장 과정의 정규화 대상이며, 본문 보존 실패로 판정하지 않았다.

**통합 수용 대상이다.** GitHub 상태는 merge 직전에 최신 head로 재확인한다.
