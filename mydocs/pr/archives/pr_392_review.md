# PR #392 검토 — 다단 섹션 누적 공식 회귀 정정 (#359 후속)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#392](https://github.com/edwardkim/rhwp/pull/392) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| base / head | `devel` ← `local/task391` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | **BEHIND** |
| 이슈 | [#391](https://github.com/edwardkim/rhwp/issues/391) |
| 변경 통계 | +1142 / -2, 11 files |
| CI | 모두 SUCCESS |

## 결함 요약 (작업지시자 정황)

**v0.7.7 의 Task #359 정정** 으로 다단 섹션에 회귀 유발:
- `exam_eng.hwp` (2단 다단 섹션): **8p → 11p 회귀**
- 단 (column) 채움 비대칭 + 단독 1-item 단 다수 발생 (p3 단 1, p5 단 0, p7 단 1 등)

작업지시자 메모리 정책 (`feedback_v076_regression_origin.md`) 의 표본:
- v0.7.6 머지 후 메인테이너가 v0.7.7 에서 정정한 회귀 (Task #359, #361, #362) 가 **다른 영역에서 다시 회귀를 유발**
- 본 PR 은 그 회귀 (다단 섹션) 정정

## 변경 내용

### `src/renderer/typeset.rs` — 누적 공식 분기 (3 줄, 2 곳)

```rust
st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
- st.current_height += fmt.total_height;
+ // [Task #391] 다단/단단 분기:
+ //   - 단단 (col_count == 1): total_height (k-water-rfp p3 311px drift 차단, #359)
+ //   - 다단 (col_count > 1): height_for_fit (exam_eng 8p 정상 단 채움 복원)
+ st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
return;
```

라인 805 (`fits → place 전체배치`) + 815 (`line_count == 0` 폴백) 두 곳 동일 변경.

### `tests/exam_eng_multicolumn.rs` (신규) — 회귀 테스트

```rust
#[test]
fn exam_eng_page_count_after_359_fix() {
    let doc = HwpDocument::from_bytes(&bytes).expect(...);
    assert_eq!(doc.page_count(), 8, "exam_eng.hwp 8 페이지 기대 (Task #391 / #359 회귀 복원)");
}
```

### 문서

- 수행/구현계획서, stage1-3 보고서, 최종 보고서, baseline 측정값
- CHANGELOG 업데이트
- orders 갱신

## 분석 — 일반화 vs 명시 가드 (메모리 원칙 점검)

작업지시자 메모리 (`feedback_hancom_compat_specific_over_general.md`): "한컴 호환은 일반화보다 케이스별 명시 가드가 안전".

PR #392 의 변경 = **`col_count > 1` 구조 분기**:
- ✅ **구조 명시 가드** — 측정값 의존 아님 (col_count 는 SectionDef 의 컬럼 수)
- ✅ **확정적** — 같은 섹션이면 항상 같은 결과
- ✅ **회귀 추적 용이** — 분기 조건 단순

→ **메모리 원칙 부합**. PR #374 의 측정 의존 clamp 와 다른 안전한 접근.

## 누적 공식 분기의 시멘틱 정확성

### 작성자 분석 (PR 본문)

> 다단 layout 은 LINE_SEG `vpos` 기반 stacking 이므로 typeset 누적이 `total_height` (= `vpos_h + trail_ls`) 를 더하면 N items × trail_ls 만큼 인플레이션 발생 → 단을 조기 종료 → 항목들이 다음 단/페이지로 밀림.
>
> 단단 (k-water-rfp) 의 311px drift 는 layout 이 trailing_ls 를 stacking 에 포함시키는 별개 경로 — `total_height` 가 정합. 다단/단단 layout 의 stacking 차이가 본질.

### 검증 가능성

`RHWP_TYPESET_DRIFT=1` 진단 결과 (PR 본문):
- exam_eng pi=122: trail_ls=4.6, vpos_h=15.3, fmt_total=19.9, diff=+4.6
- exam_kor pi=1: trail_ls=9.2, vpos_h=235.9, fmt_total=245.1, diff=+9.2

→ 다단 layout 이 trailing_ls 를 stacking 에 포함하지 않음. typeset 누적이 `total_height` 면 인플레이션. `height_for_fit` 가 정합.

### 단단/다단 layout 의 stacking 차이

본 분석은 합리적이고 검증 가능. **다만 추가 검증 필요**:
- 다른 다단 샘플 (exam_kor 24p — pre-#359 와 동등) 의 정확한 시각 결과
- 작업지시자가 시각 판정으로 다단/단단 동작 확인

## 메인테이너 작업과의 관계

### Task #359 와의 관계

PR #392 가 정정 대상인 결함 = **Task #359 의 누적 공식 변경 (`current_height += total_height`) 이 다단에 미친 부작용**.

- Task #359: 단단 (k-water-rfp) 의 311px drift 정정 — 의도된 효과
- Task #359 부작용: 다단 (exam_eng) 의 11p 회귀 — 본 PR 정정 대상
- 본 PR: `col_count > 1` 분기로 단단 효과 보존 + 다단 정정

### Task #361, #362 와 관계

- Task #361 (page_num + PartialTable fit): 무관
- Task #362 (PartialTable + Square wrap): 무관
- **`next_will_vpos_reset` 가드는 그대로 유지** — Task #359 의 단독 항목 페이지 차단 효과 보존

## 회귀 검증 (PR 본문, 11 샘플)

| 샘플 | 단수 | pre-#359 | current devel | **PR #392 후** | 판정 |
|---|---|---|---|---|---|
| **exam_eng** | 2단 | 8p, 0 | **11p, 0** | **8p, 0** | ✓ 본 task 핵심 |
| exam_kor | 2단 | 24p, 30 | 30p, 0 | 24p, 30 | pre-#359 동등 |
| **k-water-rfp** | 1단 | 26p, 73 | 27p, 0 | **27p, 0** | ✓ #359 보존 |
| kps-ai | 1단 | 81p, 60 | 79p, 5 | 79p, 5 | 무변화 |
| aift | 1단 | 74p, 30 | 77p, 3 | 77p, 3 | 무변화 |

→ 단단 (1단) 모든 샘플 무변화. Task #359/#361/#362 효과 보존. exam_eng 만 정정.

## 변경 평가

### 강점
1. **명확한 결함 정정** — exam_eng 11p → 8p 회귀 정정
2. **구조 명시 가드** (`col_count > 1`) — 측정 의존 없음, 작업지시자 메모리 원칙 부합
3. **단단 효과 보존** — Task #359 의 k-water-rfp 정정 그대로 유지
4. **회귀 검증 정확** — 11 샘플 비교, pre-#359 baseline 까지 worktree 빌드로 확인
5. **신규 회귀 테스트** — `tests/exam_eng_multicolumn.rs` 추가로 재발 방지
6. **변경 범위 작음** (코드 3 줄)

### 약점 / 점검 필요
1. **mergeStateStatus = BEHIND** — devel rebase 필요
2. **다단 시멘틱의 깊은 이해 필요** — "다단 layout 은 vpos 기반 stacking" 이라는 작성자 가설을 작업지시자가 시각 판정으로 확인 필요
3. **planet6897 작성** — v0.7.6 회귀 origin 컨트리뷰터. 메모리 원칙 (작업지시자 직접 시각 검증 게이트) 적용
4. **PDF 환경 의존성** — exam_eng 의 한컴 출력이 작업지시자 환경 (한컴 2010 + 2022) 에서 8p 인지 시각 확인 필수

## 처리 방향 — 작업지시자 시각 판정 절차

작업지시자 지시: **시각적으로 판정할 수 있는 절차로 진행**.

### 절차

1. **현재 devel** (메인테이너 v0.7.7) 의 exam_eng 시각 (11p, 비대칭 단 채움)
2. **PR #392 적용 후** 의 exam_eng 시각 (8p 기대)
3. **작업지시자 한컴** 출력의 exam_eng 페이지 수 / 단 채움 비교
4. 두 결과 일치 시 머지 진행

### 필요 산출물

- PR 적용 전 / 후 SVG (디버그 오버레이 포함) — 작업지시자 시각 비교용
- exam_eng 의 한컴 출력 PDF (작업지시자 환경) — 정답지

## 권장

**옵션 A (단계적 진행 — 작업지시자 시각 판정)**:

1. PR #392 의 변경을 별도 브랜치 (`local/pr392`) 에 cherry-pick (devel 위에)
2. **시각 비교용 SVG 출력** (debug-overlay):
   - 현재 devel 의 exam_eng (11p, 회귀)
   - PR #392 적용 후 exam_eng (8p 기대)
3. **작업지시자 시각 판정**:
   - 한컴 출력 (작업지시자 환경) 과 비교
   - 다른 다단 샘플 (exam_kor 등) 도 비교
4. 시각 판정 통과 시 → 정상 cherry-pick 머지 + close
5. 시각 판정 실패 시 → 추가 정정 또는 close

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 (메인테이너의 컨트리뷰터, v0.7.6 회귀 origin) ⚠️
- [x] CI 통과 ✅
- [x] 코드 시멘틱 (구조 가드, 메모리 원칙 부합) ✅
- [x] 단단 효과 보존 (Task #359/#361/#362) ✅
- [x] 회귀 테스트 신설 ✅
- [ ] **작업지시자 시각 판정 (한컴 출력 비교)** — 필수
- [ ] 다른 다단 샘플 (exam_kor 등) 영향 확인

## 다음 단계

1. 시각 판정용 SVG 출력 진행
2. 작업지시자 직접 시각 비교 + 판정
3. 결과에 따라 머지 또는 정정

## 참고

- 이슈: [#391](https://github.com/edwardkim/rhwp/issues/391) (OPEN)
- PR: [#392](https://github.com/edwardkim/rhwp/pull/392) (OPEN, BEHIND)
- 관련 task: #359 (회귀 origin), #345 (exam_eng 9→8, 별개 원인)
- 작업지시자 메모리: `feedback_v076_regression_origin.md`, `feedback_hancom_compat_specific_over_general.md`
