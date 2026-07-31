---
kind: review-plan
status: active
---

# planet6897 통합 검토·반영 계획 (PR #3636, #3639, #3643, #3644)

기준은 `upstream/devel`의 `8a042f468418f641b7adda8c2f14310778f0905b`이며, 외부
기여자 PR 네 건은 개별 merge하지 않고 하나의 maintainer 통합 PR로 반영한다.

| 순서 | 원 PR | 원 head | 통합 후보 반영 |
| --- | --- | --- | --- |
| 1 | #3636 | `f2da30e0` | `6b1cf67c`·`070e4c9a`·`83e9c02d` |
| 2 | #3639 | `06c7b7a0` | `9853bc2c` |
| 3 | #3643 | `f83d9aaa` | `ac08b03c` |
| 4 | #3644 | `7d5a3e23` | `3a6479dc`·`5ebf12c5`·`992ebccb`·`e79e291a` |

## 보정과 검증

- maintainer 보정 `fac610dbe`: `dump-extents`의 0쪽 문서 범위 메시지 언더플로 방지,
  여러 text run에 걸친 오른쪽 정렬 말미 공백의 실제 style별 폭 계산.
- 새 HWPX fixture의 IR field sweep은 805개 sample에서 새 증가분 없이 통과했다. baseline에서
  사라진 기존 HWP5 rebuild 항목 2개는 이 PR이 건드리지 않는 serializer 영역이므로 갱신하지 않는다.
- #3639가 해결하는 것은 본문 줄의 쪽 밖 배치·소실이다. 한글 2020 PDF는 2쪽, rhwp는 3쪽이므로
  페이지 수 불일치는 범위 밖 후속으로 유지한다.

## 반영 순서

1. 통합 PR의 최신 head CI 및 mergeability를 다시 확인한다.
2. CI 성공 뒤 통합 PR을 squash merge한다. source fork branch는 삭제하지 않는다.
3. review 기록·visual asset·오늘할일을 docs-only 후속 PR로 fast-pass 검증·merge한다.
4. #3637은 통합 PR이 실제 반영된 뒤 close, #3386은 #3636의 `Refs` 범위이므로 자동 close하지 않는다.
5. contributor에게 통합 반영 사실과 검증 근거를 실제 줄바꿈 body-file로 알리고, upstream의
   integration/docs branch·로컬 review refs·전용 target만 절차에 따라 정리한다.
