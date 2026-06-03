# Task M100 #1017 최종 보고서 — PageLayerTree replay z-order plane 적용

## 1. 목적

#1017은 #1016에서 resolved 된 image payload를 HWP의 `BehindText` / `InFrontOfText` 의미에 맞는 direct replay 순서로 합성하는 작업이다.

문제 fixture인 `samples/복학원서.hwp`의 중앙 baked watermark는 `wrap=behindText`로 resolved 되어 있었지만, raw PageLayerTree replay 순서상 본문 textRun 뒤에 있어 native Skia / CanvasKit direct renderer에서 글 위에 올라올 수 있었다.

## 2. 적용한 정책

direct replay backend가 PageLayerTree를 다음 logical plane 순서로 그리도록 했다.

```text
background -> behindText -> flow -> inFrontOfText
```

분류:

| op | replay plane |
|----|--------------|
| `PaintOp::PageBackground` | `background` |
| `PaintOp::Image` + `text_wrap=BehindText` | `behindText` |
| `PaintOp::Image` + `text_wrap=InFrontOfText` | `inFrontOfText` |
| 그 외 모든 op | `flow` |

## 3. 주요 변경

| 파일 | 변경 |
|------|------|
| `src/paint/replay_order.rs` | `PaintReplayPlane`, `paint_op_replay_plane()` 추가 |
| `src/paint/mod.rs` | replay order helper export |
| `src/renderer/skia/renderer.rs` | native Skia root multi-pass replay 적용 |
| `rhwp-studio/src/view/canvaskit/replay-plane.ts` | CanvasKit replay plane helper 추가 |
| `rhwp-studio/src/view/canvaskit-renderer.ts` | CanvasKit root multi-pass replay 적용 |
| `src/renderer/canvaskit_policy.rs` | replay plan item에 `replayPlane` 진단 추가 |
| `tests/issue_1017.rs` | fixture 기반 raw order / replay plan 회귀 테스트 |
| `rhwp-studio/tests/render-backend.test.ts` | CanvasKit plane helper/source regression 테스트 |

## 4. 검증 결과

```text
cargo fmt --check
cargo fmt --all -- --check
cargo test --test issue_1017                 # 2 passed
cargo test --test issue_938                  # 3 passed
cargo test --test issue_516                  # 8 passed
cargo test --lib canvaskit_policy            # 13 passed
cargo test --features native-skia skia --lib # 32 passed
cargo test                                   # passed
cargo clippy -- -D warnings                  # passed
npm --prefix rhwp-studio test                # 45 passed
```

native Skia 최종 PNG export:

```text
cargo run --features native-skia --bin rhwp -- export-png samples/복학원서.hwp -p 0 -o output/task1017-final
```

산출물:

```text
output/task1017-final/복학원서.png (198675 bytes)
```

확인 결과, 중앙 baked watermark는 본문 텍스트 뒤에 깔리고 흰 사각 배경이 본문 위에 올라오지 않는다.

## 5. 알려진 검증 제한

`npm --prefix rhwp-studio run build`는 repo root의 `pkg/rhwp.js` WASM 산출물 부재로 실패했다.

```text
Cannot find module '@wasm/rhwp.js'
```

이는 Stage 4/5 수정과 별개로, README/CLAUDE 기준 Docker WASM build가 생성하는 ignored 산출물(`pkg/`)이 없는 상태 때문이다.

## 6. 결론

#1017은 유효했고, direct renderer z-order contract를 native Skia와 CanvasKit 양쪽에 적용했다.

완료 조건:

- `BehindText` image는 raw tree/leaf 순서와 무관하게 flow op보다 먼저 replay된다.
- `InFrontOfText` image는 raw tree/leaf 순서와 무관하게 flow op보다 나중에 replay된다.
- CanvasKit replay plan에서 baked watermark item이 `replayPlane="behindText"`로 노출된다.
- #516 / #938 watermark 관련 회귀 테스트가 유지된다.

후속 처리:

- 구현 커밋: `06f0029e`
- Draft PR: https://github.com/edwardkim/rhwp/pull/1163
- 이슈 클로즈는 merge/작업지시자 승인 후 진행한다.
