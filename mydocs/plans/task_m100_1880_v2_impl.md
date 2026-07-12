# Task M100 #1880 v2 구현계획서 — HWP3-origin 휴리스틱의 HWPX-변환본 게이트

- 이슈: #1880 잔존 (2959953 계열)
- 브랜치: `local/task1880-v2`
- 수행계획서: `mydocs/plans/task_m100_1880_v2.md`
- 작성일: 2026-07-05

## 방침 확정 (수행계획서 3절 후속)

`summary_hwp3_era` 확정 경로(mod.rs:317-348)도 게이트에 **포함**한다:
`is_hwp3_variant=true` 설정 + ParaShape spacing 반감으로 ratio 보정보다 더
파괴적이며, 원본 HWPX 가 HWP3-계보 요약정보를 승계하면 rhwp 변환본에도
오발동할 수 있다. 마커는 결정론이므로 두 휴리스틱 모두에 우선한다.

## Stage 1 — 게이트 + 단위 테스트

- `src/parser/mod.rs`:
  - `apply_hwp3_origin_fixup` 진입부 `if doc.is_hwpx_variant { return; }`
    (mod.rs:590 스트림 진입점은 variant 항상 false → 불변).
  - `if summary_hwp3_era` → `if summary_hwp3_era && !doc.is_hwpx_variant`.
  - 근거 주석 (#1880 v2 실측, #1886/#1608 계열).
- 테스트: 마커 스트림 포함 HWP5 를 in-memory 구성(serialize→parse 또는
  extra_streams 주입) — ratio 조건을 충족해도 margin_bottom 불변 확인.
  기존 #554/#1001 native fixture 테스트 회귀 없음.

## Stage 2 — 실측 검증

- 2959953: body_area 양 경로 일치 + PI_MOVED 5건 해소.
- 3171755 / 3235145 재측정 (동일 클래스 여부 판정).
- A/B 하니스 2,005건 (수정 전 = 통합 빌드, 수정 후 = 통합+본 수정).

## Stage 3 — 전체 회귀 + 최종 보고서

- `cargo test --tests --no-fail-fast` + clippy.
- 최종 보고서 `task_m100_1880_v2_report.md`, orders 갱신, PR 생성(단일 커밋).
