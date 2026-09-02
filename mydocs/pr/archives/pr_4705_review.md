---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4705 검토 - overlay 표의 다음 쪽 잔여 행 조각

| 항목 | 기록 |
| --- | --- |
| PR | [#4705](https://github.com/edwardkim/rhwp/pull/4705) |
| 작성자 / 원 head | @planet6897 / `b75e28b28dc7f1d477f4f1c7efe0f603297c2a8f` |
| 적용 commit | `e78b81c0f`~`7b5dd454c` 5개 |
| 통합 후보 | `c7cfaefb9` |

페이지를 넘는 overlay 표에서 앞 쪽은 cut까지 그리고, 잔여 행 조각은 다음 쪽 상단의
`ColumnContent`로 별도 보관·페인트한다. `PageItem`에 섞어 first-item 휴리스틱을 흔들던 중간 구현은
적용하지 않고, 별도 목록 설계를 유지했다.

## 메인터너 보정

`49add03c3`은 두 페이지 연속 조각의 상하 위치와 내용 소유를 직접 검증하도록 회귀를 강화했다.
`d9868b0b4`은 기존 pagination test의 새 연속 메타데이터 초기화를 명시해 all-targets clippy 및
테스트 빌드가 구조체 필드 누락으로 실패하지 않게 했다. 두 보정 모두 renderer의 출력 정책을 바꾸지
않고, 누적 적용에서 드러난 테스트 계약을 완성한 것이다.

## 시각 증적과 잔여 범위

`samples/issue4514/sample1-repro.hwp`와 한컴 2020 기준 PDF를 비교했다. rhwp p9의 앵커 조각과
p10의 잔여 조각은 연속으로 보존됐다.

- [한컴 p9](../assets/pr_4705_issue4568_hancom2020_p009_review.png), [rhwp p9](../assets/pr_4705_issue4568_rhwp_p009_review.png)
- [한컴 p10](../assets/pr_4705_issue4568_hancom2020_p010_review.png), [rhwp p10](../assets/pr_4705_issue4568_rhwp_p010_review.png)

문서는 한컴 46쪽, rhwp 48쪽으로 전체 쪽수 차이가 남아 있다. 이 PR은 overlay 잔여 행 소유와
잘림을 고친 것이며 전체 layout fidelity 완료를 주장하지 않는다. 잔여 쪽수·배치 보정은
[#3820](https://github.com/edwardkim/rhwp/issues/3820)에서 계속 다룬다.

## 완료한 검증

- `issue_4514_overlay_table_flow` focused 회귀와 `overflow_cell_baseline`을 통과했고, baseline ratchet은 62에서 0으로 내려갔다.
- 누적 후보 전체 `nextest`는 5,923건 통과, 37건 제외, 실패 0건이었다.
- Native Skia library 58건 및 `issue_2225_missing_picture_placeholder` 2건도 통과했다.

**부분 범위 통합 수용 대상이다. #4568 close는 남은 전체 fidelity 확인 뒤에만 판단한다.**
