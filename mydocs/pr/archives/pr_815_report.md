---
PR: #815
제목: test — PUA U+F02B1~F02BC CharOverlap 변환 검증 (Refs #727)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 14번째 PR
처리: 옵션 C — 본질 점검 잘못 + 신규 테스트 cargo test 실패 영역 영역 close + 분리 PR 재제출 요청
처리일: 2026-05-11
---

# PR #815 처리 보고서

## 1. 처리 결과

✅ **close 완료** — 옵션 C (본질 진단 잘못 + cargo test 실패 영역 영역 close + 분리 PR 가이드)

| 항목 | 값 |
|------|-----|
| 처리 | close + 컨트리뷰터 본질 안내 + 분리 PR 2 가이드 |
| 사유 | PR 본문 가정 영역 영역 실제 코드 명세 영역 영역 부정합 + cargo test 2/2 FAILED |
| Issue #727 | OPEN 유지 (본질 정정 영역 영역 분리 PR 1 영역 영역 후속) |

## 2. 본 PR 본질

Issue #727 영역 영역 PUA U+F02B1~F02BC (사각형 안 숫자 1~12) 영역 영역 조사 결과 + 회귀 가드 단위 테스트 추가.

### PR 본문 결론
> `convert_pua_enclosed_numbers` 메커니즘 정상 동작:
> - U+F02B1~F02C4 → CharOverlapInfo { border_type: 3 } (사각형) 정확히 변환
> - pua_to_display_text → "1"~"20" 정확히 반환

### 추가 테스트 (PR 영역 영역)
1. `test_pua_enclosed_number_becomes_char_overlap` — U+F02B1 영역 영역 CharOverlap(border_type=3) 검증
2. `test_pua_to_display_text_range` — F02B1~F02C4 → "1"~"20" 변환 검증

## 3. ⚠️ 본 환경 cherry-pick + cargo test 결과 — 2/2 FAILED

본 환경 영역 영역 2 commits cherry-pick + 충돌 수동 해결 (양측 보존 — Task #555 옛한글 PUA + PR #815 사각형 안 숫자) 후:

```
test renderer::composer::tests::test_pua_enclosed_number_becomes_char_overlap ... FAILED
  panicked: U+F02B1 should produce a CharOverlap run

test renderer::composer::tests::test_pua_to_display_text_range ... FAILED
  panicked: assertion `left == right` failed
    left: None, right: Some("1")
```

## 4. 본질 진단 — PR 영역 영역 잘못된 가정

### 실제 코드 명세 (`src/renderer/composer.rs`)

라인 1122/1171/1184 영역 영역 명확한 주석:
```rust
// - U+F02B1~U+F02C4: map_pua_bullet_char 에서 ①~⑳ 으로 매핑 (CharOverlap 제외)
// - U+F02CE~U+F02E1: 반전 사각형 안의 숫자 1~20 (border_type=4)

pub fn pua_to_display_text(ch: char) -> Option<String> {
    let cp = ch as u32;
    // U+F02B1~F02C4 는 map_pua_bullet_char 에서 ①~⑳ 으로 매핑 — 여기 도달 불가
    if (0xF02CE..=0xF02E1).contains(&cp) {
        let num = cp - 0xF02CD;
        return Some(format!("{}", num));
    }
    None
}
```

### 매핑 영역 비교

| 코드포인트 | 실제 매핑 | PR 가정 |
|-----------|----------|---------|
| **U+F02B1~F02C4** | `map_pua_bullet_char` → ①~⑳ (CharOverlap **제외**) | ❌ CharOverlap border_type=3 |
| **U+F02CE~F02E1** | `convert_pua_enclosed_numbers` → CharOverlap border_type=**4** | (PR 미언급) |

PR 영역 영역:
- 가정한 코드포인트 (U+F02B1~F02C4) 영역 영역 CharOverlap 영역 영역 매핑되지 않음
- border_type 영역 영역 4 (반전 사각형) — PR 본문 "border_type=3" 잘못
- 실제 CharOverlap 범위 (U+F02CE~F02E1) 영역 영역 미언급

## 5. Issue #727 영역 영역 재진단

PR 본문 결론 ("정상 동작 입증") 영역 영역 본 환경 cargo test 영역 영역 반박됨.

### 정확한 본질 (재진단)

**한컴 권위** (`pdf/table-vpos-01-2022.pdf`): table-vpos-01.hwpx p.5 영역 영역 PUA U+F02B1~F02BC 영역 영역 **사각형 안 숫자 1~12**

**본 환경 출력**: U+F02B1 → `map_pua_bullet_char` 영역 영역 ① (원문자) — PR #600 (Task #509) 매핑 영역 영역 도입

→ Issue #727 영역 영역 진단 대상 영역 영역 `convert_pua_enclosed_numbers` 정상 동작 여부 아닌, **`map_pua_bullet_char` 영역 영역 U+F02B1~F02C4 → ①~⑳ 매핑 자체** 영역 영역 한컴 권위 영역 영역 부정합 본질.

### 두 가능성
| 가능성 | 본질 | 해결 방향 |
|--------|------|----------|
| (가) **글로벌 매핑이 잘못** | 모든 fixture 영역 영역 사각형 안 숫자 영역 영역 일관 | `map_pua_bullet_char` 영역 영역 매핑 영역 영역 정정 (mel-001 / kps-ai 재검증) |
| (나) **fixture 별 다른 본질** | context 영역 영역 따른 분기 필요 | context 분기 가드 |

## 6. 컨트리뷰터 안내 (정중 톤)

본 환경 영역 영역 댓글 [#815#issuecomment-4421677741](https://github.com/edwardkim/rhwp/pull/815#issuecomment-4421677741):
- 본 환경 cargo test 2/2 FAILED 결과 명시
- 실제 코드 명세 vs PR 가정 영역 영역 매핑 영역 비교
- Issue #727 영역 영역 정확한 본질 재진단
- 분리 PR 2 가이드:
  - **분리 PR 1** — Issue #727 본질 정정 (`map_pua_bullet_char` 영역 영역 매핑 점검)
  - **분리 PR 2** — 회귀 가드 테스트 (실제 매핑 정합 영역 영역 정정 후 재제출)

## 7. 본 환경 reset

cherry-pick 후 cargo test 실패 → `git reset --hard origin/devel` 영역 영역 PR commits 제거 → devel 영역 영역 영향 부재.

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 14번째 PR) |
| `feedback_image_renderer_paths_separate` | 테스트 신규만 영역 영역 Rust 렌더링 경로 무영향 (그러나 잘못된 코드포인트 영역 영역 검증 영역 영역 의미 부재) |
| `feedback_process_must_follow` 권위 사례 강화 | **cargo test 자기 검증 필수** 영역 영역 본질 진단 영역 영역 잘못된 점 발견 — CI 결과 부재 (DIRTY) 영역 영역 본 환경 자기 검증 표준 영역 영역 정합 |
| `feedback_hancom_compat_specific_over_general` | Issue #727 영역 영역 본질 — `map_pua_bullet_char` 영역 영역 글로벌 매핑 (PR #600) vs fixture 별 한컴 권위 영역 영역 점검 영역 영역 본질적 |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | **두 본질 분리 진단** — (1) `map_pua_bullet_char` (U+F02B1~F02C4 → 원문자) vs (2) `convert_pua_enclosed_numbers` (U+F02CE~F02E1 → CharOverlap border_type=4). PR 영역 영역 두 본질 영역 영역 혼동 → 잘못된 가정 |
| `feedback_visual_judgment_authority` | Issue #727 영역 영역 작업지시자 시각 판정 영역 영역 발견 + 본 환경 cargo test 영역 영역 결정적 검증 영역 영역 PR 가정 영역 영역 반박 |
| `feedback_pr_supersede_chain` | PR #600 (Task #509, ①~⑳ 매핑 도입) → Issue #727 (한컴 권위 영역 영역 부정합 발견) → **PR #815** (close, 잘못된 가정 + cargo test FAILED) → 분리 PR 1 (본질 정정) + 분리 PR 2 (회귀 가드, 재제출) 영역 영역 후속 |

## 9. 잔존 후속

- 컨트리뷰터 영역 영역 분리 PR 1 (Issue #727 본질 정정) + 분리 PR 2 (회귀 가드 정정 재제출) 영역 영역 대기
- Issue #727 OPEN 유지 — 분리 PR 1 영역 영역 close
- 본 환경 reset 영역 영역 devel 무영향

---

작성: 2026-05-11
