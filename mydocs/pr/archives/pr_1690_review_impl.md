# PR #1690 처리 계획 — #1688 후속 통합 cherry-pick 반영

## 대상 커밋

원 PR #1690의 merge commit은 제외하고 실제 변경 커밋만 통합 브랜치에 순서대로 cherry-pick 했다.

| 원 커밋 | 통합 브랜치 커밋 | 내용 |
|---|---|---|
| `d72357dea` | `9099c656a` | 중첩 표 높이 double-count 제거 |
| `6cae94a67` | `2dbc186a2` | round 3 계획/진단/보고 문서 |
| `aec4ad8a3` | `45954cb81` | #1690 검증용 문서 보강 |
| `ea89716a0` | `dc7e66c9f` | `valign_offset_gate.py` 누락 exit 1 보정 |

## 처리 단계

1. #1688 커밋을 먼저 통합 브랜치에 cherry-pick
2. #1690 커밋을 이어 cherry-pick
3. #1688/#1690 개별 리뷰 문서와 오늘할일 기록 추가
4. 로컬 통합 검증 수행
5. 통합 PR 생성
6. 통합 PR #1712 merge 후 원 PR #1690 close/comment 완료

## 후속 코멘트

#1690에 게시한 코멘트:

```text
@planet6897 반영 감사합니다.

PR #1690의 실제 변경 커밋 4개를 #1688과 함께 통합 cherry-pick PR #1712로 반영했습니다.

- d72357dea -> 9099c656a
- 6cae94a67 -> 2dbc186a2
- aec4ad8a3 -> 45954cb81
- ea89716a0 -> dc7e66c9f

통합 PR #1712는 GitHub Actions 통과 후 merge 완료되었습니다.
merge commit: b7d76030b5b0a54435e6d1237de976e45ffd3aba

통합 브랜치에서 #1688 선행 후 #1690을 이어 적용해 로컬 검증했고,
`valign_offset_gate.py`의 정상 fixture와 누락 실패 처리도 확인했습니다.
이 PR은 통합 PR에 포함되어 supersede 처리하겠습니다.
```

- 게시 URL: https://github.com/edwardkim/rhwp/pull/1690#issuecomment-4845818372
- close 완료: `2026-06-30T16:38:32Z`
