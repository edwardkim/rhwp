---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6125 검토 - 되감긴 float/TAC host 페인트 순서 역전 보정

## 접수 메타데이터

| 항목 | 확인값 |
| --- | --- |
| 원 PR / 작성자 | [#6125](https://github.com/edwardkim/rhwp/pull/6125) / [@planet6897](https://github.com/planet6897) |
| 원 PR head | `80b166bd39605c574c91a6164c7236b587d2e22a` |
| 관련 issue | [#5700](https://github.com/edwardkim/rhwp/issues/5700), [#5701](https://github.com/edwardkim/rhwp/issues/5701) |
| 통합 PR | [#6159](https://github.com/edwardkim/rhwp/pull/6159) |
| 코드 후보 | `a6616441b21c6127999039419625a622f86db982` |
| 작성 시점 상태 | 코드 후보 CI 성공, 이 trailing 기록 head의 CI 대기 |

## 변경과 판단

- #5700: 되감긴 TAC partial paragraph 꼬리에 일반 PP y-reset을 적용하지 않아, 다음 흐름이 되감긴 꼬리보다 위로 올라가지 않게 한다.
- #5701: 되감긴 자리차지 표 host의 흐름 전진 하한을 페인트 하단으로 둬, 뒤따르는 본문이 표와 겹치지 않게 한다.
- 원 PR의 세 커밋을 보존해 통합했으며, 변경은 두 재현 fixture의 레이아웃 흐름으로 한정된다.

## 시각 증적

HWP 2024 MCP client를 `--engine 2020`으로 사용했다. private 원본 HWP는 Git에 넣지 않았다.

| issue | RHWP 논리 페이지 / Hancom PDF 대응 면 | 보관 자산 | 판정 사용 방식 |
| --- | --- | --- | --- |
| #5700 | p139 / p69 | [기준 PDF](../assets/pr_6159/baseline/issue5700-original/issue5700-original-hwp2020.pdf), [수동 대응 래스터](../assets/pr_6159/visual-sweep/issue5700-original-p139/issue5700-original-p139/manual-compare/) | 양면 PDF와 논리 페이지 체계가 달라 번호 동일성 proxy는 사용하지 않음 |
| #5701 | p76 / p38 | [기준 PDF](../assets/pr_6159/baseline/issue5701-original/issue5701-original-hwp2020.pdf), [수동 대응 래스터](../assets/pr_6159/visual-sweep/issue5701-original-p76/issue5701-original-p76/manual-compare/) | 동일 번호 76<->76 sweep은 진단용으로만 보관하고 판정에는 사용하지 않음 |

두 원본의 대응 텍스트와 페이지 체계 차이, 변환 엔진과 해시는 [증적 매니페스트](../assets/pr_6159/EVIDENCE.md)와 [SHA-256 목록](../assets/pr_6159/SHA256SUMS)에 기록했다.

## 검증

- suite manifest와 source unit tier 정책 검사를 통과했다.
- `cargo fmt --all -- --check`를 통과했다.
- #5700, #5701, #6126, #6128 관련 회귀 네 건을 통과했다.
- 전체 integration은 8,401건을 통과했다.
- Native Skia lib test와 WASM build를 통과했다.
- 코드 후보의 전체 CI는 [run 32986377307](https://github.com/edwardkim/rhwp/actions/runs/32986377307)에서 성공했다. PR 열기와 재열기 이벤트가 run을 만들지 않아, 동일 head에 workflow_dispatch 회귀 모드를 실행한 결과임을 기록한다.

## 최종 판정

**수용 권고, trailing CI 대기.** 최신 문서와 증적 head의 CI가 성공하면 #6159를 merge하고, 원 PR과 두 issue에 통합 위치와 검증 결과를 남긴다.
