# 최종 결과 보고서 — Task #109

**이슈**: [#109](https://github.com/edwardkim/rhwp/issues/109)
**타이틀**: Chrome 확장 호버카드 썸네일 UX 개선 — 클릭 문서 열기 + hover 애니메이션
**마일스톤**: M100
**완료일**: 2026-04-12
**브랜치**: `local/task109`

---

## 결과 요약

호버카드 썸네일에서 사용자가 클릭 가능 여부를 인식하지 못하던 문제와
시각적 피드백이 없던 문제를 CSS 수정으로 해결하였다.

---

## 원인 분석

카드 전체(`rhwp-hover-card`)에 `cursor: pointer`와 click 이벤트가 이미 연결되어 있었으나,
`img` 요소는 user-agent 기본 스타일로 `cursor: default`가 적용되어
썸네일 위에서 커서가 화살표로 표시되었다.

hover 애니메이션은 아예 정의되지 않아 썸네일이 정적으로만 표시되었다.

---

## 수정 내용

### `rhwp-chrome/content-script.css`

| 선택자 | 추가 속성 |
|--------|----------|
| `.rhwp-hover-thumb` | `transition: transform 0.15s ease, box-shadow 0.15s ease` |
| `.rhwp-hover-thumb:hover` | `transform: scale(1.03)`, `box-shadow: 0 4px 12px rgba(0,0,0,0.15)` (신규) |
| `.rhwp-hover-thumb img` | `cursor: pointer` |

- `cursor: pointer` — 썸네일 위에서 손 모양 커서로 클릭 가능함 명시
- `scale(1.03)` — hover 시 3% 확대 (`overflow: hidden`으로 카드 영역 초과 방지)
- `box-shadow` — hover 시 그림자 강조로 입체감 부여
- `transition 0.15s` — 카드 fade-in 애니메이션과 동일한 속도로 일관성 유지

---

## 검증

| 항목 | 결과 |
|------|------|
| 썸네일 위 cursor | 화살표 → 손 모양 (pointer) ✅ |
| 썸네일 hover 애니메이션 | scale(1.03) + box-shadow 전환 ✅ |
| 썸네일 클릭 → 문서 열기 | 카드 click 이벤트 버블링으로 정상 동작 ✅ |

---

## 커밋 목록

| 커밋 | 내용 |
|------|------|
| `26cf93a` | 호버카드 썸네일 UX 개선 — cursor pointer + hover 애니메이션 |
