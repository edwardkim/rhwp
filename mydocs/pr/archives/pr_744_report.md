---
PR: #744
제목: Task #606 — HWPX ColorRef 상위 바이트(alpha) 보존
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 13번째 PR)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge + PR #607 supersede
처리일: 2026-05-10
머지 commit: a4b368a5
PR_supersede: PR #607 (@dicebattle 4/29) supersede — (a) 패턴
---

# PR #744 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge `a4b368a5`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `a4b368a5` (--no-ff merge) |
| Cherry-pick commits | `4b82f669` + `2e5e993f` (Copilot 리뷰) |
| closes | #606 |
| supersede | PR #607 (@dicebattle, 4/29 OPEN) — close 후속 처리 |
| 자기 검증 | cargo test ALL GREEN + utils tests 3 PASS + sweep 170/170 same |

## 2. PR supersede (a) 패턴

| 항목 | PR #607 (@dicebattle, 4/29) | PR #744 (@oksure, 5/9) |
|------|----------------------------|------------------------|
| 변경 규모 | +9/-5, 1 file | +26/-8, 3 files |
| Parser 정정 | ✅ 동일 | ✅ 동일 |
| Serializer 정정 | ❌ 부재 | ✅ 추가 (양방향 정합) |
| Copilot 리뷰 | ❌ | ✅ |

→ PR #744 가 PR #607 본질 완전 포함 + serializer 양방향 정합 추가. `feedback_pr_supersede_chain` (a) 패턴 — close+통합 머지.

## 3. 정정 본질 — 3 files, +26/-8

### 3.1 `src/parser/hwpx/utils.rs` (+8/-5)

```rust
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

### 3.2 `src/serializer/hwpx/header.rs` (+12/-2)

```rust
fn color_hex(c: ColorRef) -> String {
    if c == 0xFFFFFFFF {
        return "none".to_string();  // 투명 센티넬 (Copilot 리뷰)
    }
    let a = ((c >> 24) & 0xFF) as u8;
    // ... alpha 비제로 시 #AARRGGBB 출력
}
```

### 3.3 `src/serializer/hwpx/section.rs` (+6/-1)
`color_ref_to_hwpx` 동일 패턴.

### 3.4 Copilot 리뷰 반영 (commit `2e5e993f`)
- `color_hex` 투명 센티넬 처리 (`0xFFFFFFFF` → "none")
- 문서 수정

## 4. 결함 본질 (Issue #606)

HWPX 문서의 `faceColor="#FF000000"` (alpha=0xFF, 채우기 없음) 영역 영역 `0x00000000` (검정) 으로 파싱 → 표 배경 검정색 렌더링. `style_resolver` 영역 영역 ColorRef 상위 바이트 비제로 → "채우기 없음" 분기 존재 영역 영역, alpha 소실 영역 영역 분기 도달 부재.

## 5. 본 환경 cherry-pick + 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ 충돌 0건 |
| `cargo build --release` | ✅ 통과 (31.65s) |
| `cargo test --lib parser::hwpx::utils` | ✅ **3 PASS** (`test_parse_color_str_with_alpha` 신규 케이스 포함) |
| `cargo test --release` (전체) | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (HWPX 만 영향, HWP 변환본 sweep 무영향) |

## 6. 영향 범위

### 6.1 변경 영역
- HWPX 파싱: `#AARRGGBB` 영역 영역 alpha 보존
- HWPX 직렬화: alpha 비제로 시 `#AARRGGBB` 출력 (양방향 정합)
- 표 배경 `faceColor="#FF000000"` 영역 영역 채우기 없음 정합

### 6.2 무변경 영역 (sweep 170/170 same 입증)
- `#RRGGBB` 6자리 입력 (alpha=0)
- 다른 layout/render 경로
- HWP3/HWP5 파서

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 영역 영역 13번째 PR), @dicebattle 첫 PR (PR #607) |
| `feedback_pr_supersede_chain` | **(a) 패턴 적용** — PR #744 가 PR #607 supersede (close+통합 머지). PR #649 → #650 / PR #738 → #765 동일 패턴 |
| `feedback_image_renderer_paths_separate` | parser/utils.rs + serializer/* 격리 — 다른 layout/render 경로 무영향 (sweep 170/170 same 입증) |
| `feedback_pr_comment_tone` | @dicebattle 첫 PR close 시 정중 한국어 댓글 + supersede 명시 |

## 8. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- PR #607 supersede close (작업지시자 결정 후 진행)

---

작성: 2026-05-10
