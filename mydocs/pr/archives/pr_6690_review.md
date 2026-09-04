---
kind: pr-review
pr: 6690
reviewed_at: 2026-09-04
source_head: c379257716458c30028dbd44f84ce8b463c0b96d
---

# PR #6690 검토 - 개체만 담긴 칸 마지막 줄의 꼬리 줄간격

## 판정: 승인

원 PR은 글자 없이 개체만 담긴 table cell의 마지막 줄에 꼬리 줄간격을 더하지 않도록
`src/renderer/height_measurer.rs`를 보정한다. 원 PR head
`c379257716458c30028dbd44f84ce8b463c0b96d`는 통합 후보에 다음 commit으로 적용됐다.

| 구분 | commit |
| --- | --- |
| 원 변경과 회귀 테스트 | `eb84bbbc7` |
| clippy 후속 정리 | `9f0455b6f` |

## 검토 범위와 결과

- 변경은 개체만 있는 마지막 줄로 한정되어 일반 텍스트 줄간격 처리에는 적용되지 않는다.
- 회귀 테스트 `issue_6681_cell_last_object_line_drops_trailing_ls`가 포함되어 있다.
- 원 PR의 required check는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6690/checks)
- 통합 후보의 로컬 호환/통합 검증은 다음 명령으로 성공했다. 공식 CI full lane 또는
  nextest 전체 검증을 대체하지 않는다.

```sh
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 시각 증적

이 변경도 `samples/exam_science.hwp`의 4쪽에서 원본 Hancom 2020 PDF와 현재
Studio 출력을 직접 비교했다. `#6683`과 같은 page evidence를 공유하지만, 판정은
개체-only 마지막 줄의 trailing line-spacing 계약에만 한정한다.

| 자료 | 경로 | SHA-256 |
| --- | --- | --- |
| Hancom 2020 기준 4쪽 | `../assets/pr_6683_6705_20260904/reference-6683-6690-exam-science-p4.png` | `42d4a5018d80272e07efb09e45eb9d556381fd5537d5ad08e44899020b596dbe` |
| 현재 Studio 4쪽 | `../assets/pr_6683_6705_20260904/studio-6683-6690-exam-science-p4.png` | `c2af0d6d027f4fda2909a282aa6e3d03b4a57914a54719c5da96d7196c81005b` |

## 메인터너 보정 판단

추가 메인터너 코드 보정은 필요하지 않다. 최종 병합은 통합 PR 최신 head의 required
CI와 mergeability를 다시 확인하는 일반 절차를 따른다.
