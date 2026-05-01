# PR #374 처리 보고서 — close (흡수 가치 없음)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#374](https://github.com/edwardkim/rhwp/pull/374) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| 이슈 | [#362](https://github.com/edwardkim/rhwp/issues/362) (이미 CLOSED) |
| 처리 결정 | **close + 흡수 가치 없음** |
| 처리 일자 | 2026-04-27 |

## 처리 결정

PR #374 의 결함 (kps-ai p56 외부 표 클립) 은 메인테이너 **Task #362** (8 항목 누적 수정) 로 v0.7.7 에 이미 정정 완료. 이슈 #362 CLOSED.

## 두 접근의 비교

| 항목 | PR #374 | 메인테이너 Task #362 |
|------|---------|--------------------|
| 접근 | `vpos.min(remaining_room)` (측정 의존 일반화 clamp) | `if !has_nested_table` (구조 명시 가드) |
| 정정 범위 | p56 단일 case | p56 + p67 + p68-70 + p72-73 (8 항목) |
| 시멘틱 | 암묵적 (측정 의존) | 명시적 (구조 의존) |
| 회귀 위험 | 측정 결함 발생 시 clamp 결과 부정확 | 측정 의존 없음 |

## 흡수 가치 평가

| 가치 | 흡수 가능성 |
|------|----------|
| 모듈 추출 | 없음 (단일 6 줄) |
| 회귀 테스트 신설 | 없음 |
| 다른 경로 정정 | 없음 |
| 시멘틱 가치 | 메인테이너의 명시 가드보다 약함 |

→ **흡수 가치 없음**.

## 한컴 호환 원칙 (메모리 등록)

작업지시자 관측 — **한컴 호환은 일반화보다 케이스별 명시 가드가 안전**. 한컴 자체의 비일관성으로 일반화 알고리즘이 다른 케이스에서 회귀 발생. PR #374 의 일반화 clamp 접근은 이 원칙과 배치.

상세: `~/.claude/projects/.../memory/feedback_hancom_compat_specific_over_general.md`

## 처리 단계

1. 검토 문서 작성 (`pr_374_review.md`)
2. PR 댓글 + close
3. 본 보고서 작성
4. local/devel commit + push

## 작성자 attribution

PR 의 결함 진단 (Task #347 의 vpos 적용이 콘텐츠 꽉 찬 케이스 부작용) 정확. 다만 정정 접근이 메인테이너의 명시 가드보다 회귀 위험 높음.

## 참고

- 검토 문서: `mydocs/pr/pr_374_review.md`
- 메인테이너 Task #362: `mydocs/report/task_m100_362_report.md`
- 메모리: `feedback_hancom_compat_specific_over_general.md`
