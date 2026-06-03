# 최종 보고서: 수식 폰트 적용

- **타스크**: [#141](https://github.com/edwardkim/rhwp/issues/141)
- **마일스톤**: M100
- **브랜치**: `local/task141`
- **작성일**: 2026-04-14

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/equation/svg_render.rs` | `EQ_FONT_FAMILY` 상수 정의, 8개 `<text>` 생성에 font-family 추가 |
| `src/renderer/equation/canvas_render.rs` | `set_font()` 함수에 수식 폰트 체인 적용 |
| `web/fonts/LatinModernMath-Regular.woff2` | Latin Modern Math woff2 추가 (382.6 KB) |
| `rhwp-studio/src/core/font-loader.ts` | `FONT_LIST`에 Latin Modern Math 항목 추가 |

## font-family 체인

```
'Latin Modern Math', 'STIX Two Math', 'Cambria Math', 'Pretendard', serif
```

## 검증 결과

- `cargo test`: 785개 전체 통과
- exam_math.hwp 1페이지 SVG 출력: 88개 `<text>` 요소에 font-family 정상 포함
- Latin Modern Math woff2: 382.6 KB (GUST Font License, 번들링 자유)
