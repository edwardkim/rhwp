# 1단계 완료 보고서 — Task #132

**단계**: 1단계 — package.json commands + menus 등록
**완료일**: 2026-04-13

## 변경 파일

- `rhwp-vscode/package.json`

## 변경 내용

`contributes`에 4개 커맨드와 2개 컨텍스트 메뉴 등록:

| 커맨드 | 타이틀 |
|--------|--------|
| `rhwp.print` | HWP: 인쇄 |
| `rhwp.exportSvg` | HWP: SVG로 내보내기 |
| `rhwp.debugOverlay` | HWP: 디버그 오버레이 보기 |
| `rhwp.dumpParagraph` | HWP: 문단 덤프 |

메뉴 등록:
- `explorer/context` — 탐색기 파일 우클릭 메뉴
- `editor/title/context` — 에디터 탭 우클릭 메뉴
- `when` 조건: `resourceExtname == .hwp || resourceExtname == .hwpx`
- `group`: `"rhwp@1"` ~ `"rhwp@4"` (HWP 그룹으로 묶임)
