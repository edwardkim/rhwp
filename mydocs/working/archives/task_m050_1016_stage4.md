# Task M050 #1016 Stage 4 검증 완료보고서

## 1. 요약

`PaintOp::Image` resolved image payload 일반화 구현을 대상으로 PageLayerTree JSON, legacy SVG, Studio overlay, native Skia, browser CanvasKit 관련 계약을 검증했다.

핵심 확인 결과는 다음과 같다.

- `samples/복학원서.hwp` 1페이지 PageLayerTree JSON에서 watermark image op가 더 이상 원본 JPEG로 노출되지 않는다.
- watermark image op의 `mime`은 `image/png`이고 `bakedWatermark=true`가 유지된다.
- native Skia feature compile, test harness compile, Skia 단위 테스트, 실제 `export-png` 산출이 모두 통과했다.
- Studio Node 테스트는 통과했다.
- Studio build는 이 worktree에 `node_modules`가 없어 실행하지 못했다.

## 2. PageLayerTree JSON 검증

실행 명령:

```text
cargo test --test issue_516 issue_516_diag_count_image_ops -- --nocapture
cargo test --test issue_516 issue_516_diag_image_op_locations -- --nocapture
cargo test --test issue_938 issue_938_layer_tree_watermark_is_resolved_hancom_baked_png -- --nocapture
```

결과:

```text
image ops: 2, wrap=behindText: 2, mime png: 2, mime jpg: 0
```

image op 위치 진단:

```text
image op #0: bbox x=65.493, y=49.013, width=77.013, height=87.893, mime=image/png
image op #1: bbox x=137.707, y=270.240, width=495.040, height=495.733, mime=image/png
```

`issue_938_layer_tree_watermark_is_resolved_hancom_baked_png`에서 확인한 계약:

| 항목 | 기대 | 결과 |
|------|------|------|
| watermark image MIME | `image/png` | 통과 |
| `bakedWatermark` | `true` | 통과 |
| 원본 JPEG watermark op 미노출 | `mime=image/jpeg` 없음 | 통과 |
| 원본 effect metadata | `grayScale`, `brightness=-50`, `contrast=70` 유지 | 통과 |
| baked PNG 크기 | `728 x 729` | 통과 |
| alpha | opaque `255..255` | 통과 |
| gray tone 통계 | #976 SVG/overlay 기준 범위 | 통과 |

## 3. Rust 기본 회귀 검증

실행 명령:

```text
cargo check
cargo fmt --check
cargo test --test issue_938
cargo test --test issue_516
cargo test --test issue_514
cargo test --test svg_snapshot
```

결과:

| 명령 | 결과 |
|------|------|
| `cargo check` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_938` | 통과, 3 passed |
| `cargo test --test issue_516` | 통과, 8 passed |
| `cargo test --test issue_514` | 통과, 3 passed |
| `cargo test --test svg_snapshot` | 통과, 8 passed |

## 4. native Skia 검증

실행 명령:

```text
cargo check --features native-skia
cargo test --features native-skia --no-run
cargo test --features native-skia skia --lib
cargo run --features native-skia --bin rhwp -- export-png samples/복학원서.hwp -p 0 -o output/task1016-stage4
```

결과:

| 명령 | 결과 | 비고 |
|------|------|------|
| `cargo check --features native-skia` | 통과 | 최초 sandbox DNS 실패 후 승인된 네트워크 실행 |
| `cargo test --features native-skia --no-run` | 통과 | 기존 warning 6건만 존재 |
| `cargo test --features native-skia skia --lib` | 통과 | 30 passed |
| `cargo run --features native-skia --bin rhwp -- export-png ...` | 통과 | PNG 1개 생성 |

생성 산출물:

```text
output/task1016-stage4/복학원서.png
```

파일 확인:

```text
PNG image data, 794 x 1123, 8-bit/color RGBA, non-interlaced
size: 175080 bytes
```

주의:

- `cargo run --features native-skia -- export-png ...`는 binary가 둘 이상이라 실패했고, `--bin rhwp`를 명시해 재실행했다.
- native Skia 실제 binary build는 rust-skia binary 다운로드가 필요해 sandbox network에서는 실패했고, 승인된 네트워크 실행으로 확인했다.

## 5. Studio / CanvasKit 관련 검증

실행 명령:

```text
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
```

결과:

| 명령 | 결과 | 비고 |
|------|------|------|
| `npm --prefix rhwp-studio test` | 통과 | 25 passed |
| `npm --prefix rhwp-studio run build` | 미수행 | `tsc: command not found` |

`npm run build` 실패 사유:

```text
rhwp-studio node_modules가 없는 worktree 환경이라 tsc 실행 파일이 없음
```

이번 변경에서 browser CanvasKit은 별도 bake 로직을 추가하지 않는다. CanvasKit renderer는 PageLayerTree JSON의 image op `base64` / `mime`을 소비하므로, PageLayerTree JSON에서 watermark가 resolved PNG로 노출되는 계약을 테스트로 확인했다.

## 6. 남은 제한 사항

- Studio의 전체 TypeScript build는 `rhwp-studio` 의존성 설치 후 별도 확인이 필요하다.
- 이번 검증은 `samples/복학원서.hwp`의 PageLayerTree/Skia 산출 경로를 중심으로 수행했다. 실제 브라우저 CanvasKit 화면 시각 비교는 로컬 dev dependency 준비 후 별도 수행 가능하다.
- 기존 `LAYOUT_OVERFLOW` 진단 로그는 이번 변경 전부터 나타나는 레이아웃 진단이며, image payload resolved 계약과는 무관하다.

## 7. 다음 단계

작업지시자 승인 후 Stage 5 최종 보고서를 작성한다. 최종 보고서 승인 전에는 이슈 close를 수행하지 않는다.
