# Task #404: 헤딩 문단이 후속 콘텐츠와 다른 페이지로 분리됨 — 구현계획서

## 핵심 원인 (사전 코드 조사 결과 확정)

`src/renderer/typeset.rs`의 `typeset_section()` (line 353~) 메인 루프는 다음 순서로 페이지 분할 결정을 한다:

1. (line 384~390) Multi-column / Column break
2. (line 393~401) `force_page_break` (Page/Section break or `page_break_before` 속성)
3. (line 403~419) **vpos-reset 가드** (#321) — 현재 문단 vpos=0이고 직전 문단 vpos가 큰 경우 강제 분할
4. (line 430~454) **단독 항목 페이지 차단** (#359) — 다음 문단이 vpos-reset 발동 예정이면 안전마진 비활성화
5. (line 460~488) 어울림 wrap-around 흡수
6. (line 492~501) `typeset_paragraph` 또는 `typeset_table_paragraph` 호출 (내부에서 height 누적 + 필요 시 분할)

**누락:** vpos가 페이지 본문 영역을 **초과하는 방향**의 검사는 없다. 현재 페이지의 `first_seg.vpos + body_h_in_hu`를 넘는 문단이 와도, 누적 height가 여유 있으면 그대로 배치 → orphan heading 발생.

## 시나리오 (pi=83 case)

| 항목 | 값 |
|------|----|
| body_h | 933.5 px = 70015 HU |
| page 9 첫 문단(pi=77) vpos | 532187 |
| page 9 vpos 한계 | 532187 + 70015 = 602202 |
| pi=83 vpos 시작 | 601325 |
| pi=83 vpos 끝 (예상) | 601325 + ~1100 = 602425 |
| 비교 | **602425 > 602202** → vpos 기준 9쪽 초과 |
| 누적 height 기준 | 9쪽 used=930.1 + heading 14.7 = 944.8 (마진 약간 초과지만 분할 미발동) |

→ vpos 기준으로 페이지 분할이 발동되어야 함. 누적 height 기준으로는 마진 차이로 잡지 못함.

## 구현 단계

### 1단계: vpos 추적 + 진단

**목적:** TypesetState에 `page_first_vpos` 필드 추가하고 페이지 시작 시 기록. pi=83 진입 시점에 `first_seg.vpos`, `page_first_vpos`, `body_h_in_hu`, `vpos_end` 값을 임시 로그로 출력하여 가설 확정.

**작업:**
- `TypesetState`에 `page_first_vpos: Option<i32>` 필드 추가
- `flush_column_always` / `force_new_page` / `advance_column_or_new_page` 등 페이지 전환 지점에서 `page_first_vpos = None`으로 reset (다음 문단의 first_seg.vpos가 새 page first vpos로 기록됨)
- `typeset_section` 메인 루프에서 `page_first_vpos`가 None이면 현재 문단의 `first_seg.vpos`로 초기화
- 임시 `eprintln!` 로 pi=83 진입 시 추적 값 출력

**산출물:** `mydocs/working/task_404_stage1.md`

**검증 기준:**
- 로그에서 pi=83의 `first_seg.vpos > page_first_vpos + body_h_in_hu` 가 실제로 참인지 확인
- 다른 paragraph (예: pi=77, pi=82)에서는 거짓이어야 함 (false positive 없음)

### 2단계: vpos 기반 fit 검사 추가

**목적:** vpos 기준으로 현재 페이지를 초과하는 문단을 다음 페이지로 강제 분할.

**작업:**
- `typeset_section` 메인 루프 line 419 (vpos-reset 가드 다음, 단독 페이지 차단 이전)에 vpos overflow 검사 추가
- 조건: `!current_items.is_empty()` AND `first_seg.vpos + estimated_height_in_hu > page_first_vpos + body_h_in_hu` (안전 여유 1mm = 283 HU 정도 허용)
- 발동 시 `st.advance_column_or_new_page()` 호출 → 현재 문단을 다음 페이지로 push
- 진단 로그 제거

**예외 처리:**
- `wrap_around_cs >= 0` 인 wrap-around 흡수 zone에서는 발동 안 함 (paragraph가 표 옆에 배치되므로 vpos 의미가 다름)
- `force_page_break` / `page_style_break` 직후에는 이미 새 페이지라 발동 안 함 (`current_items.is_empty()` 가드)
- multi-column zone에서는 단별 분리 동작이 다르므로 보수적으로 분할 보류 (col_count > 1 시 skip)

**핵심 변경 (의사 코드):**
```rust
// Task #404: vpos 초과 검사 - HWP의 vpos 기반 페이지 한계와 동기
if !st.current_items.is_empty()
    && st.wrap_around_cs < 0
    && st.col_count == 1   // 단일 단에서만 적용 (multi-column 회피)
{
    if let (Some(first_seg), Some(page_top_vpos)) =
        (para.line_segs.first(), st.page_first_vpos)
    {
        let body_h_hu = px_to_hwpunit(st.layout.body_area.height, self.dpi);
        let para_h_hu = px_to_hwpunit(para_estimated_height_px, self.dpi);
        let vpos_end = first_seg.vertical_pos + para_h_hu;
        let page_bottom_vpos = page_top_vpos + body_h_hu;
        // 1mm 안전 여유 (283 HU)
        if vpos_end > page_bottom_vpos + 283 {
            st.advance_column_or_new_page();
        }
    }
}
```

**산출물:** 코드 수정 + `mydocs/working/task_404_stage2.md`

**검증:** 빌드 통과 + pi=83이 10쪽으로 이동 확인.

### 3단계: 회귀 검증 + 최종 보고

**목적:** 광범위 샘플에서 회귀 없음 확인 + 최종 보고.

**작업:**
1. **타겟 검증** — 9쪽 SVG에 pi=83 부재, 10쪽 SVG에 pi=83 + pi=84/85 함께 표시 확인 (qlmanage)
2. **회귀 테스트** — `cargo test`
3. **샘플 회귀 검증** — 10개 대표 샘플 LAYOUT_OVERFLOW 카운트 비교 (수정 전/후)
4. **페이지 수 확인** — 본 샘플 30 → 30 또는 31. orphan heading 1건 이동만 영향이라면 큰 변화 없음

**산출물:**
- `mydocs/report/task_404_report.md`
- `mydocs/orders/{yyyymmdd}.md` 갱신

**검증 기준 (수행계획서 그대로):**
1. 9쪽 SVG에 pi=83 헤딩이 더 이상 표시되지 않음
2. 10쪽 SVG가 pi=83 헤딩 + pi=84/85 표를 함께 표시
3. 기존 회귀 테스트 모두 통과
4. 10개 대표 샘플 LAYOUT_OVERFLOW 카운트 회귀 없음

## 위험 / 영향 범위

- **변경 범위가 단일 함수**: `typeset.rs::typeset_section` 메인 루프의 한 부분 + TypesetState 한 필드. 격리됨.
- **Multi-column / wrap-around 회피**: 보수적으로 단일 단에서만 발동, false positive 가능성 차단.
- **#321 vpos-reset 가드와 보완 관계**: #321은 vpos=0 이면 새 페이지로, 본 타스크는 vpos가 페이지 한계를 넘으면 새 페이지로. 둘이 상호 배타적이므로 순서 의존성 낮음.
- **#402 의존**: 본 타스크는 #402의 height 누적 정확도 개선 위에서 의미가 있다. #402 미적용 상태(현재 task404 base)에서는 다른 paragraph 배치라 검증 어려움 → #402 merge된 devel을 base로 rebase 후 검증해야 함.

## #402와의 관계

- **base 의존**: 현재 `local/task404`는 devel 기반(=#402 미적용). 검증 단계에서는 `local/task402` HEAD에서 cherry-pick 또는 rebase 후 검증
- **PR 순서**: #402 PR이 먼저 merge된 다음 #404 작업

## 비고

- ParaShape `keep_with_next` 속성은 본 케이스(ps_id=11, kwn=0)에 사용 불가하지만, 향후 활성 케이스가 발견되면 별도 보강 가능 (본 타스크 범위 밖).
- `engine.rs::paginate_with_measured`(현재 fallback 경로)에는 부분적 vpos 보정 로직이 이미 있으나 좁은 조건. 본 수정과 별개로 후속 정리 가능.
