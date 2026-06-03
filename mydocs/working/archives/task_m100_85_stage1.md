# Task #85 — 1단계 완료보고서

## 원인 규명: iframe 방식 검증 ❌

### 테스트 결과

| 방식 | iOS Safari | macOS Safari |
|------|-----------|-------------|
| `tabs.create()` + 확장 페이지 | JS 미실행 (silent failure) | ✅ 정상 |
| iframe + `safari-web-extension://` URL | 네트워크 차단 (빨간색) | 미테스트 |

### 결론

iOS Safari는 **웹 페이지 컨텍스트에서 `safari-web-extension://` URL에 대한 접근을 차단**한다.
- `tabs.create()`: HTML 렌더링은 되지만 JS 실행 안 됨
- iframe: 네트워크 레벨에서 차단됨

### 남은 선택지

1. **Content script에서 직접 WASM 로드 + 렌더링** — 복잡도 높음
2. **웹 호스팅 뷰어로 리다이렉트** — 서버 필요
3. **네이티브 앱(Swift/WKWebView)에서 렌더링** — 확장과 별도 앱
