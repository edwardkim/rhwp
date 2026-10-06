---
kind: snapshot
status: active
canonical: mydocs/working/m06f_render_backend_fatten.md
last_verified: 2026-08-18
---

# M06-f render_backend 계약·픽스처 고도화

이슈 #5462. `src/renderer/**` 미수정. serializer 미수정. gym 미수정.

## 무엇을

devel 의 `RenderBackend`(Svg/Png/Skia + Null/Trace) 위에 계약 카탈로그·장면
빌더·정직성 표·픽스처 로더·상호 diff 요약을 얹고, 통합 시험 196 장면을
닫았다.

## 왜

M06-1~3 이 어댑터와 광고 정직성을 넣었지만, kind 전수·치수 사다리·plane
재정렬·형식 가족 skip 을 한 표로 재현하는 픽스처가 없었다. source-side
`#[test]` 는 총량 동결이라 `tests/cases/` 와 JSON 픽스처로 고도화한다.

## 어떻게

- `src/render_backend/catalog.rs` — 18 kind 표
- `scenes.rs` — 합성 장면 빌더
- `contract.rs` — 생명주기 스크립트
- `honesty.rs` — 광고 vs 실지원
- `fixture.rs` — JSON 스키마·최소 파서
- `diff.rs` — 가족 비교
- `tests/fixtures/render_backend/scenes/` — 196 JSON
- `tests/cases/render_backend_m06f_*.rs` — 통합 시험

## 검증

- `cargo fmt --all -- --check`
- `node scripts/rust-test-suite-manifest.mjs --check`
- `node scripts/rust-unit-test-tiers.mjs --check`
- `cargo test --lib render_backend::`
- `cargo test --test render_backend_m06f_catalog` 등 케이스
