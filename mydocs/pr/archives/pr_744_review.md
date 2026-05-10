---
PR: #744
제목: Task #606 — HWPX ColorRef 상위 바이트(alpha) 보존 — 표 배경 검정 렌더링 수정
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 13번째 PR)
base / head: devel / contrib/hwpx-colorref-alpha
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS
변경 규모: +26 / -8, 3 files
검토일: 2026-05-10
PR_supersede: PR #607 (@dicebattle, 4/29 OPEN) supersede — (a) 패턴 정합
---

# PR #744 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #744 |
| 제목 | Task #606 — HWPX ColorRef 상위 바이트(alpha) 보존 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 13번째 PR) |
| base / head | devel / contrib/hwpx-colorref-alpha |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS |
| 변경 규모 | +26 / -8, 3 files |
| 커밋 수 | 2 (Task + Copilot 리뷰) |
| closes | #606 |

## 2. PR supersede 분석 — PR #607 본질 동일

### 2.1 직전 PR #607 (@dicebattle, 4/29 OPEN, 첫 PR)
- 제목: `fix: HWPX ColorRef 상위 바이트 보존`
- 변경 규모: +9 / -5, 1 file (`src/parser/hwpx/utils.rs`)
- 정정 영역: parser 만 (`parse_color_str` 영역 영역 alpha 보존)
- closes #606

