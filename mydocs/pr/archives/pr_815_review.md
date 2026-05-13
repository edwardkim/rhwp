---
PR: #815
제목: test — PUA U+F02B1~F02BC CharOverlap 변환 검증 (Refs #727)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 14번째 PR
base / head: devel / contrib/pua-overlap-tests
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: 결과 부재
변경 규모: +37 / -0, 1 file
커밋: 2
검토일: 2026-05-11
---

# PR #815 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #815 |
| 제목 | test: PUA U+F02B1~F02BC CharOverlap 변환 검증 (#727) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 (5/11 사이클 **14번째 PR**) |
| base / head | devel / contrib/pua-overlap-tests |
| mergeable | CONFLICTING (DIRTY — 1 파일 충돌) |
| CI | 결과 부재 |
| 변경 규모 | +37 / -0, 1 file (테스트 신규만) |
| 커밋 수 | 2 (1 본질 + 1 리뷰 반영) |
| Refs | #727 (close 부재 — 조사 결과 + 회귀 가드 테스트만) |

## 2. 본질 (Issue #727 조사 결과)

Issue #727 영역 영역 `table-vpos-01.hwpx` p.5 nested 11×3 그리드 PUA U+F02B1~F02BC 영역 영역 한컴 권위 (사각형 안 숫자) vs 본 환경 출력 (원문자 ①~⑨ / 두부 캐릭터) 부정합 점검.

### 본 PR 결론 (PR 본문 명시)
`convert_pua_enclosed_numbers` 메커니즘은 **정상 동작 입증**:
- `U+F02B1~F02C4` → `CharOverlapInfo { border_type: 3 }` (사각형) 정확히 변환
- `pua_to_display_text` → "1"~"20" 정확히 반환
- SVG/Canvas 렌더러 `draw_char_overlap` → border_type=3 시 사각형 + 숫자 텍스트 정상 렌더링

→ 웹 뷰어 영역 영역 ① 출력 현상 영역 영역 **중첩 표 (11×3) 렌더링 경로** 영역 영역 별 결함 가능 (#726 영역 영역 별 이슈).

### PR 영역 영역 본질 정정 부재
- **조사 결과 보고 + 회귀 가드 테스트 추가만**
- Issue #727 영역 영역 close 부재 (별 본질 #726 영역 영역 중첩 표 SVG 렌더링 결함 점검 필요)

## 3. 정정 본질 — composer/tests.rs +37 (1 file)

### 3.1 `test_pua_enclosed_number_becomes_char_overlap`
```rust
let para = Paragraph { text: "\u{F02B1} 테스트", ... };
let composed = compose_paragraph(&para);
let overlap_run = runs.iter().find(|r| r.char_overlap.is_some());
assert!(overlap_run.is_some(), "U+F02B1 should produce a CharOverlap run");
assert_eq!(overlap.border_type, 3, "border_type should be 3 (rectangle)");
```

→ `compose_paragraph` 통과 시 U+F02B1 영역 영역 `CharOverlap(border_type=3)` 런 생성 검증.

### 3.2 `test_pua_to_display_text_range`
```rust
assert_eq!(pua_to_display_text('\u{F02B1}'), Some("1".to_string()));
assert_eq!(pua_to_display_text('\u{F02B9}'), Some("9".to_string()));
assert_eq!(pua_to_display_text('\u{F02BA}'), Some("10".to_string()));
assert_eq!(pua_to_display_text('\u{F02BB}'), Some("11".to_string()));
assert_eq!(pua_to_display_text('\u{F02BC}'), Some("12".to_string()));
assert_eq!(pua_to_display_text('\u{F02C4}'), Some("20".to_string()));
```

→ F02B1~F02BC 영역 영역 "1"~"12" + F02C4 영역 영역 "20" 변환 검증.

### 3.3 리뷰 반영 commit (`2a0a76f9`)
테스트 주석 PUA 범위 영역 영역 `F02B1~F02BC` → `F02B1~F02C4` 정정 (실제 테스트 영역 영역 F02C4 까지 영역 영역 정합).

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `compose_paragraph` (기존 composer) | 테스트 입력 |
| `Paragraph` / `LineSeg` / `CharShapeRef` (기존 model) | 테스트 데이터 |
| `pua_to_display_text` (기존 helper) | display text 검증 |

→ 신규 인프라 도입 부재 — 회귀 가드 단위 테스트만.

## 5. 영역 좁힘 (회귀 부재 가드)

- **테스트 신규만** — 기존 코드 변경 부재 → 회귀 위험 부재
- `composer/tests.rs` 파일 끝 영역 영역 신규 테스트 추가 → 다른 테스트 영향 부재
- Issue #727 영역 영역 본질 정정 영역 영역 별 PR / 별 본질 #726 영역 영역 후속

## 6. 본 환경 충돌 분석

### 6.1 1 파일 충돌
| 파일 | base | our (devel) | their (PR) |
|------|------|-------------|------------|
| `src/renderer/composer/tests.rs` | b4556f3e | db342eab | b403e504 |

devel 측 영역 영역 5/7 사이클 (Task #555/#528) 영역 영역 변경 누적. PR 측 영역 영역 파일 끝 영역 영역 테스트 신규 추가.

### 6.2 정합 전략
- PR 측 영역 영역 파일 끝 신규 테스트 추가 — devel 측 누적 변경 영역 영역 무관
- auto-merge 가능 영역 영역 cherry-pick 시 충돌 가능성 낮음 (그러나 git merge-tree 영역 영역 충돌 표시)
- 양측 모두 보존 (devel 측 누적 + PR 측 신규)

## 7. CI 결과 부재

mergeStateStatus = `DIRTY` 영역 CI 미실행. 충돌 해결 후 자기 검증 필수.

## 8. 처리 옵션

### 옵션 A (권장) — 2 commits cherry-pick + 1 파일 충돌 수동 해결 (필요 시) + no-ff merge

```bash
git checkout local/devel
git cherry-pick b50e8468 2a0a76f9
# composer/tests.rs 충돌 해결: 양측 보존 (devel 누적 + PR 신규)
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash cherry-pick + 충돌 수동 해결

본 환경 영역 영역 commit 이력 보존 권장 옵션 A.

## 9. 검증 게이트

### 9.1 자기 검증
- [ ] cherry-pick 2 commits + 1 파일 충돌 수동 해결 (필요 시)
- [ ] `cargo test --release` (신규 2 테스트 통과 + 기존 회귀 부재)
- [ ] tsc --noEmit (TypeScript 변경 부재 영역 영역 면제)
- [ ] WASM 재빌드 불필요 (테스트 신규만 영역 영역 WASM 영향 부재)

### 9.2 시각 판정 면제
- 테스트 신규만 영역 영역 시각 판정 불필요
- Issue #727 영역 영역 본질 정정 영역 영역 별 PR (Issue #727 영역 영역 OPEN 유지)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 14번째 PR) |
| `feedback_image_renderer_paths_separate` | 테스트 신규만 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (compose_paragraph + pua_to_display_text) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | 회귀 가드 테스트 영구 보존 — `feedback_close_issue_verify_merged` 권위 사례 강화 후보 |
| `feedback_diagnosis_layer_attribution` | `convert_pua_enclosed_numbers` 정상 동작 입증 + 중첩 표 SVG 렌더링 (#726) 영역 영역 별 본질 정확 분리 |
| `feedback_visual_judgment_authority` | Issue #727 영역 영역 작업지시자 시각 판정 영역 영역 발견 — 본 PR 영역 영역 조사 결과 + 회귀 가드 (본질 정정 부재) |
| `feedback_pr_supersede_chain` | PR #694 (Task 영역 영역 중첩 표 + nested 11×3 그리드) → Issue #727 (PUA 매핑 점검) → **PR #815** (조사 결과 + 회귀 가드 테스트) + 별 본질 (#726, 중첩 표 SVG 렌더링) |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick 2 commits + 1 파일 충돌 수동 해결 (필요 시)
2. 자기 검증 (cargo test + 신규 테스트 2 통과)
3. WASM 재빌드 불필요 (테스트 신규만)
4. no-ff merge + push + archives + 5/11 orders
5. Issue #727 영역 영역 OPEN 유지 (별 본질 #726 후속)
6. PR #815 close

---

작성: 2026-05-11
