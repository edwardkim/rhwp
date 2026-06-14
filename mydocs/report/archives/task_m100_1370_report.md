# 최종 결과보고서 — Task #1370: 미주 측정 SSOT 게이트 재보정 (정확 sim 위 hancom 배치)

- **이슈**: #1370 (M100), 부모 #1363 v3 후속
- **브랜치**: `local/task1370` (base `local/task1363` = A3 인프라 커밋 `f20199ae`)
- **성격**: 미주 break-높이 게이트 재보정 (레버2 — 게이트 한정 compact override)

## 1. 결과 요약

| 항목 | 상태 |
|------|------|
| A3 회귀 13건 → **7건** (6 해결) | ✓ |
| `issue_1082` overflow A3 5/5 | **유지** ✓ |
| 전체 A3 cargo test 무회귀(잔여 7건 외) | ✓ |
| 기본(B) 무회귀 (72/72·5/5) | ✓ |
| 잔여 7건(2023_sep cascade) | **후속 #1373 분리** |

**6/13 을 무회귀로 해결**. 잔여 7건은 비단조 cascade 로 다회차 탐색이 필요해 후속 타스크(#1373)로 분리
(작업지시자 승인).

## 2. 단계별 경과

| 단계 | 내용 | 결과 |
|------|------|------|
| Stage 1 | A3 회귀 13건 진단 카탈로그(코드 무수정) | 그룹·책임 게이트 1차 매핑 |
| Stage 2 | 정밀진단 → **A2sim 스냅 A3 OFF**(break-높이 compact 환원) | **13→7**, 1082 5/5·B 무회귀 |
| Stage 3 | 2023_sep 잔여 cascade 진단, 레버 3건 시도 | 근본 규명, 3건 음성(비단조) → 후속 분리 |

## 3. 핵심 산출물 — break-높이 디커플 (레버2)

**근본**: A3 는 매 미주 para 누적 후 `st.current_height` 를 **정확 전-단 렌더 sim**으로 스냅
(typeset.rs:3637)했는데, 이 exact 스냅이 rewind/빈 para 를 hancom 보다 **단당 ~80px 높게** 누적 →
경계 para split 차단 → 하류 cascade.

**해결**: A2sim 스냅을 `ssot_level >= A2` → `== A2` 로 게이트(`1aa67e02`). A3 는 break-결정 높이를
compact(acc)로 환원하되, **`a2_overflow_with_para` 는 정확 sim 직접 호출 유지** → overflow 안전선=exact,
단 fill=compact 로 분리(레버2 정합).

**해결 6건**: page17_q30·split_titles·question29_full_para·internal_rewind_formula_tail·
page21_q23_left_tail·page22_23_tail (2022_sep/2022_nov/2024_between20).

## 4. 기술적 발견 (재발 방지)

1. **양방향 과대추정**: 미주 break-높이의 hancom 발산은 단방향이 아니다.
   - 2022: **exact-snap 이 과대** → over-fill (제목 침하·overflow). 스냅 OFF 로 해결.
   - 2023: **saved-delta en_fit 이 과대** (forward-jump 빈 para = 115px vs 정확 18px) → 조기 advance
     (단 under-fill). 미해결.
   - **hancom ≈ min(saved-delta, 정확 sim)** 이나, 적용 조건이 비단조라 단일 레버로 양립 불가.
   (메모리 `tech_endnote_overflow_nonmonotonic_gate` 갱신)
2. **`advance_for_fit`(3381) 전역 정확-sim 교체는 비단조 회귀**: pi=799 조기 advance 는 막지만 타 단의
   필요한 advance 억제 → 16건(전역)/8건(빈 para 한정) 회귀. para 별 높이가 전 게이트에 정밀 결합.
3. **검증 경로**: A3 `export-svg` CLI 158쪽 폭발은 별개 아티팩트. `dump-pages`(CLI)·`build_page_render_tree`·
   `dump_page_items`(통합테스트)·`RHWP_EN_SSOT_DEBUG`(EN_SSOT/EN_ACC/EN_COLSIM)만 권위.

## 5. 검증

- A3 전체 cargo test: 총 실패 7건 = `issue_1139` 잔여뿐(다른 파일 무회귀).
- A3 `issue_1082` 5/5, B `issue_1139` 72/72·`issue_1082` 5/5.
- 음성 레버 3건은 전부 revert → 코드는 Stage 2 커밋 상태.

## 6. 잔여 과제 → 후속 #1373

2023_sep 조기 advance cascade 7건(2023_sep ×6 + 2024_between20 ×1). forward-jump 빈 para 의 break-높이를
`min(saved-delta, 정확 sim)` 으로 보는 다조건 게이트를 전 exam 동시 green 으로 탐색. 본 타스크 진단이 시드.

(별개 미해결: export-svg CLI A3 158쪽 폭발, per-para 전-단 레이아웃 O(n²) 성능 — #1370 이슈 본문 명시.)

## 7. 커밋

`local/task1370`: Stage 1(`ee0bba19`)·Stage 2(`1aa67e02`, 소스)·Stage 3(`87337cc6`, 진단). A3 코드는
전부 `RHWP_EN_SSOT` opt-in, 기본 B 무영향.
