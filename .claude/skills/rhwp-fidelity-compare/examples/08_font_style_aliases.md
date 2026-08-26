# 예제 — 기본 --font-style

이슈 #5329. 실 에이전트 경로. gym 아님.

## 확인

하네스는 `RHWP_SVG_FONT_MODE` 기본 `style` → `--font-style`.
에이전트가 export-svg 를 직접 칠 때도 같은 플래그.

```bash
rhwp export-svg samples/doc.hwp -o /tmp/p.svg --font-style
rg -n "local\\(" /tmp/p.svg | head
```

`RHWP_SVG_FONT_MODE=full` 을 일상 비교에 올리지 않는다.

관련: `references/07_font_style.md`, `08_local_face_aliases.md`.
정지 F04.
