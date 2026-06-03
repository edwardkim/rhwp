# 단계별 완료 보고서 — Task #131 Stage 1

**이슈**: [#131](https://github.com/edwardkim/rhwp/issues/131)
**타이틀**: 한컴 단축키 호환성 전체 정비 — Ctrl+G 계열 완성
**작성일**: 2026-04-13

---

## 완료 내용

### chordMapG 추가 — Ctrl+G 계열 5개 단축키 구현

**파일**: `rhwp-studio/src/engine/input-handler-keyboard.ts`

| 단축키 | 기능 | 한글 IME |
|--------|------|---------|
| `Ctrl+G,C` | 조판 부호 보이기/숨기기 | `ㅊ` |
| `Ctrl+G,T` | 문단 부호 보이기/숨기기 | `ㅅ` |
| `Ctrl+G,P` | 화면 확대 쪽 맞춤 | `ㅍ` |
| `Ctrl+G,W` | 화면 확대 폭 맞춤 | `ㅈ` |
| `Ctrl+G,Q` | 화면 확대 100% | `ㅂ` |

- `chordMapG` 테이블 추가
- Chord 2번째 키 처리 블록에 `_pendingChordG` 추가
- `handleCtrlKey()`에 `Ctrl+G(ㅎ)` 1번째 키 처리 추가

### shortcutLabel 및 index.html 표기 정비

- `view:ctrl-mark` shortcutLabel: `Ctrl+G+C` → `Ctrl+G,C` (한컴 표기 통일)
- `index.html` 보기 메뉴:
  - 조판 부호: `Ctrl+G+C` → `Ctrl+G,C`
  - 문단 부호: 단축키 없음 → `Ctrl+G,T` 추가
  - 투명 선: 단축키 없음 → `Alt+V,T` 추가
  - 쪽 맞춤: 단축키 없음 → `Ctrl+G,P` 추가
  - 폭 맞춤: 단축키 없음 → `Ctrl+G,W` 추가
  - 100%: 단축키 없음 → `Ctrl+G,Q` 추가

---

## 검증

- `npx tsc --noEmit` 통과
- 표 관련 기존 단축키 미변경 확인

---

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | `chordMapG` + Chord 처리 추가 |
| `rhwp-studio/src/command/commands/view.ts` | `shortcutLabel` 표기 정비 |
| `rhwp-studio/index.html` | 보기 메뉴 `md-shortcut` 정비 |
