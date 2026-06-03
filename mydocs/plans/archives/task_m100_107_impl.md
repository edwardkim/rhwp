# 구현계획서 — Task #107: Chrome 확장 호버 카드 클릭 HWP 열기 구현

**이슈**: [#107](https://github.com/edwardkim/rhwp/issues/107)
**마일스톤**: M100 (v1.0.0)
**작성일**: 2026-04-11
**브랜치**: `local/task107`

---

## 현황 분석

- Chrome/Safari 양쪽 모두 '클릭하여 rhwp로 열기'가 단순 텍스트(`div`)로 UX가 불충분
- card 자체에 클릭 이벤트 없음
- CSS에 `cursor: pointer` 없음

---

## 단계 1 (단일): 호버 카드 클릭 동작 구현 + 풋터 바 UX

### UX 설계

```
┌──────────────────────────┐
│ [썸네일]                  │
│ 문서 제목                 │
│ HWP · 12쪽 · 2.3MB      │
│ 홍길동 · 2024-03-01      │
├──────────────────────────┤
│  ▶  rhwp으로 열기   →   │  ← 풋터 바 (배경 #f8fafc)
└──────────────────────────┘
```

- 카드 전체가 클릭 영역 (`cursor: pointer`)
- 하단 풋터 바: 배경 `#f8fafc`, 좌측 ▶ + "rhwp로 열기" + 우측 →
- 카드 호버 시 풋터 배경 `#eff6ff`(파란빛)로 미세하게 전환
- 풋터는 카드 가장자리까지 꽉 참 (`border-radius: 0 0 7px 7px`)
- 카드 내부 padding에서 풋터를 분리하기 위해 카드 `padding-bottom: 0` 처리

### 1-1. `content-script.js` — card 구조 변경

card에 `padding-bottom: 0` 인라인 스타일 추가 후 풋터 요소 생성:

```javascript
// card padding 하단 제거 (풋터가 카드 가장자리까지 닿도록)
card.style.paddingBottom = '0';

// 풋터 바
const footer = document.createElement('div');
footer.className = 'rhwp-hover-action';
const footerInner = document.createElement('span');
footerInner.className = 'rhwp-hover-action-inner';
footerInner.textContent = '▶  rhwp로 열기';
const footerArrow = document.createElement('span');
footerArrow.className = 'rhwp-hover-action-arrow';
footerArrow.textContent = '→';
footer.appendChild(footerInner);
footer.appendChild(footerArrow);
card.appendChild(footer);
```

card 전체 클릭 이벤트:

```javascript
card.addEventListener('click', () => {
  chrome.runtime.sendMessage({ type: 'open-hwp', url: anchor.href });
  hideHoverCard();
});
```

### 1-2. `content-script.css` — 풋터 바 스타일

```css
.rhwp-hover-card {
  /* 기존 유지 */
  cursor: pointer;
}

.rhwp-hover-card:hover .rhwp-hover-action {
  background: #eff6ff;
  color: #2563eb;
}

.rhwp-hover-action {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 8px -12px -12px;   /* 카드 padding 상쇄 */
  padding: 8px 12px;
  background: #f8fafc;
  border-top: 1px solid #e2e8f0;
  border-radius: 0 0 7px 7px;
  font-size: 12px;
  font-weight: 500;
  color: #64748b;
  transition: background 0.15s, color 0.15s;
}

.rhwp-hover-action-arrow {
  opacity: 0.5;
  transition: transform 0.15s, opacity 0.15s;
}

.rhwp-hover-card:hover .rhwp-hover-action-arrow {
  transform: translateX(3px);
  opacity: 1;
}
```

### 1-3. Safari 동일 적용

- `rhwp-safari/src/content-script.js` — 동일 구조 적용 (`browser.runtime.sendMessage`)
- Safari CSS 인젝션 방식 확인 후 동일 스타일 적용

---

## 완료 기준

- [ ] 호버 카드 클릭 시 HWP 파일이 rhwp 뷰어로 열림
- [ ] 풋터 바 표시 (▶ rhwp로 열기 + → 화살표)
- [ ] 카드 호버 시 풋터 파란빛 + 화살표 슬라이드
- [ ] Safari 동일 적용
- [ ] Chrome 확장 빌드 성공
