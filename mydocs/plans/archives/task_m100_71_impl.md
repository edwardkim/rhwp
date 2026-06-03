# Task #71: CodeQL 보안 경고 8건 수정 — 구현 계획서

## 1단계: CI/CD 워크플로우 최소 권한 설정 (경고 6,7,8)

### `.github/workflows/ci.yml`

`build-and-test` job에 permissions 추가:
```yaml
permissions:
  contents: read
```

`wasm-build` job에 permissions 추가:
```yaml
permissions:
  contents: read
```

### `.github/workflows/npm-publish.yml`

각 job에 명시적 permissions 추가:

| job | permissions |
|-----|-----------|
| build-wasm | `contents: read` |
| publish-npm-core | `contents: read`, `id-token: write` |
| publish-npm-editor | `contents: read`, `id-token: write` |
| publish-vscode | `contents: read` |

## 2단계: XSS 취약점 수정 (경고 2,3,4)

### `web/clipboard_test.html:74` (경고 4, error)

현재:
```javascript
html += `<div class="rendered">${data}</div>`;
```

수정: iframe sandbox로 격리 렌더링
```javascript
html += `<div class="rendered"><iframe sandbox srcdoc="${escapeHtml(data)}"></iframe></div>`;
```

### `web/app.js:108` (경고 3)

현재:
```javascript
panel.innerHTML = `...${fileName}...${info.version}...`;
```

수정: DOM API로 요소 구성, `textContent`로 값 삽입
```javascript
function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}
panel.innerHTML = `...${escapeHtml(fileName)}...${escapeHtml(info.version)}...`;
```

### `web/editor.js:1157` (경고 2)

현재:
```javascript
const plainText = html.replace(/<[^>]*>/g, '').trim();
```

수정: DOMParser로 안전하게 텍스트 추출
```javascript
const parsed = new DOMParser().parseFromString(html, 'text/html');
const plainText = (parsed.body.textContent || '').trim();
```

## 3단계: SSL/TLS + 평문 로깅 수정 (경고 1,5)

### `web/https_server.py:17` (경고 1)

현재:
```python
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
```

수정: TLS 1.2 최소 버전 명시
```python
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.minimum_version = ssl.TLSVersion.TLSv1_2
```

### `src/main.rs:1249` (경고 5)

현재:
```rust
println!("{}  hp[{}] ctrl[{}]: {}", prefix, hpi, hci, cn);
```

수정: 텍스트 내용을 30자로 절단 + 나머지 마스킹
```rust
let display = if cn.chars().count() > 30 {
    format!("{}...(truncated)", cn.chars().take(30).collect::<String>())
} else {
    cn.clone()
};
println!("{}  hp[{}] ctrl[{}]: {}", prefix, hpi, hci, display);
```

## 검증

- `cargo build` + `cargo test` 통과
- CodeQL 재실행 시 경고 0건
- 기존 기능에 영향 없음 확인
