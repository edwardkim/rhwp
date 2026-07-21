# PR #2626: hwpctl SaveAs HML 소스 포맷 지원 및 확장자 검증에 .hml 추가

## 이슈
- **Issue**: #2625 — SaveAs가 HML 소스 문서를 HWP로 잘못 내보내고 확장자 검증에 .hml 누락

## 분석

`rhwp-studio/src/hwpctl/index.ts`의 `SaveAs()`에 두 가지 문제가 있었다.

### 문제 1: HML 소스 문서의 포맷 손실

기존 분기 로직은 HWPX 여부만 이분법으로 판단했다:
```typescript
const isHwpx = format === 'hwpx' || (!format && sourceFormat === 'hwpx');
```

`sourceFormat === 'hml'`이면 `isHwpx = false`가 되어 `exportHwp()`를 호출한다.
WASM 브리지(`wasm-bridge.ts:318`)는 이미 `exportHml()`을 제공하지만
hwpctl 경로에서는 호출되지 않았다.

### 문제 2: filename 확장자 검증에 .hml 누락

```typescript
if (!filename.endsWith(ext) && !filename.endsWith('.hwp') && !filename.endsWith('.hwpx')) {
```

`.hml` 확장자가 조건에 없어 `SaveAs("문서.hml")` 호출 시
`report.hml.hwp`가 생성된다.

## 변경

1. **HML 포맷 분기 추가**: `isHml` 변수 도입 — format이나 sourceFormat이 'hml'이면 true
2. **exportHml() 호출**: isHml 분기에서 `this.wasmDoc.exportHml()` 사용
3. **MIME/ext 설정**: HML은 `application/xml` + `.hml`
4. **확장자 검증**: `.hml` 조건 추가 — 4개 확장자(hwp/hwpx/hml) 검증
5. **로그 메시지**: 포맷 레이블을 조건부 문자열로 표시

```typescript
// before
if (isHwpx) {
  bytes = this.wasmDoc.exportHwpx();
  mimeType = 'application/hwp+zip';
  ext = '.hwpx';
} else {
  bytes = this.wasmDoc.exportHwp();   // HML도 여기로 빠짐
  mimeType = 'application/x-hwp';
  ext = '.hwp';
}
if (!filename.endsWith(ext) && !filename.endsWith('.hwp') && !filename.endsWith('.hwpx')) {
  filename += ext;  // .hml을 인식 못 함
}

// after
if (isHml) {
  bytes = this.wasmDoc.exportHml();   // HML 유지 저장
  mimeType = 'application/xml';
  ext = '.hml';
} else if (isHwpx) {
  bytes = this.wasmDoc.exportHwpx();
  ...
}
if (!filename.endsWith(ext) && !filename.endsWith('.hwp') && !filename.endsWith('.hwpx') && !filename.endsWith('.hml')) {
  filename += ext;  // .hml 인식
```

## 검증

- 기존 HWP/HWPX 저장 경로는 isHml=false로 동일하게 동작
- HML 문서 저장 시 `exportHml()` 호출 확인
- `SaveAs("문서.hml")` → `filename` 그대로 유지
- WASM `exportHml()` 메서드는 wasm-bridge.ts에서 이미 검증 완료

## 결과
- **PR**: https://github.com/edwardkim/rhwp/pull/2626
- **Closes**: #2625
