# 단계별 완료보고서 — Task #107 단계 1

**이슈**: [#107](https://github.com/edwardkim/rhwp/issues/107)
**단계**: 1 / 1
**작성일**: 2026-04-11
**브랜치**: `local/task107`

---

## 완료 내용

### Chrome (`rhwp-chrome/content-script.js`, `content-script.css`)

- card 전체에 click 이벤트 추가 → `chrome.runtime.sendMessage({ type: 'open-hwp' })` + `hideHoverCard()`
- 액션 `div` 텍스트 제거 → 풋터 바(`rhwp-hover-action`)로 교체
  - 좌측: `▶ rhwp로 열기` (`rhwp-hover-action-label`)
  - 우측: `→` (`rhwp-hover-action-arrow`)
- CSS: `cursor: pointer`, `overflow: hidden`, `padding-bottom: 0`
- 카드 호버 시 풋터 배경 `#eff6ff`(파란빛) + 화살표 `translateX(3px)` 슬라이드

### Safari (`rhwp-safari/src/content-script.js`, `content-script.css` 신규)

- 동일 풋터 바 구조 적용 (`browser.runtime.sendMessage` 기반 card click은 기존 구현 유지)
- `content-script.css` 신규 생성 (manifest 선언은 있었으나 파일 미존재)
- Chrome과 동일한 스타일 적용

---

## 빌드 결과

`cd rhwp-chrome && npm run build` → 성공

---

## 커밋

`4569651` — Task #107: 호버 카드 클릭 구현 + 풋터 바 UX 개선 closes #107
