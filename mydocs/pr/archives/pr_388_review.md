# PR #388 검토 — 수식 렌더링 개선 (TAC 높이 + 한글 이탤릭) (#174, #175)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#388](https://github.com/edwardkim/rhwp/pull/388) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) |
| base / head | **`main`** ⚠️ ← `contrib/equation-rendering-improvements` |
| state | OPEN |
| mergeable | MERGEABLE |
| mergeStateStatus | **BLOCKED** (main 분기 보호 — REVIEW_REQUIRED) |
| isDraft | false |
| 이슈 | [#174](https://github.com/edwardkim/rhwp/issues/174), [#175](https://github.com/edwardkim/rhwp/issues/175) |
| 변경 통계 | +75 / -10, 4 files |
| CI | 전부 SUCCESS |

## 작성자 정보

신뢰 컨트리뷰터 (PR #387 와 동일 작성자):
- 머지 이력: PR #334 (replaceOne API), #335 (이미지 base64), #387 (밝기/대비 — base 정정 대기)

## 결함 요약

`samples/exam_math.hwp` 의 두 가지 수식 렌더링 결함:

1. **#174**: 큰 수식 (∑, 분수, lim 등) 이 인접 줄과 겹침 — 수식 TAC 높이가 줄 높이 (LINE_SEG) 계산에 미반영
2. **#175**: CASES 수식 내 한글+수학 혼합 텍스트가 모두 이탤릭으로 렌더 — 수학 변수명만 이탤릭이고 한글 설명 텍스트는 정체 (upright) 여야 함

## 변경 내용

### #174 — 수식 TAC 높이

**`paragraph_layout.rs`** (인라인 수식 처리):
```rust
// HWP 저장 높이를 우선 사용 (한컴 조판 결과 기준)
let hwp_eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
let eq_h = if hwp_eq_h > 0.0 { hwp_eq_h } else { layout_box.height };
// HWP 높이와 레이아웃 높이가 다르면 baseline 도 비례 조정
let eq_y = if hwp_eq_h > 0.0 && layout_box.height > 0.0 {
    let scale = hwp_eq_h / layout_box.height;
    (y + baseline - layout_box.baseline * scale).max(y)
} else { ... };
```

**`svg.rs`** (수식 SVG 렌더 시 scale):
```rust
let scale_y = if eq.layout_box.height > 0.0 && node.bbox.height > 0.0 {
    node.bbox.height / eq.layout_box.height
} else { 1.0 };
let needs_scale = (scale_x - 1.0).abs() > 0.01 || (scale_y - 1.0).abs() > 0.01;
// 기존: scale(x, 1) → 수정: scale(x, y)
```

### #175 — 한글 이탤릭 제거

**`equation/svg_render.rs`**:
```rust
// CJK/한글 텍스트는 이탤릭 없이 렌더링 (수학 변수명만 이탤릭)
let has_cjk = text.chars().any(|c| matches!(c,
    '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{AC00}'..='\u{D7AF}'
));
let style = if has_cjk { "" } else { " font-style=\"italic\"" };
```

**`equation/layout.rs`**: 한글 폭 산출 시 이탤릭 1.05배 보정 제외.

## ⚠️ 핵심 정황 — base=main (PR #387 와 동일)

본 PR 도 **base 가 main** 입니다. PR #387 와 동일한 정황으로 BLOCKED.

### 처리 방향

PR #387 의 댓글에서 동일 컨트리뷰터에게 **devel 기반 재PR** 안내 완료. 본 PR 도 같은 안내가 필요합니다.

## 메인테이너 작업과의 충돌 분석

### `paragraph_layout.rs`
- devel 의 변경: PR #366 의 PageNumberAssigner 등 — 다른 함수 영역 → 자동 머지 예상
- 본 PR 의 변경: 인라인 수식 처리 (line 1733 부근) → 자동 머지 가능성 높음

### `svg.rs`
- devel 의 변경: PR #373 의 형광펜 배경 (`draw_text` 함수) — 다른 함수 영역
- 본 PR 의 변경: 수식 렌더 (line 329 부근) → 자동 머지 예상

### `equation/svg_render.rs`, `equation/layout.rs`
- devel 의 변경: 없음 → 자동 머지

### 충돌 가능성
- 매우 낮음 (다른 함수/모듈 영역)
- PR #385 의 Serialize derive 가 equation 타입에도 적용됐는지 확인 필요 (확인 결과 ImageNode/PageRenderTree 중심, equation 모듈 영향 적음)

## 변경 평가

### 강점
1. **명확한 결함 정정** — 수식 TAC 높이 미반영 (#174) + 한글 이탤릭 부적절 (#175)
2. **HWP 권위값 사용** — `eq.common.height` 를 우선 사용 (한컴 조판 결과 기준)
3. **CJK 범위 정확** — `\u{3000}-\u{9FFF}`, `\u{F900}-\u{FAFF}`, `\u{AC00}-\u{D7AF}` (한자 + CJK 호환 + 한글)
4. **변경 범위 작음** (+75 / -10, 4 files)
5. **CI 통과**: cargo test 1010 (신규 2건 포함) + clippy + svg_snapshot 6
6. **신뢰 컨트리뷰터**

### 약점 / 점검 필요
1. **base=main** — PR #387 와 동일 (devel 로 변경 필요)
2. **HWP 권위값 일관성** — 본 task 의 정황 (한컴 PDF 환경 의존성) 적용 점검 필요. 본 PR 의 변경은 `eq.common.height` (HWP 파일의 저장값) 기반 → 환경 의존성 없음 (안전)
3. **다른 렌더러 미적용** — Canvas / HTML 의 수식 렌더는 별도 PR 후보일 수 있음

### 작업지시자 정답지 정책 부합 여부

본 PR 의 변경은 **HWP 파일 자체의 저장값 (`eq.common.height`)** 을 사용하므로 한컴 PDF 환경 의존성과 무관. 메모리의 정답지 정책 (한컴 2010+2022 편집기) 위반 없음.

## 처리 방향 후보

### 옵션 A: devel 기반 재PR 안내 (PR #387 와 동일)

PR #387 와 같은 절차 — close 후 작성자에게 devel 기반 재PR 요청.

### 옵션 B: cherry-pick 으로 devel 에 흡수

PR #366, #371, #373, #385 와 같은 방식 — 작성자 attribution 보존하며 메인테이너가 직접 cherry-pick.

장점: 작성자 부담 적음, 빠른 진행.
단점: PR #387 가 동일 정황인데 다른 처리 방식이면 일관성 깨짐.

### 옵션 C: PR #387 와 함께 일괄 처리

작성자 (oksure) 가 PR #387 + #388 두 개 모두 base=main. **두 PR 을 동일한 방식**으로 처리하는 것이 일관성 측면에서 적절.

## 권장

**옵션 A (PR #387 와 동일 — devel 기반 재PR 안내)**:

이유:
1. PR #387 와 일관 처리 (작성자에게 같은 메시지)
2. 작성자가 다음 PR 에서는 base 정확히 설정하도록 학습
3. 메인테이너가 이미 PR #387 로 안내 완료 — 본 PR 도 같은 안내 (수정 + 재PR)

cherry-pick 방식 (옵션 B) 은 작성자에게 base 정정 학습 기회 안 줌. PR #387 와 일관성도 깨짐.

## 다음 단계 — 작업지시자 결정

옵션 A / B / C 중 결정 부탁드립니다.

권장 — 옵션 A (PR #387 와 동일 안내):
1. 본 PR 댓글로 devel 기반 재PR 안내 (PR #387 와 같은 톤)
2. 작성자 close + 재PR 대기
3. 또는 메인테이너 직접 close (작성자에게 부담 없도록)

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 ✅
- [x] 코드 품질 (HWP 권위값 사용 + CJK 범위 정확) ✅
- [x] CI 통과 ✅
- [x] 메인테이너 동시 정정 없음 ✅
- [x] base=main 정황 — PR #387 와 동일 ⚠️
- [x] devel 충돌 가능성 (다른 함수 영역 → 낮음) ✅
- [x] 한컴 PDF 환경 의존성 정책 부합 (HWP 저장값 기반) ✅

## 참고

- 이슈: [#174](https://github.com/edwardkim/rhwp/issues/174), [#175](https://github.com/edwardkim/rhwp/issues/175) (둘 다 OPEN)
- PR: [#388](https://github.com/edwardkim/rhwp/pull/388) (OPEN, BLOCKED)
- 동일 작성자 다른 OPEN PR: [#387](https://github.com/edwardkim/rhwp/pull/387) (밝기/대비, 동일 base=main 정황)
