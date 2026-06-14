# 최종 결과보고서 — Task #1363 v3: 미주 높이 모델 측정 SSOT (layout_partial_paragraph 측정 추출)

- **이슈**: #1363 (M100). v2(A2 휴리스틱 시뮬)가 두 배치를 해결했으나 타 문서 hancom 배치
  재튜닝 회귀 — 휴리스틱 높이 추정의 한계. **v3 은 렌더러 실제 높이를 직접 받아 정확화**.
- **브랜치**: `local/task1363` (base `stream/devel`).
- **성격**: 다세션 코어 추출 — 렌더-측정 일원화.

## 1. 결과 요약

| 항목 | 상태 |
|------|------|
| 측정 SSOT 인프라(`measure_endnote_column_bottom`) | **완성** ✓ |
| sim == render (전-단 순차 scratch 렌더) | **구조 보장** ✓ |
| 두 배치(p17 C×C 우단·p21 pi=1127) | **해결** ✓ |
| sep20/20 overflow (v2 A2 의 23.5px 회귀) | **→ 0** ✓ |
| `issue_1082`(5 exam overflow) A3 | **5/5 pass** ✓ |
| 부작용 격리 + 회귀 가드 테스트 | **완성** ✓ |
| 기본(B) 무회귀 | **유지**(A3 전용 opt-in) ✓ |
| 13건 hancom 배치 재현(issue_1139/1189/1209/1284) | **잔여**(별도 후속) ✗ |

**v3 의 핵심 인프라와 1차 목표(두 배치·overflow→0)는 완성.** 남은 13건은 계획서가 예고한
"전 exam 재튜닝" — 정확 sim 위에서 hancom 의 특이 배치를 재현하는 다회차 보정이다.

## 2. 단계별 경과

| 단계 | 내용 | 결과 |
|------|------|------|
| Stage 1 | 측정 호출 배선 — scratch `LayoutEngine::layout_partial_paragraph` 실측(A3 게이트) | per-para 측정 동작, divergence 정량화 |
| Stage 2 | 부작용 격리 실증 — 3계층 분리(self/render/scratch), 전역 가변 무 | 회귀 가드 테스트 1건, B 무회귀 |
| Stage 3 | split/fit 정합 시도 | **음성** — overflow 가 높이 추정에 비단조(cascade) 규명 |
| Stage 4 PartA | simulate↔render 정합 — `EN_RENDER` 계측 + 발산 카탈로그 | leading 컨트롤-줄 정합(a). per-para 고립 측정의 구조적 한계 규명 |
| Stage 5 | **옵션3 전환** — 전-단 1회 순차 scratch 렌더 | **sim==render·overflow→0·issue_1082 5/5**. blocker 2건 해소 |

## 3. 핵심 산출물 — 측정 SSOT 인프라

**`LayoutEngine::measure_endnote_column_bottom`** (`src/renderer/layout.rs`): 미주 단의 전 items 를
scratch 로 **1회 순차 `build_single_column` 렌더**해 정확한 단 bottom(px)을 반환. 렌더 코드
자체로 측정하므로 vpos forward-jump·trailing·`text_start_line` 등 dispatch 가 네이티브 처리 →
**sim==render 가 구조적으로 보장**된다. per-para 고립 측정의 컨텍스트 의존·순차 상호작용 발산을
회피.

**typeset A3 분기** (`simulate_endnote_column_bottom_y`): `current_items`(+candidate)를 로컬 +1
오프셋 재색인해 호출. 매 호출 새 scratch 엔진(상태 격리).

## 4. 기술적 발견 (재발 방지)

1. **overflow 는 sim 높이 추정에 비단조**: 큰 추정이 overflow 를 줄이지 않고 늘림(break cascade).
   정확 측정만으로 fit-parity 불가, 단일 문서 최적화 금지 — 전 exam **동시 green** 게이트.
   (메모리 `tech_endnote_overflow_nonmonotonic_gate`)
2. **per-para 고립 측정의 한계**: 같은 para 가 컨텍스트(단 폭)에 따라 측정 분기. 순차 forward-jump↔
   trailing 상호작용을 고립 측정으로 재현 불가 → 전-단 순차 렌더(옵션3)가 정답.
3. **scratch 렌더 blocker 2건**: ① `endnote_para_base=usize::MAX` → 미주 vpos 정규화 비활성
   (절대 vpos 누수) → `=0` 설정. ② 0-기반 재색인이 `para_index==0` column-top vpos fallback
   오발동(수식 35px→13721px 폭발) → 로컬 **+1 오프셋**(0 더미) 회피.
4. **정확 sim ≠ hancom 특이 배치**: 옵션3 의 정확한 fit 결정이 hancom 과 다른 단/쪽 경계를 만듦.
   튜닝 게이트가 hancom quirk 를 인코딩하고 있었음 → 13건 배치 회귀의 근본.
5. **export-svg CLI 아티팩트**: A3 에서 `export-svg` 가 render_page_svg_native(권위 경로)·
   `dump-pages`(23쪽)와 다른 페이지 수(158) → export-svg 경로의 A3 페이지네이션 재실행/상태
   차이 의심. 실제 렌더·테스트 무관하나 별도 추적 대상.

## 5. 검증

- **기본(B) 전체 cargo test 무회귀**: 측정 경로 전부 `ssot_level >= A3` 게이트 — 미설정(기본 B)
  시 종전과 100% 동일. Stage 1-2 시점 전체 suite 123/123 바이너리 ok 확인.
- **A3 권위 경로**: `issue_1082` 5/5(두 배치·overflow→0). `dump-pages` 23쪽 건전.
- **격리**: `test_measure_endnote_advance_side_effect_free` 유지.

## 6. 잔여 과제 (별도 후속 타스크)

1. **13건 hancom 배치 재보정** (주): `issue_1139`/`1189`/`1209`/`1284` 의 hancom-PDF 배치 테스트를
   정확 sim 위에서 재현. `split_endnote_to_fit`·`a2_overflow_with_para`·`available*k` 임계를
   full-render sim 단일 신뢰원으로 재보정. 비단조 cascade → 전 exam 동시 green 게이트, 다회차.
   완료 시 A3 기본 승격 + 레거시 경로 정리.
2. **export-svg CLI A3 페이지 폭발** 조사(별개).
3. **성능**: per-para 전-단 레이아웃 = O(n²) 측정. 미주 단위 캐싱/증분 측정(계획서 Stage 4 항목).

## 7. 커밋

`local/task1363`: Stage 1(`104fd8bd`)·Stage 2(`a469cb43`)·Stage 3(`a7c13e38`)·
Stage 4 PartA(`57ebe971`)·Stage 5(`adb875cd`)·Stage 5 정정(`7bcaec04`). A3 코드는 전부
`RHWP_EN_SSOT=A3` opt-in. 기본 B 무영향.
