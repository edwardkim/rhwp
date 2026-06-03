# 구현 계획서 — Task #109

**이슈**: [#109](https://github.com/edwardkim/rhwp/issues/109)
**타이틀**: Chrome 확장 호버카드 썸네일 UX 개선 — 클릭 문서 열기 + hover 애니메이션
**마일스톤**: M100
**작성일**: 2026-04-12
**브랜치**: `local/task109`

---

## 수정 대상

`rhwp-chrome/content-script.css`

---

## 현재 코드 분석

```css
/* 현재 */
.rhwp-hover-thumb {
  margin-bottom: 8px;
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid #e2e8f0;
  /* hover 효과 없음 */
}

.rhwp-hover-thumb img {
  display: block;
  width: 100%;
  max-height: 200px;
  height: auto;
  object-fit: contain;
  /* cursor: pointer 없음 */
}
```

카드 전체(`rhwp-hover-card`)에 `cursor: pointer`가 있으나,
`img` 요소는 user-agent 기본 `cursor: default`로 재정의되므로
썸네일 위에서 커서가 화살표로 표시된다.

---

## 구현 단계

### 1단계: CSS hover 효과 추가

#### 변경 내용

```css
.rhwp-hover-thumb {
  margin-bottom: 8px;
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid #e2e8f0;
  transition: transform 0.15s ease, box-shadow 0.15s ease;  /* 추가 */
}

.rhwp-hover-thumb:hover {
  transform: scale(1.03);                                    /* 추가 */
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);               /* 추가 */
}

.rhwp-hover-thumb img {
  display: block;
  width: 100%;
  max-height: 200px;
  height: auto;
  object-fit: contain;
  cursor: pointer;                                           /* 추가 */
}
```

- `cursor: pointer` — 썸네일 위에서 손 모양 커서, 클릭 가능함 암시
- `transform: scale(1.03)` — hover 시 3% 확대 (`.rhwp-hover-thumb`에 `overflow: hidden`이 있어 img가 카드 밖으로 나가지 않음)
- `box-shadow` — hover 시 그림자 강조
- `transition 0.15s` — 카드 fade-in과 동일한 속도로 부드러운 전환

---

## 승인 요청

위 구현 계획서를 검토 후 승인해주시면 구현을 시작하겠습니다.
