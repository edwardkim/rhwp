# PR #2611: 다운로드 인터셉터 HWP_EXTENSION_RE에 .hml 추가

## 이슈
- **Issue**: #2610 — 다운로드 인터셉터가 .hml 파일을 감지하지 못함

## 변경
`rhwp-shared/sw/download-interceptor-common.js`:
```js
// before
export const HWP_EXTENSION_RE = /\.(hwp|hwpx)(\?|$)/i;
// after
export const HWP_EXTENSION_RE = /\.(hwp|hwpx|hml)(\?|$)/i;
```

## 결과
- 크롬/파이어폭스 다운로드 인터셉터가 .hml 파일도 감지
- Closes #2610
