# Task M100 #1451 2단계 완료보고서 — 회귀 검증 + 보존 가드

- 이슈: #1451
- 브랜치: `local/task1451`
- 작성일: 2026-06-21
- 단계: 2/3 (회귀 검증 + Polygon 보존 가드)

## 1. 신규 보존 가드 테스트

`src/serializer/hwpx/roundtrip.rs` `task1451_legacy_shape_comment_serialize_roundtrip` 추가.

- 기존 `task1392_shape_comment_loss_in_gate` 는 IR diff **검출**만 가드(Ellipse loss).
- 본 테스트는 Polygon description 을 serialize_hwpx → parse_hwpx 왕복시켜 **보존 성공**을 직접 가드.
- 결과: **통과** (`다각형입니다.` 왕복 보존 확인).

## 2. 게이트 + 회귀 검증

| 검증 | 결과 |
|---|---|
| `serializer::hwpx::roundtrip` 모듈 (task1392 게이트군 포함) | **49 passed / 0 failed** |
| 보존 fixture `aift.hwpx` | diff 0 (PASS) |
| 보존 fixture `tac-img-02.hwpx` | diff 0 (PASS) |
| 보존 fixture `business_overview.hwpx` | diff 0 (PASS) |
| `hwpx_roundtrip_baseline` 게이트 (4 테스트) | **4 passed** (baseline / large / xfail / grade) |

회귀 없음. 기존 picture/equation/rectangle/container 경로 무영향 확인.

## 3. 다음 단계

3단계: 전체 `--tests` + fmt --check + clippy → 최종 보고서 + 오늘할일 갱신.
