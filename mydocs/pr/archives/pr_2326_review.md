# PR #2326 검토 — KBU=1 글자 단위 줄바꿈의 행두 금칙 retraction (#2244)

- PR: https://github.com/edwardkim/rhwp/pull/2326 (planet6897)
- 이슈: #2244 (Closes) — 편집 reflow 의 마침표 고립
- base=devel 단독, composer 한정 (레이아웃 PR 들과 무충돌)

## 변경 본질

`korean_break_unit=1` 문단의 char_break 분기에서 초과 문자가 행두 금칙
(마침표 등)일 때 무조건 분리하던 것을, 한컴 오라클("적용한 | 다.111…")
대로 **직전 글자 1자를 함께 이월(retraction)**:

- 조건: 같은 run(스타일 경계 아님) + 직전 글자 비공백·비금칙 + 줄 2자 이상
- 구조: flush 전 pop(현재 줄에서 제거) → flush 후 새 줄 run 에 재주입 →
  금칙 문자 후속 — LINE_SEG char_start 정합은 테스트로 단언

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| composer 단위 (신규 포함) | 67/67 |
| 핀 | byeolpyo 4/26, 시장 312, 연결맵 385 유지 |
| `cargo test --tests` | 실패 0 / fmt 통과 |

## 판단

**merge 권고.** 한컴 저장 오라클 기반의 최소 조판 규칙 정정. 오늘 처리한
6건 큐의 마지막 건.
