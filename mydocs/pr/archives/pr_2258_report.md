# PR #2258 최종 보고 — HWP5 취소선 모양 화이트리스트 (yeonic, rhwp 첫 PR)

- 결정: **merge** (2026-07-14, CI green 확인 후) — yeonic 첫 PR
- 사유: 스펙(표 25)·HWPX #154 대칭·실측(FAILED 실증 + 111쪽 렌더 동일)
  3중 근거 성립 + 작업지시자 시각 판정 통과.
- 게이트: 전수 3,160/0 · fmt/clippy 0 · OVR 5샘플 0건 · 원격 CI 11항목
  green (첫 기여자 워크플로 실행 승인 후).
- 파생 기록: 스펙 정오표 34번 항목 신설 (`tech/hwp_spec_errata.md`) —
  CharShape 취소선 비트 단독 판정 불가, 3D placeholder 관행.
- 시각 판정 자료: `output/poc/pr2258/visual/compare_3way.png` (before/after/hwpx).
