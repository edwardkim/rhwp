# PR #2365 검토 — 생성기 ladder sb-누락 후방스냅 보정 + 서브픽셀 하니스 (#2279, 스택 3)

- PR: https://github.com/edwardkim/rhwp/pull/2365 (planet6897)

## 변경 본질

1. **서브픽셀 정렬 하니스 신설**: rhwp PDF ↔ 한글 PDF 의 baseline origin +
   괘선 프레임을 내용 순서 정렬해 anchor 간격 Δ를 0.1pt 해상도로 산출 —
   페이지 분할이 달라도 발산을 문단 단위로 국소화하는 조사 도구
2. 하니스가 찾은 결함: 본문 수십 문단 **일률 Δ−5.0pt = ladder 의 sb-누락**
   (생성기가 spacing_before 를 ladder 에 안 넣음 → rhwp 가산분을 후방
   스냅이 정확히 되감음)
3. 규칙: 후방 스냅량 ≈ 해당 문단 sb(±2px)면 스냅 무시 — HWPX 한정.
   정상 문서는 스냅량≠sb 라 불변 (조건 코드 확인)

## 로컬 재실증 (merged tree)

전체 스위트 0 실패 · svg_snapshot 8/8 · 시장 313 · prep 145 · byeolpyo 유지
· fmt/clippy 0 · 충돌 0

## 판단

**merge 권고.** 도구를 먼저 만들어 결함을 계측으로 찾고(0.1pt 해상도),
좁은 저장 필드 조건으로 정정 — 비기능 투자가 기능 정정을 견인한 표본.
