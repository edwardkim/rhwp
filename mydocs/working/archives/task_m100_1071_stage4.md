# Task M100-1071 Stage 4 WASM 빌드 및 작업지시자 판정 요청

## 1. 목표

Stage 4의 목표는 Stage 2~3에서 정정한 `shape-001.hwpx` TAC 도형 배치/커서 이동 변경을
WASM 산출물에 반영하고, rhwp-studio에서 작업지시자 시각/상호작용 판정을 받는 것이다.

## 2. 빌드 명령

`CLAUDE.md`의 WASM 전용 빌드 절차에 따라 Docker 빌드를 수행했다.

```text
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
[INFO]: :-) Done in 2m 42s
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.
```

## 3. 산출물

WASM 산출물:

```text
pkg/rhwp.js
pkg/rhwp_bg.wasm
```

웹 판정 혼선을 줄이기 위해 rhwp-studio public 산출물도 동기화했다.

```text
rhwp-studio/public/rhwp.js
rhwp-studio/public/rhwp_bg.wasm
```

파일 크기:

```text
pkg/rhwp.js       242K
pkg/rhwp_bg.wasm 4.7M
```

`pkg/rhwp_bg.wasm` 과 `rhwp-studio/public/rhwp_bg.wasm` 은 바이트 단위로 동일하다.

## 4. 판정 대상

샘플:

```text
samples/hwpx/shape-001.hwpx
samples/shape-001.hwp
pdf-large/hwpx/shape-001.pdf
```

판정 항목:

```text
1. HWPX shape-001.hwpx 첫 문단의 TAC 도형 가로 위치가 HWP/PDF 정답지와 맞는가
2. 첫 문단에서 좌우 커서 이동 시 TAC 도형이 한 글자처럼 지나가는가
3. SectionDef/ColumnDef 같은 구조 컨트롤 위치에서 커서가 불필요하게 한 번 더 멈추지 않는가
4. 도형 위/주변 hit-test 또는 cursor rect 가 역행하지 않는가
```

## 5. 사전 검증

Stage 4 전 Rust 검증:

```text
cargo fmt --check
cargo check
cargo test --test issue_1071_tac_cursor_nav
cargo test --test issue_1067_shape_rotation
cargo test --test hwpx_to_hwp_adapter stage4_section_def
cargo test --test issue_598_footnote_marker_nav
cargo test --lib
```

결과:

```text
cargo test --lib:
  1336 passed
  0 failed
  6 ignored
```

## 6. 판정 요청

작업지시자는 rhwp-studio에서 `samples/hwpx/shape-001.hwpx` 를 열고, 첫 문단 TAC 도형의
시각 위치와 좌우 커서 이동을 확인한다.

판정 결과 기록:

| 파일 | 시각 배치 | 커서 이동 | hit-test/cursor rect | 비고 |
|---|---|---|---|---|
| `samples/hwpx/shape-001.hwpx` | 성공 | 실패 | 실패 | 다각형 사이 공백은 출력되나 HWP/HWPX 모두 TAC 다각형을 한 글자처럼 커서 이동하지 못함 |

## 7. 판정 해석

작업지시자 판정 결과:

```text
- shape-001.hwpx: 다각형과 다각형 사이 공백 출력 성공
- HWP/HWPX 모두 다각형을 한 글자로 인식해서 커서 이동하기 실패
```

따라서 Stage 4의 WASM 판정은 다음처럼 분리한다.

```text
1. HWPX secPr control slot 정정으로 TAC 도형 사이 공백 배치는 해결됐다.
2. 그러나 실제 편집 커서에서 도형 단위를 통과하는 caret rect 가 아직 실패한다.
3. 다음 단계는 offset 이동 여부와 caret x 좌표를 분리해, cursor rect 계산 경로를 정정한다.
```
