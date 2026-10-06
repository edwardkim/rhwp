# #5524 Stage 1 — 한글 2024 조판 호환 플래그(Δ1) 도입

## 무엇을

한글 편집기 세대별 조판 차이를 흡수하는 opt-in 레이아웃 호환 플래그를 도입하고, 실측으로
확정된 첫 번째 규칙 델타(Δ1: 자리차지 표 앵커 문단의 선행 앵커 줄 세그먼트 계상)를 구현했다.

- `LayoutCompatibilityProfile::hangul2024_layout()` + `with_hangul2024_layout()`
  (`src/model/provenance.rs`) — 기본 false = 현행 2022 계열, 규약대로 profile 질의로만 분기.
- `DocumentCore::set_hangul2024_compat()`(전 구역 dirty + 재페이지네이션) +
  `effective_layout_profile()` — 조판·레이아웃 profile 공급 4지점을 유효 프로필로 교체.
  세션 설정이므로 provenance(파서 확정 값)에 넣지 않았다.
- CLI: `dump-pages --compat 2022|2024`. 자동 감지는 하지 않는다 — HWPX `appVersion` 기반
  추정은 과거 오탐 회귀로 제거된 이력이 있다(`parser/hwpx/mod.rs` 주석).

## 왜

- 10k 전수 대조(`mydocs/report/hangul_version_oracle_r1_20260807.md`)에서 한글 2022↔2024 가
  다르게 조판하는 문서 247건(2.47%)을 확정했고, Discussion #4137 에서 정답지 이동을 논의 중.
- 후속 3자 대조 실측: rhwp 는 2022 쪽 152 : 2024 쪽 35 로 확고한 2022 계열 — 정답지 교체는
  즉시 ~150건 회귀. 플래그로 양쪽을 다 목표로 만드는 것이 제4안.
- 재저장(SaveAs) 오라클로 Δ1 을 특정: 같은 문서를 2022(12.0.0.4547)·2024(13.0.0.564)로
  재저장하면 앵커 문단 lineseg 가 2개(선행 줄+표 밴드) vs 1개(밴드만)로 갈린다.

## 어떻게 (typeset Δ1)

1. `typeset_tac_table` 의 `table_height` 사슬에 compat 분기 — 선행 앵커 줄 세그
   (`tac_seg_idx > 0`)를 fit 에서 제외하고 회수량을 `TypesetState::hangul2024_reclaimed` 에
   적립. 리셋은 단/쪽 전환 지점(`advance_column_or_new_page`/`reset_for_new_page`) —
   `flush_column` 에서 리셋하면 Square 밴드 마감이 카운터를 지운다(실측 함정).
2. 저장 vpos 쪽-경계 신호 3지점(리셋 트리거·되감김·분할 경로 near-top 리셋)에 재적합 술어:
   회수분이 있고 문단(빈 문단 need=0, 실문단 첫 줄)이 회수 보너스 안에서 들어가면 2022 조판이
   남긴 경계를 덮는다. 신호를 덮은 그 빈 문단만 place 적합을 우회(blank-spill).
3. 반증으로 기각한 설계(회귀 방지 기록): 전역 available 인플레이션(1회성 공간 연쇄 재사용
   과적재), sticky 연쇄 오버라이드(반쯤 찬 쪽의 의미적 저장 리셋까지 덮어 파탄 0.92→0.53),
   이웃 빈 문단까지 spill(한 문단 과적재).

## 검증 실측

- **프로브 게이트** (BREAK_DIFF×EXACT_22 41건 — r1 코호트에서 rhwp==2022 완전일치·2024만
  다른 문서, 한글 2024 재저장 지문과 COM-free 대조): `--compat 2024` 에서
  **IMPROVE 10(완전일치 8) / SAME 30 / REGRESS 0**. upstream/devel e5ef2620b 리베이스 후 동일.
- **기본값 무회귀**: 플래그 없이 247건 코호트 지문 재생성 → 도입 전과 **diff 0행**
  (리베이스 후 재확인 포함).
- fmt --all --check / clippy -D warnings / 매니페스트·티어 검사 통과, 통합 테스트
  `issue_5524_hangul2024_compat` 추가(샘플 `samples/issue5524_hangul2024_compat_letterhead.hwp`,
  공공 보도자료).
- 측정 원자료: 로컬 `output/poc/hangul_version_compat_phase0_20260818/` (Phase 0~2 REPORT.md,
  재저장본, 게이트 스크립트).

## 알려진 한계 (후속 축)

- Δ2(텍스트 재래핑 — 글꼴 메트릭), Δ3(줄간격 계상)는 미구현.
- 연쇄 쪽-경계(첫 덮음 이후 쪽들), tac=false 블록 표 경로(host_spacing 핀 밀집 지대) 미커버.
- compat 모드의 쪽 내 y 는 2022 저장 좌표 근사 — 쪽 소속만 2024 정합.
