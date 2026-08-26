---
kind: pr-review
status: approved-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5836 검토 - renderer bughunt r6

## 판정과 메인터너 보정

- source head `57b91034412090db77ed92dbfea1b35af2b73293`를 적용했다.
- #5802/#5807/#5808 보정은 수용한다. #5804는 #5826과 중복되므로 #5836의 최종 revert가 선택한 #5826 구현을 유지했다.
- 다만 그 revert가 오래된 SVG golden까지 복원해 통합 full nextest를 실패시켰다. 메인터너가 #5826의 현 출력과 바이트 동일한 golden을 되돌려 `issue_677` 회귀를 제거했다. 이는 중복 코드 수용이 아니라 충돌 후 stale test-data 복구다.

## 검증과 시각 증적

- `issue_5802_hf_cross_section_inherit`, `issue_5807_coanchored_float_tac_order`, `issue_5808_square_group_left_outer_margin`은 각각 1/1 통과했고, 통합 전체 nextest 8,109/8,109 통과.
- HWP 2020 기준 `pdf/pr_open_20260821/coanchored_float_tac_order-2020.pdf`는 job `4cf3d15e-f193-4e6b-8699-d695c76d340d`, validation ok, 3 pages, SHA-256 `3b287fc09387d39501658a06925a9fc13efa3aca3cce260c77cadb3400bc9782`다.
- p3에서 pixel 96.20787%/proxy 4.05482%이며, 사람 대조에서 최종 의견 상자의 순서는 보존됐다. typography/text flow 충실도는 잔여사항이며 대표 이미지는 `mydocs/pr/assets/pr_5836_issue5807_review_003.png`다.

## 최종 판단과 GitHub 기록

- **수용, fidelity 후속 보류**: #5844 전체 CI와 #5802/#5807/#5808 계약은 성공했다. stale golden 복구는 이미 적용한 메인터너 보정이며 추가 코드 보정은 없다. p3 typography/text-flow fidelity는 후속 과제로 **보류**하되, co-anchored table 순서 계약을 막지는 않는다.
- merge 뒤 원 PR에는 #5826과의 중복 #5804 처리, stale golden 보정, r6 세 계약 수용과 fidelity 후속 보류를 모두 comment로 남기고 close한다.
