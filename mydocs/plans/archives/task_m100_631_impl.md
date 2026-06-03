# Task #631 구현계획서

> **이슈**: [#631](https://github.com/edwardkim/zhwp/issues/631)
> **브랜치**: `local/task631`
> **작성일**: 2026-05-06
> **선행**: Stage 1 보고서 (`task_m100_631_stage1.md`)

---

## 원인 확정 (Stage 1 후속 진단)

### 진단 과정

1. `engine.rs::paginate_text_lines` 에 임시 eprintln 삽입 → **호출 안 됨** 확인
2. `rendering.rs:901` 에서 `RHWP_USE_PAGINATOR` 환경변수가 `1` 일 때만 `paginator(engine.rs)` 사용, 기본은 `TypesetEngine(typeset.rs::typeset_section)` 사용
3. 실제 페이지네이션 코드는 `src/renderer/typeset.rs::typeset_paragraph`

### 진짜 원인: `LAYOUT_DRIFT_SAFETY_PX` 이중 차감

`typeset.rs:872` `const LAYOUT_DRIFT_SAFETY_PX: f64 = 10.0;`

`typeset_paragraph` 줄 단위 분할에서 10px 안전 마진이 **두 군데**에서 차감:

```rust
// typeset.rs:1046
let base_available = (st.base_available_height() - LAYOUT_DRIFT_SAFETY_PX).max(0.0);
// = 971.4 - 10 = 961.4

// typeset.rs:1065-1067 (cursor_line==0 일 때)
let page_avail = (base_available - st.current_footnote_height - fn_margin
    - st.current_height - st.current_zone_y_offset).max(0.0);
// = 961.4 - 0 - 0 - 912.2 - 0 = 49.2

// typeset.rs:1074
let avail_for_lines = (page_avail - sp_b - LAYOUT_DRIFT_SAFETY_PX).max(0.0);
// = 49.2 - 0 - 10 = 39.2  ← 두 번째 차감
```

`pi=222` 시뮬레이션 (실제 본 페이지 잔여 = 971.4 − 912.2 = **59.2px**, 2줄 advance = 51.2px → 본래 들어가야 함):

| li | content_h | cumulative+content_h | avail_for_lines (39.2) | 결과 |
|----|-----------|----------------------|------------------------|------|
| 0  | 16.0      | 16                   | ✓ 통과                  | end_line=1, cumulative=25.6 |
| 1  | 16.0      | 41.6                 | ✗ 41.6 > 39.2 → break   | line 1 누락 |

→ end_line=1. 1줄만 page 18에 배치, line 1~3 page 19로 밀림.

### 알려진 트레이드오프 (Task #332 stage4b)

`git show 0211e574` (Task #332 stage4b):
> "stop drawing 정책은 콘텐츠 손실(21_언어 pi=10 line 1, hwp-multi-002 pi=68, **aift pi=222**) 발생으로 폐기. ... typeset.rs partial split 의 avail_for_lines 에서 LAYOUT_DRIFT_SAFETY_PX 추가 차감."
> "21_언어 pi=10 line 1 의 15.5px overflow 는 layout 의 본질적 drift (~48.9px) 으로 Stage 5 의 header 측정 통합에서 해결 예정."

→ **aift pi=222 는 Task #332 시점부터 알려진 보수적 마진의 부작용**. 본 task가 그 후속(Stage 5).

## 수정 전략

### 핵심 아이디어

HWP 파일의 **LINE_SEG vertical_pos** 값은 한컴 엔진이 직접 계산한 페이지 내 정답 좌표.
파일 자체에 "이 줄은 페이지 안에 있다"고 인코딩된 정보가 있다면, typeset 의 보수적
누적 추정(20px 마진)이 그것을 덮어쓰면 안 된다.

**HWP 권위값(line_seg.vertical_pos)으로 하단 경계 더블체크**:
- 각 line li 의 `vpos[li] + lh[li]` 가 본문 영역 안 (≤ body_height) 이면, 
  typeset 누적 추정이 부족하다고 판단해도 **그 줄은 현재 페이지에 포함**한다.
- 다음 line `li+1` 의 vpos == 0 (vpos-reset) 이면 그것은 명백한 페이지 경계 신호.

### 수정안 (단일 지점, 최소 변경)

`typeset.rs:1078-1086` 줄 단위 분할 루프에 HWP 권위값 더블체크 추가:

```rust
for li in cursor_line..line_count {
    let content_h = fmt.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line {
        // HWP 권위값 더블체크: line li 의 vpos+lh 가 body 안에 있고,
        // 다음 line(li+1)이 vpos-reset(=0) 이면 HWP 가 의도한 페이지 경계 → 포함
        let hwp_authoritative = check_hwp_line_fits(para, li, base_available_height, dpi)
            && next_line_is_vpos_reset(para, li);
        if !hwp_authoritative {
            break;
        }
    }
    cumulative += fmt.line_advance(li);
    end_line = li + 1;
}
```

조건을 좁게 (`next_line_is_vpos_reset`) 두어 회귀 위험 최소화.

### 대안 (검토 후 채택 여부 결정)

- **A. 마진을 1개로 줄임**: line 1074 의 `- LAYOUT_DRIFT_SAFETY_PX` 제거. 단순하지만 #332 회귀 위험 큼.
- **B. 마진 값을 20→15 로 미세 조정**: 더 위험. 임의값 회귀 우려.
- **C (채택)**: HWP 권위값 더블체크 — 명확한 신호(vpos-reset) 시에만 마진 우회.

C 가 가장 안전.

## 단계별 구현 (3단계)

### Stage 2: 권위값 더블체크 구현

- `typeset.rs:1078-1086` 줄 단위 분할 루프 수정
- 헬퍼 함수: `check_hwp_line_fits` (line vpos+lh ≤ body_height) + `next_line_is_vpos_reset`
- aift.hwp 18페이지 → pi=222 lines=0..2 확인 (2줄 page 18 진입)
- cargo build, 단일 샘플 export-svg 검증

### Stage 3: 회귀 검증

- `cargo test --lib` 전체 통과 확인
- 전체 샘플 SVG 비교 (Task #332 가 주의시한 21_언어, hwp-multi-002, aift 등)
- 페이지 수 변화 0 확인
- 회귀 발생 시 권위값 조건 더 좁힘 (예: vpos-reset 인접 line 만)

### Stage 4: 최종 보고서

- 수정 사항 정리, 회귀 0건 확인
- `mydocs/orders/20260506.md` 갱신
- local/devel merge 준비

## 제약 조건

- **수정 범위 한정**: `typeset.rs::typeset_paragraph` 의 partial split 루프 1곳만
- **회귀 0건**: 기존 샘플 픽셀 동일 (특히 21_언어 pi=10, hwp-multi-002 pi=68)
- **HWP 신호가 없는 케이스에는 영향 없음**: 더블체크 조건이 명확한 vpos-reset 신호 검증

## 산출물

- `mydocs/plans/task_m100_631_impl.md` (본 문서)
- `mydocs/working/task_m100_631_stage{2,3}.md`
- `mydocs/report/task_m100_631_report.md`
