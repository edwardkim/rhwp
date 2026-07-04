# Task #1705 검증용 한글 문서 (어울림 표 트레일링 빈 문단 앵커-페이지 귀속)

어울림(floating) 표 옆 wrap zone 의 빈 문단이 표의 **앵커(첫) 페이지**에 놓여야 하나, #1700 의
last-page 귀속으로 표 끝 페이지에 잘못 놓이던 회귀 방지용 표본.

| 파일 | 원본 ID | 특성 | 기대 |
|------|---------|------|------|
| `wrap_empty_para_anchor_page.hwp` | 14504219 | 9행 어울림 표(2쪽 시각확장) + 좁은 sw 빈 문단 | 빈 문단(pi=2)이 표 앵커 **page 1** 에 귀속. 한글 MATCH. |

대비 케이스(전체폭 → 표 끝 페이지)는 `samples/task1700/myeonjeok_wrap_10page.hwp`(2067603) 참조.

판별: 빈 문단 `line_seg.segment_width` 가 body 폭의 90% 미만(wrap zone)이면 앵커(첫) 페이지,
아니면 표 끝(마지막) 페이지.