### 2.2 본 PR #744 (@oksure, 5/9)
- 변경 규모: +26 / -8, 3 files
- 정정 영역: parser + serializer (양방향 정합)
  - `src/parser/hwpx/utils.rs` (+8/-5) — `parse_color_str` alpha 보존 (PR #607 동일)
  - `src/serializer/hwpx/header.rs` (+12/-2) — `color_hex` alpha 비제로 시 `#AARRGGBB` 출력 + `0xFFFFFFFF` 투명 센티넬 처리 (Copilot)
  - `src/serializer/hwpx/section.rs` (+6/-1) — `color_ref_to_hwpx` 동일 alpha 보존 출력

### 2.3 비교 표

| 항목 | PR #607 (@dicebattle) | PR #744 (@oksure) |
|------|----------------------|-------------------|
| 등록일 | 2026-04-29 | 2026-05-09 |
| 변경 규모 | +9/-5, 1 file | +26/-8, 3 files |
| Parser 정정 | ✅ 동일 | ✅ 동일 |
| Serializer 정정 | ❌ 부재 | ✅ 추가 (양방향 정합) |
| Copilot 리뷰 반영 | ❌ | ✅ (투명 센티넬 + 문서) |
| 검증 방법 | 샘플 SVG fill="#000000" count 비교 (123kb / 74kb) | 단위 테스트 + Copilot 리뷰 |

### 2.4 처리 결정 — `feedback_pr_supersede_chain` (a) 패턴

PR #744 가 PR #607 의 본질 (parser alpha 보존) 을 **완전 포함** + 추가로 serializer 양방향 정합. PR #607 close + 통합 머지 ((a) 패턴 정합 — PR #649 → #650 / PR #738 → #765 동일 패턴).

@dicebattle 영역 영역 영역 첫 PR (이력 1건) — PR #607 close 시 정중한 한국어 댓글 + supersede 명시 권장.

## 3. 결함 본질 (Issue #606)

### 3.1 결함 영역
HWPX 문서의 `faceColor="#FF000000"` (alpha=0xFF, 채우기 없음) 영역 영역 `0x00000000` (검정) 영역 영역 파싱 영역 영역 표 배경 검정색 렌더링 발생.

### 3.2 본질
`parse_color_str` 영역 영역 `#AARRGGBB` 8자리 입력 영역 영역 상위 alpha 바이트 영역 영역 버리고 `0x00BBGGRR` 영역 영역 변환. `style_resolver` 영역 영역 ColorRef 상위 바이트 비제로 영역 영역 "채우기 없음" 처리 로직 존재 영역 영역, 그러나 alpha 소실 영역 영역 분기 도달 부재.

## 4. PR 의 정정 — 3 files, +26/-8

### 4.1 `src/parser/hwpx/utils.rs` (+8/-5)

```rust
// AFTER
} else if hex.len() == 8 {
    // AARRGGBB → 0xAABBGGRR (alpha 보존)
    if let Ok(v) = u32::from_str_radix(hex, 16) {
        let a = (v >> 24) & 0xFF;
        let r = (v >> 16) & 0xFF;
        let g = (v >> 8) & 0xFF;
        let b = v & 0xFF;
        return a << 24 | b << 16 | g << 8 | r;
    }
}
```

기존 테스트 갱신 + 케이스 추가:
- `parse_color_str("#80FF0000")` → `0x800000FF` (alpha 보존)
- `parse_color_str("#FF000000")` → `0xFF000000` (상위 바이트 비제로 → 채우기 없음)
- `parse_color_str("#00FF0000")` → `0x000000FF` (alpha=00 → 동일)

### 4.2 `src/serializer/hwpx/header.rs` (+12/-2)

```rust
fn color_hex(c: ColorRef) -> String {
    if c == 0xFFFFFFFF {
        return "none".to_string();  // 투명 센티넬 (Copilot 리뷰)
    }
    let a = ((c >> 24) & 0xFF) as u8;
    let r = (c & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = ((c >> 16) & 0xFF) as u8;
    if a == 0 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", a, r, g, b)
    }
}
```

### 4.3 `src/serializer/hwpx/section.rs` (+6/-1)

`color_ref_to_hwpx` 동일 패턴 — alpha 비제로 시 `#AARRGGBB` 출력.

### 4.4 Copilot 리뷰 반영 (commit `3ff9c564`)
- `color_hex` 투명 센티넬 처리 (`0xFFFFFFFF` → "none")
- 문서 수정

## 5. 영향 범위

### 5.1 변경 영역
- HWPX 파싱: `#AARRGGBB` 영역 영역 alpha 보존
- HWPX 직렬화: alpha 비제로 시 `#AARRGGBB` 출력 (양방향 정합)
- 표 배경 `faceColor="#FF000000"` 영역 영역 채우기 없음 정합

### 5.2 무변경 영역
- `#RRGGBB` 6자리 입력 (alpha=0)
- 다른 layout/render 경로
- HWP3/HWP5 파서 영역 영역 무관

### 5.3 위험 영역
- HWPX→HWPX 라운드트립 정합 영역 영역 보존 (양방향 정정 핵심)
- 한국 정부 공식 양식 영역 영역 빈번히 사용되는 패턴

## 6. 본 환경 점검

- merge-base: `30351cdf` (5/9 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 격리: parser/utils.rs + serializer/header.rs + serializer/section.rs — 다른 layout/render 경로 무관

## 7. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 5/10 사이클 진전, 본 PR 격리 변경으로 충돌 부재

## 8. 처리 옵션

### 옵션 A — 2 commits cherry-pick + no-ff merge + PR #607 supersede close (추천)

```bash
git checkout -b local/task606 92802645
git cherry-pick 19d471d4 3ff9c564
git checkout local/devel
git merge --no-ff local/task606
# 머지 후 PR #607 close (supersede) + 정중 한국어 댓글
```

→ **옵션 A 추천**.

## 9. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` ALL GREEN (단위 테스트 신규 케이스 PASS)
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (HWPX 영역 영역 만 영향, HWP/HWP3 무관)

### 시각 판정 게이트 — **선택적**

본 PR 본질은 **HWPX 라운드트립 정합** + 표 배경 정합:
- 결정적 검증 (단위 테스트) 통과 → 시각 판정 면제 가능
- 또는 작업지시자 `123kb.hwp` / `74kb.hwp` 영역 영역 SVG fill="#000000" count 비교 (PR #607 검증 방법 활용)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 13번째 PR), @dicebattle 첫 PR (PR #607) |
| `feedback_pr_supersede_chain` | **(a) 패턴 적용** — PR #744 가 PR #607 supersede (close+통합 머지). PR #649 → #650 / PR #738 → #765 동일 패턴 정합 |
| `feedback_image_renderer_paths_separate` | parser/utils.rs + serializer/* 격리 — 다른 layout/render 경로 무영향 |
| `feedback_pr_comment_tone` | @dicebattle 첫 PR close 시 정중 한국어 댓글 + supersede 명시 권장 (과도한 표현 자제) |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 광범위 sweep)
3. 시각 판정 면제 합리 (결정적 검증 통과) — 또는 작업지시자 선택적 시각 판정
4. no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #744 close (closes #606 자동 정합)
6. **PR #607 close + 정중 한국어 댓글** (supersede 명시) — 작업지시자 결정 권장

---

작성: 2026-05-10
