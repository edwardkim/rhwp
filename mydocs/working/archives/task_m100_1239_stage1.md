# Stage 1 완료 보고서 — Task #1239: 인라인 수식 줄 배정 원인 특정

- **이슈**: #1239 (M100)
- **브랜치**: `feature/issue-1239-equation-multiline-merge`
- **단계**: Stage 1 / 3 (코드 무변경, 조사)
- **작성일**: 2026-06-02

## 수행 내용

진단 로그(`DBG_EQM`/`DBG_EQM2`)로 문20 pi=602 의 인라인 수식 줄 배정을 추적해 근본 원인 특정.

## 핵심 결론

- 문20 S= 블록은 단일 미주 문단(pi=602), 5 LINE_SEG·vpos 정상, 수식은 인라인 객체.
- **tac_ci 3·4 가 같은 char position(2)** 를 받아 같은 줄(line 3)에 배정 → 병합, line 2 공백.
- 원인: `model/paragraph.rs::control_text_positions` (L861-867) — **한 char_offsets 갭의 여러
  컨트롤이 모두 같은 position(i+1)**. 연속 인라인 수식(사이 텍스트 char 없음)을 한컴은
  LINE_SEG 로 별도 줄에 두나, char 위치 기준 줄 배정(`tac_offsets_for_line`)이 같은 줄로 병합.

## 산출물

- `mydocs/tech/endnote_inline_eq_line_1239.md` (상세 + 진단 표).

## 승인 요청

원인(연속 인라인 수식의 LINE_SEG 경계 미반영 줄 배정)과 Stage 2 방향(줄 배정에서 LINE_SEG
로 연속 TAC 분배, 일반 문단 회귀 게이트 + 골든 가드)을 확정해 주시면 Stage 2 착수.
