# 4~5단계 완료 보고서 — Task #132

**단계**: 4단계 — rhwp.debugOverlay + rhwp.dumpParagraph / 5단계 — 빌드 검증
**완료일**: 2026-04-13

## 변경 파일

- `rhwp-vscode/src/extension.ts` (cmdDebugOverlay, cmdDumpParagraph 추가)

## 변경 내용

### cmdDebugOverlay

1. `initWasmHost()` → `new HwpDocument(fileBytes)`
2. `doc.set_debug_overlay(true)` 활성화
3. 페이지별 `renderPageSvg(i)` 호출 → SVG 배열 수집
4. `doc.set_debug_overlay(false)` 복원
5. 전 페이지를 하나의 HTML(`<div class="page">` 목록)로 구성
6. 임시 파일 `os.tmpdir()/rhwp-debug-{md5hash}.html` 저장
7. `vscode.commands.executeCommand("vscode.open", tmpFileUri)` — VS Code 내장 HTML 탭으로 열기

HTML 구조:
- 회색 배경에 흰색 페이지 카드 (box-shadow)
- 각 페이지 우상단에 "Page N" 라벨
- SVG 원본 그대로 포함 (디버그 경계선/라벨 포함)

### cmdDumpParagraph

1. `initWasmHost()` → `new HwpDocument(fileBytes)`
2. `doc.getSectionCount()`로 섹션 수 조회
3. QuickPick — 섹션 선택 (레이블: `섹션 N`, 설명: `문단 M개`)
4. QuickPick — 문단 선택 (레이블: `문단 N`)
5. `doc.getParaPropertiesAt(sec, para)` — ParaShape JSON
6. `doc.getLineInfo(sec, para, 0)` — LINE_SEG JSON (char_offset=0)
7. Output 채널 `"HWP Dump"`에 key=value 형식으로 출력
8. `dumpChannel.show(true)` — 포커스 전환 없이 채널 표시

`formatJson()` 헬퍼: 중첩 객체를 들여쓰기 포함 `key = value` 형식으로 변환.

Output 채널은 activate()에서 1회 생성 후 재사용.

## 5단계 빌드 검증

```
npm run compile

extension.js 71 KiB [emitted] [minimized]
  - fs, path, os, crypto, vscode: externals (node built-in)
  - @rhwp-wasm/rhwp.js: 217 KiB (alias 정상 해석)
  - src/extension.ts + hwp-editor-provider.ts + wasm-host.ts: 24.5 KiB

webpack 5.105.4 compiled successfully (경고: 폰트/WASM 크기 — 기존과 동일)
```

tsc --noEmit: 0 errors
