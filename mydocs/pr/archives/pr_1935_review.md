# PR #1935 검토 — Task #1880 v2: HWP3-origin 휴리스틱을 HWPX-변환본 마커에서 게이트

- 작성일: 2026-07-05 / planet6897 → devel / 시간순 7/7 (06:10)

## 요지
`apply_hwp3_origin_fixup`(#554 비율 휴리스틱, margin_bottom −1600)과 `summary_hwp3_era`
확정 경로가 rhwp HWPX→HWP 변환본(is_hwpx_variant, #1886 마커)에 오발동 → 2959953 PI_MOVED
5개 pi. **결정론 마커 > 비율 휴리스틱** early-return 게이트 2곳.

## 검토
- #1608(is_hwp3_origin 오탐지)·#1886 계열의 원칙적 연장 — 케이스별 명시 가드 방침과 정합.
- 검증 촘촘: (sec,pi)→page 1,888 entries 완전 일치, A/B 2,005건 신규 divergence 0,
  양방향 단위 테스트(native 보정 유지 / 마커 시 불변) 2건.
- 잔존 2건(3171755/3235145)은 body_area 가설 불성립까지 확인해 분리 — 차기 진입점 기록.

## 판단
머지 권고. #1880 이슈는 잔존 2건 기록 유지(close는 작업지시자 판단).

## 결합 게이트 결과 (devel `d982067b` + 7건 시간순 통합, 2026-07-05)

fmt 통과 / clippy 경고 0 / `cargo test --profile release-test --tests` **2,889 통과·실패 0**
(신규 핀 11건 포함) / OVR baseline 5샘플 **개체 회귀 0건** (기준 00014ecf)
