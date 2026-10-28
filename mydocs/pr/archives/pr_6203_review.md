---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6203 review - #6185 자기 높이 음수 오프셋 무시

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6203
- 작성자: `planet6897`
- 원 PR head: `926848c3026a`
- 통합 검토 브랜치: `review/planet6897-6199-6217-20260827`
- 최신 기준: `upstream/devel@9d6f69b4d1a0`
- 검증 실행 기준: `upstream/devel@584320e0ee02`
- 원 PR 상태: non-draft, source CI green, comments/reviews 0건
- 관련 이슈: #6185

## 검토 판단

**수용 권고**. non-TAC 자리차지 사각형이 `vert=paragraph`, `Top/Inside`, `offset == -height`인 경우만
자기 변위 잔재로 보고 배치 오프셋을 0으로 클램프한다. 일반 음수 오프셋 전체를 무시하지 않고 정확한
지문으로 좁혀, 실제 의도된 음수 배치와의 충돌 위험을 낮췄다.

## 증적과 검증

- 원 PR 시각 보고서: `mydocs/report/issue-6185-self-displacement-offset/{before,after}.png`
- 검토자가 직접 확인한 대표 after: 로고 글상자가 담당부서 표 아래로 내려가 둘째 행을 덮지 않음
- 파일 버전 증적: `mydocs/pr/assets/pr_6199_6217_156570535_logo_box_self_displacement_hwpx_info.json`
- focused test: `issue_6185_self_displacement_vertical_offset` 1 pass
- #6203의 선행 commit `82a82d154b89`는 최신 `upstream/devel`에 이미 포함되어 통합 브랜치에서는
  중복 적용하지 않았다.
- 공통 검증: fmt, suite manifest, unit tier, clippy, 전체 nextest, Native Skia 3종, WASM build 통과.
  상세 명령과 숫자는 통합 구현 문서에 기록했다.
- 2026-08-28 최신 `upstream/devel@9d6f69b4d1a0`로 충돌 없이 rebase했다. 사용자 지시에 따라 별도
  중복 테스트는 수행하지 않았다.

## 후속

통합 PR에는 중복 commit 제외와 정확 일치 지문 기반 보정을 함께 적어, 원 PR head를 단순 수용하지 않은
이유를 남긴다.
