---
kind: pr-review
status: active
---

# PR #3644 review — dump-extents 조사 CLI

| 항목 | 값 |
| --- | --- |
| 작성자 / base | planet6897 / `devel` |
| head 참고값 | `7d5a3e2380f4b3a937f268251c6d02818a342ec7` |
| 관련 이슈 | #3637 |
| 권고 | 통합 PR로 반영 |

SVG의 테두리 없는 표를 빈 공간으로 오판하지 않도록 render tree bbox를 직접 덤프하는 CLI다.
`--gaps`는 TextLine과 text 자손이 없는 표만 사용해 컨테이너 masking을 피한다. CodeQL
오탐을 피하려고 전역 인증 pre-scan 반환 경로에서 비밀번호를 제거한 변경도 CLI unit 2건으로
확인했다. 통합 보정은 0쪽 문서에 `-p 0`을 주었을 때 오류 메시지 산술이 언더플로하지 않게 한다.
