# Task M100 #1017 Stage 6 보고서 — 통합 검증

## 1. 범위

Stage 6에서는 #1017 구현 결과를 통합 검증하고 최종 PNG 산출물을 확인했다.

구현 추가는 없었고, 검증과 최종 보고서 작성만 수행했다.

## 2. 검증 명령

실행:

```text
cargo fmt --check
cargo fmt --all -- --check
cargo test --test issue_1017
cargo test --test issue_938
cargo test --test issue_516
cargo test --lib canvaskit_policy
cargo test --features native-skia skia --lib
cargo test
cargo clippy -- -D warnings
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
cargo run --features native-skia --bin rhwp -- export-png samples/복학원서.hwp -p 0 -o output/task1017-final
```

## 3. 결과

| 검증 | 결과 |
|------|------|
| `cargo fmt --check` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --test issue_1017` | 2 passed |
| `cargo test --test issue_938` | 3 passed |
| `cargo test --test issue_516` | 8 passed |
| `cargo test --lib canvaskit_policy` | 13 passed |
| `cargo test --features native-skia skia --lib` | 32 passed |
| `cargo test` | 통과 |
| `cargo clippy -- -D warnings` | 통과 |
| `npm --prefix rhwp-studio test` | 45 passed |
| `npm --prefix rhwp-studio run build` | `pkg/rhwp.js` 부재로 실패 |
| native Skia PNG export | 성공 |

`npm --prefix rhwp-studio run build` 실패:

```text
src/core/wasm-bridge.ts(1,44): error TS2307: Cannot find module '@wasm/rhwp.js'
src/hwpctl/index.ts(377,57): error TS2307: Cannot find module '@wasm/rhwp.js'
```

해석:

- `rhwp-studio` 전체 build는 repo root의 `pkg/rhwp.js` WASM 산출물이 없어서 실패했다.
- `pkg/`는 `.gitignore` 대상이며, README/CLAUDE 기준으로 Docker WASM build가 생성한다.
- Stage 4에서 수정한 CanvasKit 소스는 별도 타입 체크와 `npm test`를 통과했다.

## 4. 최종 PNG 확인

생성 파일:

```text
output/task1017-final/복학원서.png
```

출력:

```text
output/task1017-final/복학원서.png (198675 bytes)
```

확인:

- 중앙 baked watermark가 본문 텍스트 위의 흰 사각으로 올라오지 않는다.
- 본문 텍스트가 watermark 위에 표시된다.
- `cargo run --features native-skia ... export-png` 중 기존 layout overflow 진단 1건이 출력되었으나, #1017 z-order 변경과 직접 관련 없는 기존 레이아웃 진단으로 판단했다.

```text
LAYOUT_OVERFLOW: page=0, sec=0, col=0, para=16, type=Shape, overflow=2.5px
```

## 5. 결론

#1017의 핵심 조건은 충족했다.

- native Skia direct replay: `background -> behindText -> flow -> inFrontOfText`
- CanvasKit direct replay: 같은 replay plane 순서 적용
- CanvasKit replay plan: paint op item에 `replayPlane` 진단 노출
- `복학원서.hwp` 중앙 baked watermark: `replayPlane="behindText"` 확인

최종 보고서는 `mydocs/report/task_m100_1017_report.md`에 작성했다.
