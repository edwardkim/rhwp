# 구현계획서 v4: Heavy display face → font-weight=bold

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_146_v4.md`

## 0. 진입점

- **헬퍼 추가 위치**: `src/renderer/style_resolver.rs` 또는 `src/renderer/mod.rs` (TextStyle 가 정의된 곳). 재사용성 고려해 `style_resolver.rs` 에 `pub(crate) fn is_heavy_display_face(&str) -> bool`.
- **SVG 출력 분기 수정**: `src/renderer/svg.rs` 아래 4곳에서 `style.bold` 조건을 `style.bold || is_heavy_display_face(&style.font_family)` 로 확장:
  - line 215 (run 문단 내 출력)
  - line 1346
  - line 1420
  - line 1787 (`draw_text` 메인 경로)

## 1. 단계5 — 코드 수정

### 1.1 헬퍼 함수

```rust
/// Heavy display 계열 face 는 face 자체가 굵어 browser 에서 fallback 될 때
/// regular 로 렌더되면 PDF(한컴) 출력과 시각 괴리가 크다. 이 리스트에
/// 포함된 face 는 SVG 출력 시 font-weight="bold" 를 강제해 근사 렌더.
pub(crate) fn is_heavy_display_face(font_family: &str) -> bool {
    // font_family 는 "HY헤드라인M,'Malgun Gothic',..." 처럼 체인일 수 있음.
    // 첫 face 만 검사 (HWP 가 지정한 primary face).
    let primary = font_family.split(',').next().unwrap_or(font_family).trim().trim_matches('\'');
    matches!(primary,
        "HY헤드라인M" | "HYHeadLine M" | "HYHeadLine Medium"
        | "HY견고딕" | "HY견명조" | "HY견명조B"
        | "HY그래픽" | "HY그래픽M"
    )
}
```

### 1.2 SVG 출력 수정

4개 callsite 동일 패턴:

```rust
- if style.bold { attrs.push_str(" font-weight=\"bold\""); }
+ if style.bold || crate::renderer::style_resolver::is_heavy_display_face(&style.font_family) {
+     attrs.push_str(" font-weight=\"bold\"");
+ }
```

또는 중복 조건을 피하기 위해 TextStyle 에 `computed_bold()` 메서드 추가:

```rust
impl TextStyle {
    pub fn is_visually_bold(&self) -> bool {
        self.bold || crate::renderer::style_resolver::is_heavy_display_face(&self.font_family)
    }
}
```

→ svg.rs 4곳에서 `if style.is_visually_bold() { ... }` 로 통일.

본 구현에서는 **메서드 방식**(후자) 채택. 중복 최소화 + 차후 확장성(weight 단계별 정교화) 용이.

### 1.3 단위 테스트

`src/renderer/layout/tests.rs` 또는 `style_resolver` 근접 테스트 모듈에 추가:

- `test_is_heavy_display_face_matches_known_heavy_faces`: 8개 face 모두 true, 일반 face 들 (Malgun Gothic, 함초롬바탕, 바탕 등) false 검증
- `test_is_heavy_display_face_with_family_chain`: `"HY헤드라인M,'Malgun Gothic',sans-serif"` 형태 체인에서 primary face 로 판정되는지 검증

### 1.4 재현 검증

```bash
cargo run --bin rhwp -- export-svg samples/text-align.hwp -o output/svg/text-align/
grep 'font-family="HY헤드라인M' output/svg/text-align/text-align.svg | head -1
# expected: font-weight="bold" 속성 포함
```

Chrome headless 150dpi 재생성 → PDF 비교: 제목이 PDF 와 시각적으로 유사한 bold 로 렌더.

### 1.5 단계5 커밋

- 커밋 1: 소스 + 테스트 — `Task #146: Heavy display face 를 visual bold 로 렌더`
- 커밋 2: v4 계획서 + 단계5 보고서 — `docs: Task #146 v4 계획서 + 단계5 보고서`

## 2. 단계6 — 통합 검증 + 최종 보고서

(v3 예정 stage5 흡수)

### 2.1 테스트 스위프

- `cargo test --lib` 전체
- `cargo test --test svg_snapshot` — 영향 샘플 확인, 필요 시 `UPDATE_GOLDEN=1`
- `cargo clippy --lib -- -D warnings`

### 2.2 스모크 스위프

heavy face 사용 가능성 높은 샘플:
- `samples/biz_plan.hwp` (제목/머리 기호 heavy face 가능성)
- `samples/exam_kor.hwp` (제목)
- `samples/form-002.hwpx` (공식 서식 제목)

### 2.3 결과보고서 v4

`mydocs/report/task_m100_146_report_v4.md` (v2/v3 보고서는 보존):
- v1~v4 전체 여정 간단 요약
- v4 스코프: heavy face 시각 bold 근사
- 좌표/시각 비교 표
- 종결 승인 요청

### 2.4 orders 갱신

`mydocs/orders/20260423.md` 에 v4 단계 체크리스트 추가.

### 2.5 단계6 커밋

- svg_snapshot golden (필요 시)
- v4 보고서 + orders 갱신

## 3. 산출물 체크리스트

- [ ] `src/renderer/style_resolver.rs` `is_heavy_display_face` + `pub(crate)` 노출
- [ ] `src/renderer/mod.rs` (또는 TextStyle 정의 파일) `is_visually_bold` 메서드
- [ ] `src/renderer/svg.rs` 4곳 분기 수정
- [ ] `src/renderer/layout/tests.rs` 단위 테스트 2건
- [ ] `mydocs/working/task_m100_146_stage5.md`
- [ ] `mydocs/working/task_m100_146_stage6.md`
- [ ] `mydocs/report/task_m100_146_report_v4.md`
- [ ] svg_snapshot golden (영향 시)
- [ ] orders/20260423.md 갱신
