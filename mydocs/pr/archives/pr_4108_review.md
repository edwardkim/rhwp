---
kind: pr_review
status: accepted-with-maintainer-correction-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4108 검토 - 로드맵 진행률 기계 산출

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4108](https://github.com/edwardkim/rhwp/pull/4108) / @kevin9327 |
| 원 head | `4c9f79d455e5f081fc378682f4ad50bfd42063fe` |
| 규모 | 5개 파일, +321/-14 |
| 원격 참고 상태 | `MERGEABLE` / `CLEAN`, 원 head CI·CodeQL 성공 |
| 시각 검증 | 비대상. Python 집계기와 로드맵 문서만 바꾼다. |

`tools/roadmap_progress.py`가 R1~R100 태그, 등급 어휘, README 생성 블록을 검증하고 `--write`로
집계를 재생성한다. 손집계와 canonical 트랙 문서의 드리프트를 막는 범위다.

## 발견 사항과 보정

원 변경은 R94를 구현했지만 설계 문서 머리말이 여전히 "R94 [가설], 구현이 없다"고 기록했다.
또한 #4114의 R7, #4116의 R74·R79가 구현된 누적 상태를 README 기계 집계가 반영하지 못했다.

통합 보정은 R94 설명을 P2 구현 완료로 고치고, R7·R74·R79의 canonical 상태를 완료로 정렬한 뒤
`python3 tools/roadmap_progress.py --write`로 README를 재생성했다. 결과는 `완료 31 · 실측 9 · 문서 7 ·
이슈 4 · 가설 49 = 100`이다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `python3 tools/roadmap_progress.py --write` | README 생성 블록 재생성 성공 |
| `python3 tools/roadmap_progress.py` | 결번·중복·등급·README 집계 검증 성공 |
| `git diff --check` | 성공 |

Cargo 전체 회귀는 Python·문서 보정 범위이므로 실행하지 않았다.

**메인터너 보정 포함 통합 수용.** 상태 정렬은 #4114와 #4116의 기능을 전제로 하므로 개별 source
branch에 나누어 push하지 않고 통합 PR에 원자적으로 포함한다.
