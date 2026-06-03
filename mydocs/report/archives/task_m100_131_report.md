# 최종 결과 보고서 — Task #131

**이슈**: [#131](https://github.com/edwardkim/rhwp/issues/131)
**타이틀**: 한컴 단축키 호환성 전체 정비 — Ctrl+G 계열 완성 + 재매핑 + `/` 커맨드 팔레트
**마일스톤**: M100
**브랜치**: `local/task131`
**작성일**: 2026-04-13

---

## 구현 요약

4단계에 걸쳐 단축키 호환성 전체를 정비했다.

---

## 단계별 완료 내용

### Stage 1 — Ctrl+G 계열 완성

| 단축키 | 기능 | 한글 IME |
|--------|------|---------|
| `Ctrl+G,C` | 조판 부호 보이기/숨기기 | `ㅊ` |
| `Ctrl+G,T` | 문단 부호 보이기/숨기기 | `ㅅ` |
| `Ctrl+G,P` | 화면 확대 쪽 맞춤 | `ㅍ` |
| `Ctrl+G,W` | 화면 확대 폭 맞춤 | `ㅈ` |
| `Ctrl+G,Q` | 화면 확대 100% | `ㅂ` |

- `chordMapG` 테이블 구현
- `view:ctrl-mark` shortcutLabel 표기 `Ctrl+G+C` → `Ctrl+G,C` 통일
- 보기 메뉴 단축키 표시 정비 (조판/문단/투명선/쪽맞춤/폭맞춤/100%)

### Stage 2 — 서식 단축키 + 브라우저 충돌 회피 재매핑

| 단축키 | 기능 | 비고 |
|--------|------|------|
| `Ctrl+]` | 글자 크기 크게 | 추가 매핑 |
| `Ctrl+[` | 글자 크기 작게 | 추가 매핑 |
| `Ctrl+Shift+L` | 왼쪽 정렬 | 한컴 원본 유지 |
| `Ctrl+Shift+M` | 양쪽 정렬 | 한컴 원본 유지 |
| `Alt+Shift+H` | 오른쪽 정렬 | `Ctrl+Shift+R` → 브라우저 강제새로고침 충돌 |
| `Alt+Shift+C` | 가운데 정렬 | `Ctrl+Shift+C` → 브라우저 요소검사 충돌 |
| `Alt+Shift+D` | 배분 정렬 | `Ctrl+Shift+D` → Edge 북마크 충돌 |

서식 메뉴에 글꼴크기/정렬 항목 추가.

### Stage 3 — `/` 커맨드 팔레트 (`Ctrl+/`)

- `Ctrl+/` → 커맨드 팔레트 열기 (처음 `/`로 구현했다가 문서 입력 불가 문제로 `Ctrl+/`로 변경)
- 등록된 전체 커맨드 152개 실시간 검색
- 한글/영문 레이블, 커맨드 ID, 단축키 표시로 필터링
- `↑↓` 키 탐색, `Enter` 실행, `Escape` 닫기
- E2E 7개 TC 전체 통과

### Stage 4 — 정책 문서 + 서식 메뉴 보완 + 전역 단축키

- `mydocs/tech/shortcut_policy.md`: 브라우저 충돌 분석 전체 표(28개), 재매핑 원칙, Chord/IME/전역/팔레트 정책 기록
- 서식 메뉴에 진하게/기울임/밑줄/줄간격 항목 추가
- **전역 단축키 추가**: 문서 미로드 상태에서도 `Alt+N`으로 새 문서 생성 가능 (`setupGlobalShortcuts()`)

---

## 수정 파일 전체 목록

| 파일 | 내용 |
|------|------|
| `src/engine/input-handler-keyboard.ts` | `chordMapG`, `Ctrl+/` 팔레트, 전역 단축키 처리 |
| `src/engine/input-handler.ts` | `commandPalette` 필드/setter, `isActive()` |
| `src/command/shortcut-map.ts` | `Ctrl+]/[`, 정렬 단축키, 재매핑 |
| `src/command/commands/view.ts` | `shortcutLabel` 표기 정비 |
| `src/ui/command-palette.ts` | 신규 — 커맨드 팔레트 클래스 |
| `src/styles/command-palette.css` | 신규 — 팔레트 스타일 |
| `src/style.css` | CSS import 추가 |
| `src/main.ts` | `CommandPalette` 생성/주입, `setupGlobalShortcuts()` |
| `index.html` | 보기/서식 메뉴 단축키 표시 전면 정비 |
| `mydocs/tech/shortcut_policy.md` | 신규 — 단축키 정책 문서 |
| `e2e/command-palette.test.mjs` | 신규 — 팔레트 E2E 테스트 |
| `e2e/global-shortcut.test.mjs` | 신규 — 전역 단축키 E2E 테스트 |

---

## 검증

- `npx tsc --noEmit` 전 단계 통과
- E2E: 커맨드 팔레트 7개 TC 전체 PASS
- E2E: 전역 단축키 2개 TC 전체 PASS
- 기존 표 조작 단축키 미변경 확인

closes #131
