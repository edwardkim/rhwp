# PR #2609: Chrome/Firefox 컨텍스트 메뉴 .hml 누락 수정

## 이슈
- **Issue**: #2608 — 확장 컨텍스트 메뉴의 `targetUrlPatterns`에 `.hml` 누락

## 변경
`rhwp-chrome/sw/context-menus.js` + `rhwp-firefox/sw/context-menus.js`:
```js
// 추가
'*://*/*.hml',
'*://*/*.hml?*'
```

## 결과
- `.hml` 파일 링크도 "rhwp로 열기" 메뉴 사용 가능
- Closes #2608
