# Task #68 — 2단계 완료보고서

## --embed-fonts 서브셋/전체 임베딩 구현 ✅

### 수정 파일

- `Cargo.toml` — `subsetter = "0.2"`, `ttf-parser = "0.25"` 추가 (네이티브 전용)
- `src/renderer/svg.rs` — find_font_file(), 서브셋/전체 임베딩 로직

### 변경 내용

1. **find_font_file()**: 폰트명으로 TTF/OTF 파일 탐색
   - 탐색 순서: --font-path → ttfs/hwp/ → ttfs/windows/ → ttfs/ → 시스템 경로
   - macOS: /Library/Fonts, /System/Library/Fonts
   - Linux: /usr/share/fonts
   - Windows: C:\Windows\Fonts, WSL /mnt/c/Windows/Fonts

2. **--embed-fonts (서브셋)**:
   - ttf-parser로 codepoint → glyph ID 변환
   - subsetter::GlyphRemapper + subsetter::subset()로 서브셋 추출
   - base64 인코딩 → @font-face data: URL
   - 폰트 파일 없거나 서브셋 실패 시 local() 폴백

3. **--embed-fonts=full (전체)**:
   - TTF 파일 전체를 base64 인코딩
   - 서브셋 없이 직접 임베딩

### 검증 결과

- `cargo test`: 783 passed, 0 failed
- 폰트 파일 있을 때: 서브셋 추출 + base64 임베딩 (로그 출력)
- 폰트 파일 없을 때: local() 폴백으로 graceful degradation
- 현재 환경(macOS)에는 HWP 폰트 TTF 미설치 → local() 폴백 확인
