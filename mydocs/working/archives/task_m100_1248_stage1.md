# Stage 1 완료 보고서 — Task #1248 현황 맵

- **이슈**: edwardkim/rhwp#1248
- **단계**: Stage 1 / 3 (현황 맵)
- **산출물**: `mydocs/tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md` §1
- **코드 변경**: 없음 (조사 전용)

## 한 일

render/pagination/typeset 3개 레이어에서 trailing(line_spacing)을 다루는 **모든 지점을 전수 추출**하여 표로 정리.

## 핵심 발견

1. **trailing 은 단일 모델이 아니라 4개 레이어가 각자 다른 가정으로 취급** (조사문서 §0, §1.5):
   - typeset: IR `seg.line_spacing` 에 **굽고(bake)**, base-flow `1984HU` 는 vpos 에 이미 있다고 **가정**
   - pagination: fit 판정에서 **빼는 게 기본** + 7개 조건부 분기
   - render: vpos 연속이면 **포함**, 아니면 **bridge** + 7개(+#1247 8번째) 특례

2. **trailing 의 SSOT 부재**: 세 레이어가 단일 진실을 공유하지 않음. typeset 의 1984HU 가정과
   render 의 "vpos 연속 포함" 가정이 어긋나는 지점이 #1246 의 gap≈0 근본 구조.

3. **핵심 bake 지점 확정**: `typeset.rs:2219` `last_seg.line_spacing = between_notes` —
   trailing 을 IR 에 덮어쓰기로 기록. `typeset.rs:5819` `ENDNOTE_BETWEEN_NOTES_BASE_FLOW_HU = 1984`
   가 이중 경로(굽기 + vpos 초과분 가산)의 가정 상수.

4. **pagination 분기 7개 위치 확정**: engine.rs 512/521/547/1148/1155/1247/1860 (+참고 1884/2204).
   "언제 trailing 을 빼고 언제 안 빼는지"가 분기마다 다름(단일 규칙 아님).

5. **devel 기준 vpos_adjust 특례는 7종** (8번째 min-gap 은 PR #1247 미머지). 문서에 명시.

## 다음 단계 (Stage 2)

- vpos_adjust 특례 8종 각각의 트리거 조건·도입 이유·핀 고정 테스트 역추적 (§2)
- render gap ≠ pagination 예약 불일치 경로 정리 (§3)

## 승인 요청

Stage 1 현황 맵 승인 후 Stage 2 착수.
