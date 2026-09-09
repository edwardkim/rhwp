# Task M100 #1880 v2 최종 보고서 — HWP3-origin 휴리스틱의 HWPX-변환본 오발동 (2959953)

- 이슈: #1880 잔존 후속
- 브랜치: `local/task1880-v2` (base: origin/devel bf5228df)
- 계획서: `mydocs/plans/task_m100_1880_v2{,_impl}.md`
- 작성일: 2026-07-05

## 1. 결론

#1880 1차 수정(PR #1927) 후 잔존 PI_MOVED 중 최다 건(2959953, 5개 pi)의 원인을
확정·해소했다. 문단 flow 가 아니라 **페이지 기하**의 문제였다: HWP5 파스의
HWP3→HWP5 변환본 비율 휴리스틱(#554)이 rhwp HWPX→HWP 변환본에 오발동해
`margin_bottom -1600`(=body +21.3px)을 적용, HWPX 렌더와 페이지 채움이 어긋났다.

## 2. 원인

- `src/parser/mod.rs` `apply_hwp3_origin_fixup`: 문단>50 ∧ ps_ratio<0.05 ∧
  cs_ratio<0.15 → margin_bottom -= 1600 (한글97 마지막 줄 tolerance 모방).
- 2959953(저-스타일 대형 행정규칙)이 비율 조건 충족 → conv 재파스만 보정.
- HWPX 파스는 #1608 에서 동종 감지 제거 → 비대칭. `parse_hwp` 는
  `is_hwpx_variant`(#1886 마커)를 알면서도 휴리스틱을 무조건 실행했다.

## 3. 수정

`apply_hwp3_origin_fixup` 에 `is_hwpx_variant` early-return +
`summary_hwp3_era` 확정 경로(spacing 반감·is_hwp3_variant 설정)에도 동일 게이트.
결정론 마커가 비율 휴리스틱에 우선한다 (#1886/#1608 계열의 연장).
native HWP5·스트림 파스 진입점(variant 항상 false)은 불변.

## 4. 검증

- 단위: 비율 의심 합성 문서로 native 보정 유지 / 마커 시 불변 2건 통과.
- 2959953: body_area 양 경로 895.7px 일치, (section,pi)→page **1,888 entries
  완전 일치** (종전 5개 pi 이동).
- A/B 하니스 2,005건: SAME 2002→**2003**, 신규 divergence 0.
- 전체 스위트(`--tests --no-fail-fast`): 192 스위트 / 2,863 테스트 통과,
  신규 실패 0 (svg_snapshot 5건은 기존재 로컬 CRLF 노이즈 — 선행 태스크에서
  수정 전 상태와 A/B 동일 확인된 그 5건) / clippy `--all-targets` 경고 0.

## 5. 잔존 (별개 클래스 — 본 수정 전후 상세 동일)

- 3171755 PI_MOVED 1개 pi (s0:pi213 20→21).
- 3235145 PAGE_DELTA(3→2) — body_area 가설 불성립 확인(본 수정 무영향), 다른
  원인. 소규모 문서(3쪽)이므로 차기 착수 시 dump-pages 전체 diff 로 진입 권장.

## 6. 산출물

- 커밋: `261824c5`(Stage 1), `673e7f4a`(Stage 1/2 보고서), 본 보고서(Stage 3).
- PR: 단일 커밋 squash 후 `edwardkim/rhwp:devel` 대상 생성.
