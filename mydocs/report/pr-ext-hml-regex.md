# PR #????: 확장 프로그램 dev-tools-inject.js HML 링크 감지 정규식 보완

## 이슈
- **Issue**: #2689 — dev-tools-inject.js HML 링크 감지 정규식에 .hml 누락

## 분석

`rhwpDev.inspect()`는 페이지의 `<a>` 링크 중 HWP 문서 링크를 감지하여 `data-hwp="true"` 속성 누락을 검사한다. 감지 정규식에서 `.hml` 확장자가 누락되어 HML 문서 링크를 HWP 문서로 인식하지 못했다.

`content-script.js`는 올바르게 `.hml`을 포함하고 있으나 `dev-tools-inject.js`에만 누락되어 있었다.

## 변경

Chrome 및 Firefox의 `dev-tools-inject.js`에서 정규식 보완:

```javascript
// before
const isExt = /\.(hwp|hwpx)(\?.*)?$/i.test(href);
// after
const isExt = /\.(hwp|hwpx|hml)(\?.*)?$/i.test(href);
```

## 검증

- `.hwp` 링크: 기존과 동일하게 감지
- `.hwpx` 링크: 기존과 동일하게 감지
- `.hml` 링크: 신규 감지
- `.hwp?query=1`: 쿼리 파라미터 처리 기존과 동일

## 결과
- **Branch**: `pr/fix-issue-2689-ext-hml-regex`
- **PR**: https://github.com/edwardkim/rhwp/pull/???? (생성 후 업데이트)
- **Closes**: #2689
