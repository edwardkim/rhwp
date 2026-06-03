# 단계별 완료 보고서 — Task #131 Stage 2

**이슈**: [#131](https://github.com/edwardkim/rhwp/issues/131)
**타이틀**: 한컴 단축키 호환성 전체 정비 — 서식 단축키 + 브라우저 충돌 회피 재매핑
**작성일**: 2026-04-13

---

## 완료 내용

### 글꼴 크기 단축키 추가 — Ctrl+]/[

**파일**: `rhwp-studio/src/command/shortcut-map.ts`

| 단축키 | 기능 | 비고 |
|--------|------|------|
| `Ctrl+]` | 글자 크기 크게 | 브라우저 충돌 없음 |
| `Ctrl+[` | 글자 크기 작게 | 브라우저 충돌 없음 |

기존 `Alt+Shift+E/R`(한컴 원본)은 유지하고 추가 매핑으로 제공.

### 문단 정렬 단축키 추가 — 브라우저 충돌 회피 재매핑

| 단축키 | 기능 | 원래 한컴 | 충돌 사유 |
|--------|------|----------|---------|
| `Ctrl+Shift+L` | 왼쪽 정렬 | `Ctrl+Shift+L` | 브라우저 주소창 포커스(편집 영역 우선) |
| `Ctrl+Shift+M` | 양쪽 정렬 | `Ctrl+Shift+M` | 충돌 없음 |
| `Alt+Shift+H` | 오른쪽 정렬 | `Ctrl+Shift+R` | `Ctrl+Shift+R` = 브라우저 강제새로고침 |
| `Alt+Shift+C` | 가운데 정렬 | `Ctrl+Shift+C` | `Ctrl+Shift+C` = 브라우저 요소검사 |
| `Alt+Shift+D` | 배분 정렬 | `Ctrl+Shift+D` | `Ctrl+Shift+D` = 북마크 추가(Edge) |

> **`Alt+Shift+R` 충돌 해소**: 오른쪽 정렬에 `Alt+Shift+R`을 배정하면 기존
> `format:font-size-decrease`와 충돌. `H`(rigHt)로 재매핑.

### 서식 메뉴 항목 추가

**파일**: `rhwp-studio/index.html`

서식 메뉴에 글꼴 크기 증감 및 문단 정렬 항목 추가 (단축키 표시 포함):

- 글자 크기 크게 — `Alt+Shift+E`
- 글자 크기 작게 — `Alt+Shift+R`
- 왼쪽 정렬 — `Ctrl+Shift+L`
- 가운데 정렬 — `Alt+Shift+C`
- 오른쪽 정렬 — `Alt+Shift+H`
- 양쪽 정렬 — `Ctrl+Shift+M`
- 배분 정렬 — `Alt+Shift+D`

---

## 검증

- `npx tsc --noEmit` 통과
- 기존 표 조작 단축키 미변경 확인 (`table:insert-col-left`, `table:delete-col`)
- `Alt+Shift+R` 충돌 없음 (font-size-decrease 유지, align-right → `Alt+Shift+H`)

---

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `rhwp-studio/src/command/shortcut-map.ts` | 글꼴크기(Ctrl+]/[) + 정렬 단축키 추가 |
| `rhwp-studio/index.html` | 서식 메뉴에 글꼴크기/정렬 항목 추가 |
