---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-22
---

# PR #7331 검토 기록 — 편람 쪽수 상쇄를 구조 랜드마크로 기록

- 원 PR: [#7331](https://github.com/edwardkim/rhwp/pull/7331)
- 관련 이슈: [#7009](https://github.com/edwardkim/rhwp/issues/7009) — **계속 열어 둠**
- 원 code head: `4397074291be4a0185cda69420162de1ec1c4806`
- 통합 검토 branch: `review/planet-open-20260922`
- 적용 commit: `17ab086cf` (`cherry-pick -x`, 충돌 없음)

## 변경과 범위

이 PR은 renderer를 고치지 않는다. `2025 행정업무운영 편람(최종)` HWP/HWPX의 총 쪽수만 같아도
본문과 부록의 편차가 상쇄될 수 있음을, 부록 간지·본문 마지막·간지 뒤 쪽수 랜드마크로 기록한다.
현재 알려진 편차를 수리하지 않고 움직임을 검출하는 회귀 검사다.

## 검증과 판정

- 저장소 기존 입력과 `pdf/2025 행정업무운영 편람(최종)-hwp-2024.pdf`를 사용한다.
- 신규 3건은 통합 후보의 focused nextest에서 통과했다.
- 제품 코드·렌더 출력 변경이 없어 Visual Sweep은 비대상이다.

**승인.** #7009의 본체 수리와 정답지 출처 확정은 이 PR의 범위 밖이므로 닫지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR의 **실제 merge SHA와 최종 head CI URL**이 확정된 뒤 원 PR에 한국어 감사 코멘트를 남긴다.
총 쪽수 상쇄를 구조 랜드마크로 검출하게 된 계약과 focused nextest 3건의 실제 통과를 설명한다.
렌더 출력 변경이 없으므로 Visual Sweep 이미지나 overlay는 포함하지 않는다. #7009의 본체 수리와
정답지 출처 확정은 남아 있어 PR comment에서 해당 이슈를 닫았다고 표현하지 않는다.


본문은 UTF-8 파일로 만든 뒤 `gh pr comment --body-file`로 게시하고, 게시 후 한국어 본문·실제 merge SHA·CI URL과 #7009를 닫지 않는 범위를 다시 확인한다.
