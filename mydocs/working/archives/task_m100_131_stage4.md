# 단계별 완료 보고서 — Task #131 Stage 4

**이슈**: [#131](https://github.com/edwardkim/rhwp/issues/131)
**타이틀**: 한컴 단축키 호환성 전체 정비 — 정책 문서 + 서식 메뉴 보완
**작성일**: 2026-04-13

---

## 완료 내용

### 단축키 정책 문서 작성

**파일**: `mydocs/tech/shortcut_policy.md`

- 브라우저 충돌 분석 전체 표 (28개 단축키)
- 재매핑 원칙 (Ctrl+Shift+* → Alt+Shift+*)
- Chord 단축키 전체 목록 (10개)
- 전역 단축키, 커맨드 팔레트, 한글 IME 이중 매핑 정책
- Shift+Enter 처리 방식 기록

### 서식 메뉴 보완

**파일**: `rhwp-studio/index.html`

서식 메뉴에 누락된 항목 추가:

| 항목 | 단축키 | 비고 |
|------|--------|------|
| 진하게 | `Ctrl+B` | 신규 |
| 기울임 | `Ctrl+I` | 신규 |
| 밑줄 | `Ctrl+U` | 신규 |
| 줄 간격 늘림 | `Alt+Shift+Z` | 신규 |
| 줄 간격 줄임 | `Alt+Shift+A` | 신규 |

### Shift+Enter

이미 구현되어 있음 (`input-handler-keyboard.ts` line 748). 추가 작업 불필요.

---

## 검증

- `npx tsc --noEmit` 통과

---

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `mydocs/tech/shortcut_policy.md` | 신규 — 단축키 정책 문서 |
| `rhwp-studio/index.html` | 서식 메뉴 진하게/기울임/밑줄/줄간격 항목 추가 |
