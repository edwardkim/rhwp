# Task #1880 v2 Stage 1 — HWP3-origin 휴리스틱 HWPX-변환본 게이트

## 구현 내용

- `src/parser/mod.rs`:
  - `apply_hwp3_origin_fixup` 진입부: `if doc.is_hwpx_variant { return; }`
    — 비율 휴리스틱(margin_bottom -1600)이 rhwp HWPX→HWP 변환본에 오발동
    하던 것을 결정론 마커(#1886)로 차단.
  - `summary_hwp3_era` 확정 경로: `&& !doc.is_hwpx_variant` 추가 —
    `is_hwp3_variant=true` + ParaShape spacing 반감의 오발동 가능성 차단
    (원본 HWPX 가 HWP3-계보 요약정보 승계 시).
  - 스트림 파스 진입점(extra_streams 부재, variant 항상 false)은 불변.

## 테스트

- 신규 2건 (`src/parser/mod.rs` tests):
  - `issue1880v2_hwp3_fixup_applies_to_native`: 비율 의심 합성 문서(문단 60,
    ps/cs 각 1) — native 는 종전대로 margin_bottom 4252→2652 보정.
  - `issue1880v2_hwp3_fixup_skipped_for_hwpx_variant`: 동일 문서 + 마커 —
    margin_bottom 4252 불변 (오발동 금지).

## 검증

```bash
cargo test --lib parser::tests::issue1880v2   # 2 passed
```
