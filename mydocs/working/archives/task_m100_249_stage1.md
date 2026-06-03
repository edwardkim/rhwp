# Task #249 단계별 완료보고서: 단계 1 — PUA 심볼 문자 렌더링

> Issue: [#249](https://github.com/edwardkim/rhwp/issues/249)
> 완료일: 2026-04-22

---

## 구현 내용

Wingdings 등 심볼 폰트의 PUA 영역(U+F000~F0FF) 문자를 `map_pua_bullet_char()`로 유니코드 표준 문자로 변환. SVG, Canvas, HTML 세 렌더러에 일관 적용.

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/svg.rs` | `draw_text` 내 `map_pua_bullet_char()` 호출 추가 |
| `src/renderer/web_canvas.rs` | `draw_text` 내 `map_pua_bullet_char()` 호출 추가 |
| `src/renderer/html.rs` | 텍스트 렌더링 내 `map_pua_bullet_char()` 호출 추가 |

## 주요 구현 사항

- U+F000~F0FF 범위 문자를 유니코드 표준 문자로 변환
  - ⇩⇧⇦⇨ 등 화살표, ●■◆ 등 도형, ✔☑ 등 체크마크
- 세 렌더러 경로에 동일한 변환 함수 적용 → 렌더러 간 일관성 확보

## 검증

- `cargo test`: 793개 통과, 0 실패
- Visual Diff: PUA 문자 포함 페이지에서 □(두부 문자) → 정상 심볼 표시 확인
