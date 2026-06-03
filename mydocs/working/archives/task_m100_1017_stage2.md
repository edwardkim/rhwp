# Task M100 #1017 Stage 2 보고서 — replay plane contract 테스트 추가

## 1. 범위

구현계획서 승인 후 Stage 2 범위만 진행했다.

- Rust `PaintOp` 기준 replay plane helper 추가
- `Background -> BehindText -> Flow -> InFrontOfText` 순서 contract 고정
- `samples/복학원서.hwp` 1페이지 중앙 baked watermark의 raw order 회귀 테스트 추가
- native Skia / CanvasKit renderer 동작 변경은 아직 하지 않음

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `src/paint/replay_order.rs` | `PaintReplayPlane`, `PaintReplayPlane::ORDERED`, `paint_op_replay_plane()` 추가 |
| `src/paint/mod.rs` | replay order helper export |
| `tests/issue_1017.rs` | `복학원서.hwp` fixture 기반 raw PageLayerTree order 회귀 테스트 |

## 3. 확정한 contract

| PaintOp | replay plane |
|---------|--------------|
| `PaintOp::PageBackground` | `background` |
| `PaintOp::Image` + `text_wrap=BehindText` | `behindText` |
| `PaintOp::Image` + `text_wrap=InFrontOfText` | `inFrontOfText` |
| 그 외 모든 op | `flow` |

정렬 순서:

```text
background -> behindText -> flow -> inFrontOfText
```

## 4. 회귀 테스트 요약

`tests/issue_1017.rs`는 PageLayerTree JSON을 `root -> children -> ops` 순서로 직접 순회한다.

확인 항목:

- 첫 `textRun` op가 존재한다.
- baked watermark image op가 정확히 1개 존재한다.
- 해당 baked watermark는 raw tree order에서 첫 `textRun` 뒤에 있다.
- 해당 baked watermark는 `wrap="behindText"`, `mime="image/png"`, `bakedWatermark=true`이다.

즉 #1017의 실패 조건인 "payload는 BehindText로 resolved 되었지만 raw direct replay 순서로는 flow text 뒤에 그려질 수 있음"을 테스트로 고정했다.

## 5. 검증

실행:

```text
cargo fmt
cargo fmt --check
cargo test --lib paint::replay_order
cargo test --test issue_1017
```

결과:

```text
cargo test --lib paint::replay_order
5 passed

cargo test --test issue_1017
1 passed
```

비고:

- 최초 `cargo test --lib paint::replay_order` 실행은 sandbox 네트워크 제한으로 crates.io index 조회에 실패했다.
- 승인된 escalation으로 재실행 후 dependency lock/update와 테스트가 통과했다.
- 기존 코드의 warning 6개가 출력되었으나 Stage 2 변경과 무관하다.

## 6. 다음 단계

Stage 3에서는 native Skia PageLayerTree direct replay를 위 plane 순서로 multi-pass 순회하도록 변경한다.

Stage 3 진행 전 작업지시자 승인이 필요하다.
