---
PR: #966
제목: fix — WMF SetTextAlign vertical bits 파싱 정정 + baseline y shift 정합 (WMF 박스 외부 텍스트 표시 해소, closes #965, ports PR #918 Stage 33-A)
컨트리뷰터: @jangster77 (Taesup Jang) — 24+ 사이클 핵심 컨트리뷰터 (연속 5 PR #956~#964 완결 후 추가 2개 중 1번째)
base / head: devel / local/task965
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: ✅ Build & Test + CodeQL (js-ts/python/rust)
변경 규모: +655 / -20, 9 files (코드 1 / 문서 8)
커밋: 5 (본질 1 + devel merge 4)
검토일: 2026-05-18
---

# PR #966 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #966 |
| 제목 | fix: WMF SetTextAlign vertical bits 파싱 정정 + baseline y shift 정합 (ports PR #918 Stage 33-A) |
| 컨트리뷰터 | @jangster77 — **24+ 사이클** (연속 5 PR #956~#964 완결 후 추가 2개 중 **#966 1번째**, #968 후속) |
| base / head | devel / local/task965 |
| mergeable | MERGEABLE (BEHIND — base 갱신만) |
| CI | ✅ Build & Test + CodeQL (js-ts/python/rust) |
| 변경 규모 | +655 / -20, 9 files (코드 1 / 문서 8) |
| 커밋 수 | 5 (본질 `88291fdb` + devel merge 4) |
| closes | #965 |
| 관련 | **PR #918 (CLOSED, +5082/-74, 5/16) Stage 33-A 핵심만 단독 포팅** |

## 2. 본질 (Issue #965)

`samples/hwp3-sample16.hwp` page 18 WMF 다이어그램 (주전산센터 목표시스템 구성안)
내부 박스 한글 텍스트 ("PE6450", "기록서버", "Windows 서버군") 가 박스 외부로 벗어남.

### Root cause (`src/wmf/converter/svg/mod.rs:2195` set_text_align)
```rust
// 이전 (버그)
let align_vertical = [VTA_BOTTOM, VTA_TOP /* =0x0000 */]
    .into_iter()
    .find(|a| record.text_alignment_mode & (*a as u16) == *a as u16)  // mode & 0 == 0 항상 true
    .unwrap_or(VTA_BASELINE);
```

`mode & VTA_TOP(=0x0000) == 0x0000` 영역 영역 **항상 true** → BASELINE/BOTTOM mode 도
VTA_TOP 으로 오매핑 → `ext_text_out` 영역 영역 +ascent shift → baseline cell-top 보정만큼
아래로 → 박스 하단 라인 걸침.

WMF [MS-WMF] 2.1.2.18: TA_TOP=0x0000 / TA_BOTTOM=0x0008 / TA_BASELINE=0x0018.

## 3. 정정 본질 — `src/wmf/converter/svg/mod.rs` 3 영역 (~60 lines)

### 3.1 `set_text_align` (:2195) — root cause
```rust
let v_bits = record.text_alignment_mode & 0x0018;
// 우선순위 BASELINE(0x0018) → BOTTOM(0x0008) → TOP(0x0000)
```
vertical bits (0x0018 mask) 값 기준 분기 — 항상-true 버그 제거.

### 3.2 `ext_text_out` (:811) — baseline y shift 정합
- VTA_BASELINE: 0 (y 가 baseline — 그대로)
- VTA_TOP: +ascent (y 가 cell top)
- VTA_BOTTOM: -em*0.2 (y 가 cell bottom)
- 이전 `font.height < 0` 잘못된 보정 제거 (텍스트 박스 하단 shift 원인)

### 3.3 `text_out` (:1545) — META_TEXTOUT 동일 보정
PR #918 **미포함** — 본 PR 추가 (ext_text_out 과 일관성).

## 4. PR #918 supersede 분석 (핵심)

PR #918 (CLOSED, 2026-05-16, **+5082/-74 거대 PR**) close 사유 — 다양한 부작용:
LibreOffice emfio 포팅 + WASM RasterPlayer + nested SVG inline embed +
woff2 base64 임베드 제거 + DX byte-aware indexing + POLYPOLYGON fill-rule.

본 PR 영역 영역 **Stage 33-A root cause fix (~60 lines) 만 단독 포팅**:

| PR #918 (closed, 5082+) | 본 PR (~60) |
|------------------------|------------|
| LibreOffice emfio 포팅 | ❌ 제외 |
| WASM RasterPlayer | ❌ 제외 |
| nested SVG inline embed | ❌ 제외 |
| Stage 33-A set_text_align + ext_text_out | ✅ 포팅 |
| text_out baseline (PR #918 미포함) | ✅ 본 PR 추가 |

→ `feedback_pr_supersede_chain` (a) 패턴 — close 거대 PR → root cause 만 작은 PR
재제출. `feedback_small_batch_release_strategy` 정합 (5082 → 60 lines 분리).

## 5. ⚠️ 본 환경 충돌 분석

| 파일 | 충돌 | 본질 |
|------|------|------|
| `mydocs/orders/20260517.md` | changed in both | 본 환경 PR 처리 표 + PR #966 Task #965 작업 일지 — 양측 보존 |
| `src/wmf/converter/svg/mod.rs` | **changed in both** | devel 영역 영역 PR #860/#864 (5/16, `735d3057`) 영역 영역 `ext_text_out`(:790)/`text_out`(:1528) vertical alignment 분기 이미 존재. PR #966 영역 영역 동일 함수 정정 — **충돌 면밀 점검 필수** |
| `task_m100_965*` 8 | added in remote | 신규 추가 (충돌 없음) |

### svg/mod.rs 충돌 정합 전략
- devel HEAD (PR #860/#864): `ext_text_out`/`text_out` 영역 영역 vertical alignment 분기 + `set_text_align` 영역 영역 항상-true 버그 잔존 가능
- PR #966: 3 영역 root cause fix
- → cherry-pick 충돌 시 PR #966 정정 영역 영역 채택 (root cause fix), devel 측 PR #860/#864 의 다른 변경 (EMF/WMF image 렌더) 보존 — **수동 정밀 해결**

## 6. 본 환경 점검

### 6.1 변경 격리
- WMF converter (`svg/mod.rs`) 단일 — 비-WMF 영향 없음
- WASM 환경 정합 개선 예상 (Canvas2D 가 동일 SVG 사용)

### 6.2 CI 통과
- ✅ Build & Test + CodeQL (js-ts/python/rust)
- Canvas visual diff 항목 미표시 (점검 필요)

### 6.3 검증 (PR 본문)
- cargo test --release --lib: 1288 passed, 0 failed
- sample16 page 18 WMF 박스 내부 한글 텍스트 정상 위치 ✓ 한컴 viewer 정합
- WMF sample (sample14 page 0~8, sample4 page 1) PNG diff <1% (정상화 방향, 회귀 없음)

## 7. 영역 좁힘 (PR 본문 명시)

| 영역 | 영향 |
|------|------|
| WMF BASELINE/BOTTOM 모드 텍스트 | 정상 위치 (회귀 fix) |
| WMF TOP 모드 텍스트 | 기존 동작 (영향 없음) |
| 비-WMF | 영향 없음 (svg/mod.rs WMF converter 만 변경) |
| WASM 환경 | 정합 개선 예상 |

## 8. 처리 옵션

### 옵션 A (권장) — 본질 commit cherry-pick + 충돌 수동 해결 + 자기 검증 + WASM 재빌드

```bash
git checkout local/devel
git cherry-pick 88291fdb   # 본질만 (devel merge commit 4개 제외)
# 충돌 수동 해결:
#   - svg/mod.rs: PR #966 root cause fix 채택 + devel PR #860/#864 변경 보존 (정밀)
#   - orders/20260517.md: --ours + Task #965 작업 일지 갱신
# cargo test + 광범위 sweep + WMF PNG 시각 점검 (sample16 p18)
# WASM 재빌드
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash 5 commits (devel merge 포함, 비권장)

## 9. 검증 게이트

### 9.1 자기 검증
- [ ] cherry-pick `88291fdb` (본질만) + svg/mod.rs + orders 충돌 수동 해결
- [ ] devel PR #860/#864 (EMF/WMF image 렌더) 변경 보존 확인
- [ ] cargo test --release --lib ALL GREEN (PR 본문 1288 passed)
- [ ] cargo clippy --release -- -D warnings
- [ ] **광범위 sweep 7 fixture / 169 페이지** — WMF 변경 영역 영역 회귀 점검
- [ ] WMF sample (sample14/sample4) PNG diff 점검
- [ ] WASM 재빌드 (svg/mod.rs 변경)

### 9.2 시각 판정 게이트 — **작업지시자 시각 검증 권장**
- sample16 (HWP3) page 18 WMF 다이어그램 — 박스 내부 한글 텍스트 ("PE6450", "기록서버", "Windows 서버군") 정상 위치 (한컴 viewer 정합, 박스 외부 벗어남 해소)
- WMF sample (hwp3-sample14 page 0~8, sample4 page 1) 회귀 부재
- devel PR #860/#864 EMF/WMF image 렌더 회귀 부재 (충돌 정밀 해결 검증)
- 비-WMF sample 회귀 부재 (sweep)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 **24+ 사이클** (연속 5 PR 완결 후 추가 #966/#968) |
| `feedback_image_renderer_paths_separate` | WMF converter svg/mod.rs 단일 — Canvas2D 동일 SVG 사용 (정합 개선) |
| `feedback_hancom_compat_specific_over_general` | WMF [MS-WMF] 2.1.2.18 spec 정합 (v_bits 0x0018 mask) — 추측 아닌 spec 기반 |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | `mode & VTA_TOP(=0) == 0` 항상-true 버그 root cause 정확 진단 |
| `feedback_pr_supersede_chain` 권위 사례 강화 | **PR #918 (CLOSED, 5082+ 거대 PR, 다양한 부작용) → #966 (root cause ~60 lines 단독 포팅)** — (a) 패턴 + 거대 PR 분리 |
| `feedback_small_batch_release_strategy` 권위 사례 강화 | 5082 lines 거대 PR → 60 lines root cause 분리 — 작은 단위 회전 입증 |
| `reference_authoritative_hancom` | sample16 page 18 WMF 박스 한컴 viewer 정합 기준 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick `88291fdb` (본질만, devel merge 제외) + svg/mod.rs + orders 충돌 수동 해결
2. devel PR #860/#864 EMF/WMF image 렌더 변경 보존 확인
3. 자기 검증 — cargo test + clippy + 광범위 sweep + WMF PNG diff + WASM 재빌드
4. 작업지시자 시각 검증 (sample16 p18 WMF 박스 한글 텍스트 + sample14/4 + EMF/WMF 회귀 부재)
5. 검증 통과 → no-ff merge + push + archives + 5/17 orders
6. Issue #965 close + PR #966 close + 추가 PR #968 진행

---

작성: 2026-05-18
