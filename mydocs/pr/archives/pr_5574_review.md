# PR #5574 검토 기록

- PR: <https://github.com/edwardkim/rhwp/pull/5574>
- 작성자: `planet6897`
- 관련 이슈: #5563, #1893
- base / 원 head: `devel` / `2125df67f9409f8306831c48c166910f4638498e`
- 누적 적용: `fbace2ffb` (`review/planet6897-20260819`)
- 메인터너 보정: `f9c34fbc8`
- 공통 기록: [planet6897_20260819_integration_review.md](planet6897_20260819_integration_review.md)

## 변경 검토

HWPX/HWP5 저장기에 공통인 `line_segs_within_text_axis` 계약을 HWPX 저장과 IR diff에 적용해
문단 축을 넘는 stale `textpos`가 한글 개방을 무한 대기시키는 문제를 막는다. 원 PR의 네 계약
테스트는 범위 밖 접두부 절단, 경계값 보존, 전체 linesegarray 생략, IR diff 일치를 검사한다.

누적 검토에서 빈 누름틀 안내문이 DocumentCore IR에서는 비어도 직렬화 때 다시 나오는 경로를 발견했다.
원 상한만 쓰면 정상 `textpos=25`까지 잘려 #1893 셀 배치가 변했다. 메인터너 보정은 fieldEnd 직전
복원한 안내문의 UTF-16 폭을 실제 직렬화 축에 더하고, 같은 `tests/cases` 파일에 #1893 회귀를 추가한다.

## 판정

원 PR의 안전 목적과 보정 방향은 타당하다. 다만 보정 뒤 선택 nextest가 완료 요약 없이 종료되어
새 #1893 회귀의 PASS를 확정하지 못했다. 재실행 금지 지시에 따라 최종 수용과 merge는 **보류**한다.
최신 후보에서 이 선택 회귀와 corpus ratchet 실패의 원인을 확인하고 CI가 통과할 때만 수용으로 전환한다.
