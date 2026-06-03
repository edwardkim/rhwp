# Task #68 — 최종 완료보고서

## SVG export 폰트 서브셋 임베딩 ✅

### CLI 옵션

```bash
rhwp export-svg sample.hwp                          # 기본 (폰트 미포함)
rhwp export-svg sample.hwp --font-style             # local() 참조만
rhwp export-svg sample.hwp --embed-fonts            # 서브셋 임베딩
rhwp export-svg sample.hwp --embed-fonts=full       # 전체 임베딩
rhwp export-svg sample.hwp --font-path /path/fonts  # 폰트 경로 지정
```

### 수정 파일

| 파일 | 내용 |
|------|------|
| `Cargo.toml` | `subsetter`, `ttf-parser` 추가 (네이티브 전용) |
| `src/renderer/svg.rs` | FontEmbedMode, 폰트 수집, 서브셋/임베딩, 파일 탐색 |
| `src/document_core/queries/rendering.rs` | render_page_svg_with_fonts() |
| `src/main.rs` | CLI 옵션 파싱 + help 메시지 |

### 검증 결과

- Inkscape, Safari, Chrome, Edge 전 플랫폼 정상 렌더링
- 서브셋 임베딩: 25.4MB TTF → 51KB (173글자 기준, 99.8% 축소)
- `cargo test`: 783 passed, 0 failed
