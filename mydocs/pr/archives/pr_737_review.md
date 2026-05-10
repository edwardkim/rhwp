---
PR: #737
제목: Task #615 — pua_oldhangul U+F53A 매핑 제거 (hwpspec '매핑 표 외' 정합)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 6번째 PR)
base / head: devel / contrib/pua-oldhangul-fix
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS (Build & Test / Canvas visual diff / CodeQL / Analyze rust+js+py / WASM SKIPPED)
변경 규모: +24 / -4, 1 file (`src/renderer/pua_oldhangul.rs`)
검토일: 2026-05-10
---

# PR #737 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #737 |
| 제목 | Task #615 — pua_oldhangul U+F53A 매핑 제거 (hwpspec '매핑 표 외' 정합) |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735 영역 6번째 PR) |
| base / head | devel / contrib/pua-oldhangul-fix |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS (Canvas visual diff 포함) |
| 변경 규모 | +24 / -4, 1 file |
| 커밋 수 | 2 (Task 1 + Copilot 리뷰 반영 1) |
| closes | #615 |

## 2. PR supersede 체인 — 별 영역 영역 회귀 발견 패턴

### 2.1 직전 PR
- **PR #600** (closes #513, @oksure) — `fix: Supplementary PUA-A (U+F02B1~F02C4) SVG 출력 정정` (CLOSED)
- 시각 판정 중 발견 영역 영역 별개 영역 영역 결함 영역 영역 — 본 PR 영역 영역 영역 정정 영역 영역 별 PR 영역 영역 분리 영역 영역

### 2.2 본 PR (Task #615)
- 별 영역 영역 결함 영역 영역 본질 정정 — `pua_oldhangul.rs` 영역 영역 U+F53A 매핑 제거
- `feedback_pr_supersede_chain` 영역 영역 다른 패턴 (별 영역 영역 회귀 발견 → 별 PR 영역 영역 후속) — 동일 컨트리뷰터 영역 영역 후속 영역 영역 정합

## 3. 결함 본질 (Issue #615)

### 3.1 결함 영역
`samples/pua-test.hwp` 영역 영역 U+F53A 코드포인트 영역 영역 처리 영역 영역 한컴 정답지 영역 영역 정합 부재.

### 3.2 한컴 정답지 vs 본 환경

| 영역 | U+F53A 처리 |
|------|-------------|
| **한컴 PDF 정답지** | hwpspec "Basic-out, (매핑 표 외)" — 매핑 미지원, **빈 공백 표시** |
| 본 환경 SVG (정정 전) | `pua_oldhangul.rs:5022` 영역 영역 자모 시퀀스 `['\u{1112}', '\u{119E}', '\u{11AB}']` (ᄒᆞᆫ) 영역 임의 변환 → 옛한글 자모 글리프 미렌더 → 박스 표시 |

본 환경 영역 영역 임의 매핑 영역 영역 한컴 시각 영역 영역 정합 부재.

### 3.3 출처 영역
hwpspec 매핑 표 영역 영역 U+F53A = "Basic-out, hwpspec, (매핑 표 외)" — 명시적 영역 영역 매핑 미지원. KTUG HanyangPuaTableProject (자동 생성 영역 영역 의 원본 데이터) 영역 영역 hypua2jamo 영역 영역 의 의도 영역 영역 hwpspec 영역 영역 차이 — 한컴 영역 영역 의 spec 정합 우선.

## 4. PR 의 정정 — `src/renderer/pua_oldhangul.rs` (+24/-4)

### 4.1 U+F53A 매핑 제거 (line 5022)

```rust
// BEFORE
(0xF537, &['\u{1112}', '\u{119E}']),
(0xF538, &['\u{1112}', '\u{119E}', '\u{11A8}']),
(0xF539, &['\u{1112}', '\u{119E}', '\u{11C3}']),
(0xF53A, &['\u{1112}', '\u{119E}', '\u{11AB}']),  // ← 제거
(0xF53B, &['\u{1112}', '\u{119E}', '\u{11AE}']),

// AFTER
(0xF537, &['\u{1112}', '\u{119E}']),
(0xF538, &['\u{1112}', '\u{119E}', '\u{11A8}']),
(0xF539, &['\u{1112}', '\u{119E}', '\u{11C3}']),
// 0xF53A 제거: hwpspec 매핑 표 "Basic-out (매핑 표 외)" — 한컴 정답지와 정합 (#615)
(0xF53B, &['\u{1112}', '\u{119E}', '\u{11AE}']),
```

### 4.2 헤더 매핑 수 동기화 (Copilot 리뷰 반영, commit `937160c3`)
- `5,660 매핑` → `5,659 매핑` (모듈 doc comment + `static PUA_OLDHANGUL_MAP` 매핑 표 크기)
- `test_map_size` 검증값 5660 → 5659 동기화

### 4.3 자동 생성 안전성 코멘트
```rust
// 자동 생성 — scripts/gen_pua_oldhangul_rs.py 로 재생성
// 주의: hwpspec "매핑 표 외" 코드포인트(0xF53A 등)는 수동 제거됨 (#615).
// 재생성 시 test_hwpspec_unmapped_codepoints_not_in_table 테스트가 재삽입을 감지함.
```

→ 향후 자동 재생성 영역 영역 의 재삽입 영역 영역 회귀 가드 영역 영역 차단.

