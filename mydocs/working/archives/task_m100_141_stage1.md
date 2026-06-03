# 1단계 완료보고서: SVG/Canvas 렌더러에 font-family 적용

- **타스크**: [#141](https://github.com/edwardkim/rhwp/issues/141)
- **마일스톤**: M100
- **브랜치**: `local/task141`
- **작성일**: 2026-04-14

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/renderer/equation/svg_render.rs` | `EQ_FONT_FAMILY` 상수 정의, 8개 `<text>` 생성에 font-family 추가 |
| `src/renderer/equation/canvas_render.rs` | `set_font()` 함수에 수식 폰트 체인 적용 |

## font-family 체인

```
'Latin Modern Math', 'STIX Two Math', 'Cambria Math', 'Pretendard', serif
```

## 검증 결과

- `cargo test`: 785개 전체 통과
- exam_math.hwp 1페이지 SVG 출력: 88개 `<text>` 요소에 font-family 정상 포함

## 다음 단계

2단계: Latin Modern Math woff2 번들링
