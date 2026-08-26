---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5883 검토 - 선언 높이 표 축소의 내용 하한

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5883](https://github.com/edwardkim/rhwp/pull/5883) / [@kevin9327](https://github.com/kevin9327) |
| 관련 issue | [#5879](https://github.com/edwardkim/rhwp/issues/5879) |
| base / source head | `devel` / `a7f96ce76ab8ed22e9b41e98c181c0b4e8e619be` |
| 변경 규모 | 3 files, +178 / -0 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`, maintainerCanModify=true |
| 통합 반영 | `cbcbe82ca`, `81a2332f1`, `90d3a1ea7`, `4f29d3c62` |

## 범위와 검토

- 선언 높이 보정이 행의 실제 content 높이 아래로 줄어들면 비례 축소를 건너뛴다.
- 패딩만 줄이는 기존 #1510 허용 사례는 유지한다.
- regression은 p19의 정답 seam과 p20으로 이동한 문장, body clip 아래 무도형을 함께 고정한다.

## 검증과 시각 증적

- 통합 candidate `4b28259bb`에서 전체 nextest **8,160 passed, 39 skipped**, clippy, native-Skia, WASM build를 통과했다.
- 같은 head의 GitHub Build & Test, archive build/shard, Lint, Native Skia, CodeQL Rust/JavaScript/Python, Canvas visual diff, Adapter inter-diff, Proptest roundtrip도 성공했다. WASM과 frontend unit은 변경 범위 정책에 따른 정상 skip이다.
- `samples/issue4514/sample1-repro.hwp`와 `pdf/issue4514/sample1-repro-2020.pdf`의 p19-20 sweep은 `output/visual_open_prs_20260822_5879_a`에 보존했다.
- 대표 asset은 `mydocs/pr/assets/pr_5883_issue5879_p019_review.png`(SHA-256 `6d97f7c990d74e90e84fa42f81813390af90a59a23c41b15424d4465b090de17`)다.
- p19/p20 자동 후보는 모두 0건이며 visual accuracy proxy는 각각 58.053%, 53.738%였다. 비교 방식과 수치의 한계는 [visual sweep 가이드](../../manual/verification/visual_sweep_guide.md)에 따르며, 이 수치는 완전 fidelity 보증이 아니다.

## 최종 판정

**수용 권고.** 실제 content 하한을 도입하면서 패딩 압축의 기존 허용 범위를 보존한다. merge 전 PR #5889 최신 CI와 작업지시자 승인이 필요하다.