### 4.4 회귀 가드 신규 (테스트 1건)
```rust
#[test]
fn test_hwpspec_unmapped_codepoints_not_in_table() {
    let unmapped_basic_out: &[u32] = &[
        0xF53A, // Basic-out, hwpspec, (매핑 표 외)
    ];
    for &cp in unmapped_basic_out {
        if let Some(ch) = char::from_u32(cp) {
            assert!(
                map_pua_old_hangul(ch).is_none(),
                "hwpspec '매핑 표 외' U+{:04X} 가 pua_oldhangul 매핑 표에 존재 — 제거 필요",
                cp
            );
        }
    }
}
```

PR 본문 영역 영역 명시: BMP PUA 범위 영역 영역 U+F53A 영역 영역 유일 "매핑 표 외" 항목 (옵션 B 교차 검증).

## 5. 관련 매핑 영역 영역 점검

PR 본문 영역 영역 두 매핑 표 영역 영역 양립:
1. `src/parser/hwp3/johab_map.rs:4969` — `(0xF53A, '\u{617D}')` (HWP3 한자 영역 慽 슬퍼할 척)
2. `src/renderer/pua_oldhangul.rs:5022` — `(0xF53A, [ᄒ ᆞ ᆫ])` (옛한글 자모 임의 변환)

본 PR 영역 영역 영역 영역 (2) 만 정정 — HWP3 한자 영역 영역 (1) 영역 영역 별 영역 영역 (다른 포맷 / 다른 의미). 정정 범위 영역 영역 좁힘 (`feedback_hancom_compat_specific_over_general` 정합).

## 6. 본 환경 점검

- merge-base: `c9dd6f9c` (5/9 영역 영역 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 영역 영역 격리: `pua_oldhangul.rs` 단일 파일 — 다른 layout/render/parser 경로 영역 영역 무관
- 다른 PUA 매핑 영역 영역 영향 부재 (단일 코드포인트 제거)
- 시각 출력 영역 영역 영역 영향 영역 영역 — U+F53A 영역 영역 사용 영역 영역 fixture (`samples/pua-test.hwp`) 영역 영역 만 영향, 광범위 sweep 7 fixture 영역 영역 영향 점검 필요

## 7. 영향 범위

### 7.1 변경 영역
- pua_oldhangul 영역 영역 U+F53A 영역 영역 매핑 미지원 (한컴 정답지 정합)
- 자동 재생성 영역 영역 회귀 가드 (재삽입 차단)

### 7.2 무변경 영역
- 다른 PUA 매핑 5658개 영역 영역 보존
- HWP3 한자 영역 영역 (`johab_map.rs`) 영역 영역 무관 (별 영역)
- 다른 layout/render 경로 영역 영역 무영향

### 7.3 위험 영역
- **opt-in 정합** — 한컴 spec 영역 영역 정합 (매핑 미지원 → 빈 공백)
- 광범위 sweep 영역 영역 점검 필요 — 7 fixture 영역 영역 U+F53A 영역 영역 사용 가능성 영역 영역 점검

## 8. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 영역 영역 5/10 사이클 영역 영역 진전, 본 PR 영역 영역 단일 파일 영역 영역 충돌 부재

## 9. 처리 옵션

### 옵션 A — 2 commits cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task615 bbe4bb8b
git cherry-pick 93773d3c 937160c3
git checkout local/devel
git merge --no-ff local/task615
```

→ **옵션 A 추천**.

## 10. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release --lib pua` — 신규 1건 PASS + 기존 12건 + map_size 5659
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 점검 (U+F53A 영역 영역 의 사용 가능성)

### 시각 판정 게이트 — **작업지시자 권장**

본 PR 영역 영역 의 본질 영역 영역 **시각 정합** (U+F53A → 빈 공백, 한컴 정답지 정합):
- `samples/pua-test.hwp` 영역 영역 의 SVG 출력 영역 영역 빈 공백 정합 점검
- 한컴 PDF 정답지 영역 영역 의 정합 (이슈 #615 본문 명시)
- 광범위 sweep 7 fixture 영역 영역 영향 영역 영역 회귀 0 점검

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737 영역 6번째) |
| `feedback_pr_supersede_chain` | 별 패턴 — PR #600 (Supplementary PUA-A) 시각 판정 중 발견된 별 영역 영역 결함 영역 영역 별 PR 영역 영역 후속 (본 PR 영역 영역 의 supersede 영역 영역 본질 영역 영역 다른 영역 영역) |
| `feedback_image_renderer_paths_separate` | pua_oldhangul.rs 영역 영역 격리 — 다른 layout/render 경로 영역 영역 무영향 |
| `feedback_hancom_compat_specific_over_general` | U+F53A 영역 영역 단일 코드포인트 영역 영역 case 가드 — 일반화 매핑 영역 영역 영역 회귀 본질 영역 |
| `feedback_visual_judgment_authority` | 한컴 PDF 정답지 영역 영역 정합 영역 영역 권위 — 작업지시자 시각 판정 권장 |
| `feedback_process_must_follow` | 자동 생성 영역 영역 안전성 코멘트 + 회귀 가드 영역 영역 재삽입 차단 — 위험 좁힘 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + 광범위 sweep)
3. 작업지시자 시각 판정 (`samples/pua-test.hwp` 영역 영역 빈 공백 정합 + 광범위 sweep 점검)
4. 시각 판정 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #737 close (closes #615 자동 정합)

---

작성: 2026-05-10
