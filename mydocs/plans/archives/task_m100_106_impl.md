# 구현계획서 — Task #106: Chrome content-script XSS 수정

**이슈**: [#106](https://github.com/edwardkim/rhwp/issues/106)
**마일스톤**: M100 (v1.0.0)
**작성일**: 2026-04-11
**브랜치**: `local/task106`

---

## 수정 대상

**파일**: `rhwp-chrome/content-script.js`

**취약점 1** (line 100~143): `innerHTML = html` 문자열 대입  
**취약점 2** (line 192): `thumbDiv.innerHTML = \`<img src="${cached.dataUri}"...>\``  
**취약점 3** (line 209): `thumbDiv.innerHTML = \`<img src="${response.dataUri}"...>\``

**참조**: `rhwp-safari/src/content-script.js` — Task #84 수정 완료, DOM API 패턴 사용

---

## 단계 1 (단일): hover card DOM API 교체

### 1-1. `showHoverCard` 함수 — card 본문 생성 (line 100~143)

현재 `html` 문자열을 누적 후 `card.innerHTML = html` 대입하는 구조를 DOM API로 교체.

**유틸리티 함수 추가** (함수 상단 영역, `isHwpLink` 위):
```javascript
// DOM API로 안전하게 텍스트 요소 생성 (innerHTML 미사용 — H-01 XSS 방어)
function createEl(tag, className, text) {
  const el = document.createElement(tag);
  if (className) el.className = className;
  if (text != null) el.textContent = text;
  return el;
}
```

**썸네일 영역** (thumbnail 있을 때):
```javascript
// 전: html += `<div class="rhwp-hover-thumb"><img src="${thumbnail}" alt="미리보기"></div>`;
const thumbContainer = document.createElement('div');
thumbContainer.className = 'rhwp-hover-thumb';
const thumbImg = document.createElement('img');
thumbImg.src = thumbnail;
thumbImg.alt = '미리보기';
thumbImg.referrerPolicy = 'no-referrer';
thumbContainer.appendChild(thumbImg);
card.appendChild(thumbContainer);
```

**썸네일 플레이스홀더** (thumbnail 없을 때):
```javascript
// 전: html += `<div class="rhwp-hover-thumb rhwp-thumb-loading"><span class="rhwp-thumb-spinner">⏳</span></div>`;
const thumbContainer = document.createElement('div');
thumbContainer.className = 'rhwp-hover-thumb rhwp-thumb-loading';
const spinner = createEl('span', 'rhwp-thumb-spinner', '⏳');
thumbContainer.appendChild(spinner);
card.appendChild(thumbContainer);
```

**제목 영역**:
```javascript
// 전: html += `<div class="rhwp-hover-title">${title}</div>`;
//     html += `<div class="rhwp-hover-title">${fileName}</div>`;
const titleText = title || anchor.href.split('/').pop().split('?')[0];
card.appendChild(createEl('div', 'rhwp-hover-title', titleText));
```

**메타 영역** (format · pages · size):
```javascript
// 전: html += `<div class="rhwp-hover-meta">${meta.join(' · ')}</div>`;
const meta = [];
if (format) meta.push(format.toUpperCase());
if (pages) meta.push(`${pages}쪽`);
if (size) meta.push(formatSize(Number(size)));
if (meta.length > 0) {
  card.appendChild(createEl('div', 'rhwp-hover-meta', meta.join(' · ')));
}
```

메타값(`pages`, `size`, `format`)은 숫자/영문 위주지만 외부 입력이므로 `textContent` 사용.

**작성자·날짜 영역**:
```javascript
// 전: html += `<div class="rhwp-hover-info">${info.join(' · ')}</div>`;
if (author || date) {
  const info = [];
  if (author) info.push(author);
  if (date) info.push(date);
  card.appendChild(createEl('div', 'rhwp-hover-info', info.join(' · ')));
}
```

**카테고리 영역**:
```javascript
// 전: html += `<div class="rhwp-hover-category">${category}</div>`;
if (category) {
  card.appendChild(createEl('div', 'rhwp-hover-category', category));
}
```

**설명 영역**:
```javascript
// 전: html += `<div class="rhwp-hover-desc">${description}</div>`;
if (description) {
  card.appendChild(createEl('div', 'rhwp-hover-desc', description));
}
```

**액션 텍스트**:
```javascript
// 전: html += `<div class="rhwp-hover-action">클릭하여 rhwp로 열기</div>`;
// 후: (고정 문자열이므로 innerHTML도 안전하나, 패턴 통일을 위해 DOM API 사용)
card.appendChild(createEl('div', 'rhwp-hover-action', '클릭하여 rhwp로 열기'));
```

**`card.innerHTML = html;` 제거** — 더 이상 필요 없음.

### 1-2. 썸네일 동적 삽입 — `insertThumbnailImg` 헬퍼 추가

캐시 히트/Service Worker 응답 두 군데에서 동일 패턴 사용 중:
```javascript
thumbDiv.innerHTML = `<img src="${dataUri}" alt="미리보기">`;
```

헬퍼 함수로 통일:
```javascript
function insertThumbnailImg(thumbDiv, dataUri) {
  const img = document.createElement('img');
  img.src = dataUri;
  img.alt = '미리보기';
  img.referrerPolicy = 'no-referrer';
  thumbDiv.className = 'rhwp-hover-thumb';
  thumbDiv.appendChild(img);
}
```

사용처 교체:
- line 191~192: `insertThumbnailImg(thumbDiv, cached.dataUri);`
- line 208~209: `insertThumbnailImg(thumbDiv, response.dataUri);`

---

## 완료 기준

- [ ] `content-script.js` 내 `innerHTML` 문자열 대입 0건 (grep 확인)
- [ ] hover card 기능 정상: 썸네일/제목/메타/설명/액션 표시
- [ ] 빌드 성공 (`npm run build` in `rhwp-chrome/`)
