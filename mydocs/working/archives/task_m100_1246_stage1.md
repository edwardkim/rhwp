# Stage 1 완료 보고서 — Task #1246: render/pagination 미주 vpos 좌표 정합 (코드 무변경)

- **이슈**: #1246 (M100) — #1238 흡수
- **브랜치**: `feature/issue-1246-endnote-vpos-anchor`
- **단계**: Stage 1 / 4 (코드 무변경, 조사)
- **작성일**: 2026-06-02

## 핵심 결론

1. **수행계획서 프레이밍 정정**: "render 가 누적 incoming y 로 앵커한다"는 부정확. render 는 이미
   `HeightCursor.vpos_adjust`(Task #1027 Stage C)로 **vpos→y 매핑**을 수행한다. → 전면 좌표
   재작성 불필요.
2. **drift 진짜 원인**: render(`HeightCursor`)와 pagination(`compute_en_metrics`)이 **분리된 두
   vpos 측정기**다(Stage D 미완 — pagination 이 공유 커서를 안 씀). 두 공간 불일치가 #1238
   pi=475 +7.6px drift 의 근원.
3. **HeightCursor 는 이미 between-notes 처리**: compact endnote 분기에 forward-cap(1984=7mm),
   stale-forward, backtrack, **`compact_endnote_safe_vpos_backtrack`(=#1209 문12)** 등 다수 특례
   보유(단위테스트 28+). #1238 render 클램프는 이 분기를 침범해 문12 회귀를 냈다.
4. **사각지대**: HeightCursor 는 gap 이 **과도할 때 줄이는** 로직만 있고, **부족할 때(stored
   vpos gap < between_notes, 특히 0=문22) 끌어올리는 min-gap 로직이 없다**. 이것이 #1238 문22 버그.

## 수정 위치 (Stage 2 방향, 국소)

`HeightCursor.vpos_adjust` compact endnote 분기에 **min-gap 케이스 추가**:
- 새 미주 첫 문단이 forward 흐름 & `end_y − prev_content_bottom < between_notes_px` & **비-backtrack**
  이면 `end_y = prev_content_bottom + between_notes_px`.
- backtrack/rewind/stale-forward 기존 분기 우선 → #1209 무회귀.
- `between_notes_px` 셋업 주입 + `shift_vpos_base_for_rendered_delta` 로 base 이동.
- pagination(`compute_en_metrics`) 도 동일 gap 예약 → pi=475 overflow 방지.

## 산출물

- `mydocs/tech/endnote_vpos_anchor_1246.md` (좌표 구조 + HeightCursor 특례표 + 수정 위치 + 게이트).

## Stage 2 방향 (승인 요청)

전면 좌표 재작성이 아니라 **HeightCursor.vpos_adjust 의 compact endnote min-gap 추가 + pagination
정합**(국소). 수행계획서의 단계 구성은 본 발견에 맞춰 **구현계획서에서 재구성**한다.

원인·수정 위치·게이트 확정 시, 구현계획서 작성 → 승인 후 Stage 2 착수.
