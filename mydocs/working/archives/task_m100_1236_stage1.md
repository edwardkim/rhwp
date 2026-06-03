# Stage 1 완료 보고서 — Task #1236: 원인 특정·조사

- **이슈**: #1236 (M100)
- **브랜치**: `feature/issue-1236-endnote-line-spacing`
- **단계**: Stage 1 / 3 (코드 무변경, 조사)
- **작성일**: 2026-06-02

## 수행 내용

1. 증상 재확인: 미주 페이지(10~14쪽) 줄간격 좁음, 문제 페이지(1~9쪽) 정상(시각 대조).
2. 파싱 배제: `ir-diff` HWPX↔HWP → ParaShape line spacing 차이 0건(전부 indent).
3. 일반 문단 정상 확인: `RHWP_TYPESET_DRIFT_LINES` → `fmt==seg`, `diff=0`.
4. **원인 위치 특정**: 미주는 일반 `format_paragraph` 경로를 우회하고 `typeset.rs` L1885+
   미주 전용 repacking(LINE_SEG vpos-델타 누적 + 표 인접 rewind/overestimate 보정) 경로로 배치.
   드리프트 로그에 미주 문단 0건 포착 → 별도 경로임을 역으로 입증.

## 핵심 결론

- 미주 줄간격은 ParaShape 가 아니라 **원본 LINE_SEG vpos-델타**로 재구성됨.
- 표 인접 보정(`prev_endnote_had_vpos_rewind` / `..._inline_object_vpos_overestimate`)이
  특정 델타를 좁히는 것이 유력 — 사용자 지적(표 인접·간헐적 불일치)과 정합.
- 선행 #836/#1022/#1082 로 반복 패치된 **고난도·고회귀위험 영역**.

## 미해결 (Stage 2 이월)

- 정밀 under-spacing 지점: 미주 줄 렌더 Y 를 PDF 와 1:1 대조해 좁아지는 델타 분기 특정.
- black-box 픽셀 측정은 수식 첨자 노이즈로 부정확 → Y-좌표 직접 추출 진단 필요.

## 산출물

- `mydocs/tech/endnote_line_spacing_1236.md` (조사 상세)

## 승인 요청

원인 영역(typeset.rs 미주 repacking)과 "미주 한정·조건 게이트 + 회귀 가드" 접근을 확정해
주시면 Stage 2(정밀 지점 특정 + 수정)로 진행. 단, 페이지수/단배치 강결합으로 회귀 위험이
높아 신중한 진행이 필요함을 사전 보고.
