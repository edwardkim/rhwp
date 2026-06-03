# 단계5 완료보고서: Heavy display face → font-weight=bold 매핑

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **단계**: 5 / 6 (Heavy display face 시각 bold 근사)
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_146_v4.md`, `task_m100_146_v4_impl.md`

## 1. 수정 내역

### 1.1 `src/renderer/style_resolver.rs` — heavy face 판정 헬퍼

```rust
pub(crate) fn is_heavy_display_face(font_family: &str) -> bool {
    let primary = font_family.split(',').next().unwrap_or(font_family)
        .trim().trim_matches('\'').trim_matches('"');
    matches!(primary,
        "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
        | "HY견고딕" | "HY견명조" | "HY견명조B"
        | "HY그래픽" | "HY그래픽M"
    )
}
```

- `font_family` 의 primary face(체인 첫 항목)만 검사
- ASCII/단일/더블 따옴표 모두 제거 후 매칭

### 1.2 `src/renderer/mod.rs` — TextStyle::is_visually_bold

```rust
impl TextStyle {
    pub fn is_visually_bold(&self) -> bool {
        self.bold || crate::renderer::style_resolver::is_heavy_display_face(&self.font_family)
    }
}
```

### 1.3 `src/renderer/svg.rs` — 4곳 bold 분기 치환

- line 215 (문단 내 cluster 출력)
- line 1346 (큰 따옴표 안 cluster 출력 1)
- line 1420 (번호 매긴 큰 따옴표 출력 2)
- line 1787 (`draw_text` 메인 경로)

모두 `if style.bold { ... }` → `if style.is_visually_bold() { ... }` 로 치환.

### 1.4 단위 테스트 2건 (`src/renderer/layout/tests.rs`)

- `test_is_heavy_display_face_matches_known_heavy_faces`: 8개 heavy face true, 8개 일반 face false
- `test_is_heavy_display_face_with_family_chain`: 체인 내 primary face 기준 판정 + 따옴표 처리 검증

## 2. 검증 결과

### 2.1 SVG 출력 확인

`samples/text-align.hwp` 재출력:
```
<text x="75.58" ... font-family="HY헤드라인M,'Malgun Gothic',..." font-size="21.33" font-weight="bold" fill="#000000">□</text>
<text x="105.40" ... font-family="HY헤드라인M,..." font-size="21.33" font-weight="bold" fill="#000000">국</text>
```

**제목 전체에 `font-weight="bold"` 속성 추가 확인.** CharShape.bold=false 지만 primary face HY헤드라인M 이 heavy 리스트에 있어 시각 bold 적용.

### 2.2 시각 비교 (150dpi)

`output/compare/text-align/svg-chrome150-v4.png` vs `pdf-1.png`:
- 제목이 PDF 와 유사한 bold 두께로 렌더됨 (Malgun Gothic Bold fallback)
- 본문 / 표 / 주석 등 나머지 영역은 변화 없음

### 2.3 자동 테스트

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **933 passed** / 14 failed (14건 기존 실패, 본 PR 무관) |
| 신규 테스트 | `test_is_heavy_display_face_*` 2건 통과 |
| `cargo test --test svg_snapshot` | 3 passed (form-002/table-text/svg_snapshot_basic 모두 통과, heavy face 없어 영향 없음) |
| `cargo clippy --lib -- -D warnings` | clean |

**svg_snapshot 영향 없음**: 현재 golden 샘플들에 heavy display face 사용자가 없어 본 변경으로 인한 diff 가 발생하지 않았다. 이는 기존 golden 이 이미 "fallback 된 non-bold" 상태로 고정되어 있었다는 뜻이 아니라, 실제로 CharShape.bold=true 인 bold 텍스트들만 golden 에 포함되어 있었음을 의미한다.

## 3. 부수 영향

- `TextStyle::is_visually_bold()` 메서드는 SVG 경로(4곳) 외에는 호출되지 않으므로 다른 렌더러(WASM Canvas, HTML, PDF)에는 영향 없음.
- 향후 WASM Canvas 등에서도 같은 동작을 원하면 해당 렌더러에서 `is_visually_bold()` 를 호출하도록 추가 가능.

## 4. 커밋 계획

**커밋 1 (소스 + 테스트)**:
- `src/renderer/style_resolver.rs`
- `src/renderer/mod.rs`
- `src/renderer/svg.rs`
- `src/renderer/layout/tests.rs`
- 메시지: `Task #146: Heavy display face 를 visual bold 로 렌더`

**커밋 2 (문서)**:
- `mydocs/plans/task_m100_146_v4.md`
- `mydocs/plans/task_m100_146_v4_impl.md`
- `mydocs/working/task_m100_146_stage5.md`
- 메시지: `docs: Task #146 v4 계획서 + 단계5 보고서`

## 5. 다음 단계 (단계6)

- `cargo test` 전체 재확인 (v2~v4 영향 합산)
- 주요 샘플 스모크 스위프 (heavy face 사용 문서 관측)
- v4 최종 결과보고서 작성 (`mydocs/report/task_m100_146_report_v4.md`)
- orders/20260423.md 갱신
