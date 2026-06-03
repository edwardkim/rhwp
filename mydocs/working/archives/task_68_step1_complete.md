# Task #68 — 1단계 완료보고서

## CLI 옵션 파싱 + SvgRenderer 폰트 수집 + --font-style 구현 ✅

### 수정 파일

- `src/renderer/svg.rs` — FontEmbedMode 열거형, 폰트 수집, generate_font_style()
- `src/document_core/queries/rendering.rs` — render_page_svg_with_fonts()
- `src/main.rs` — CLI 옵션 파싱 (--font-style, --embed-fonts, --font-path)

### 변경 내용

1. **FontEmbedMode 열거형**: None / Style / Subset / Full
2. **폰트/글자 수집**: TextRun 렌더링 시 font_family + char 자동 수집
3. **--font-style**: @font-face local() 참조 CSS 생성, SVG `<style>` 삽입
4. **--embed-fonts / --embed-fonts=full**: CLI 파싱 완료, 서브셋 구현은 2단계
5. **--font-path**: CLI 파싱 완료, 경로 탐색은 2단계

### 검증 결과

- `--font-style` 옵션: SVG에 @font-face local() 정상 삽입
- 기본 동작 (옵션 없음): 기존과 동일 (style 미삽입)
- `cargo test`: 783 passed, 0 failed
