# Task M100-5433 완료 보고서 — canvaskit-parity 배치 2·3

- Issue: #5433
- Tracking: #536
- 브랜치: `feat/m11-parity-batches`
- 계획: `mydocs/tech/canvaskit-parity-implementation.md`
- 기계 결과: `mydocs/report/task_m100_5433_canvaskit_parity_batches.json`

## 1. 범위

MEGA QUEUE M11-p. 기존 하네스로 canvaskit-parity **배치 2(Paint Family Parity)** 와 **배치 3(Strict Text Variant Replay)** 를 실행하고 결과를 데이터로 남긴다.

하지 않은 것:

- #3772 / #3773 PDF 수정
- gym / M08
- `scripts/renderer_baseline_manifest.json` 변경 — 기존 renderer-contract·renderer_baseline 검토 절차가 임계값·샘플 추가를 요구하지 않았다

## 2. 기존 하네스

| 배치 | 이름 | 기존 진입점 |
| --- | --- | --- |
| 2 | Paint Family Parity | `cargo test --lib -- canvaskit_policy`, `rhwp-studio` CanvasKit image/resource/preflight 단위 시험, `node e2e/renderer-contract.test.mjs`, `python scripts/renderer_baseline.py --readiness-only` |
| 3 | Strict Text Variant Replay | `cargo test --lib -- text_variants`, `tests/canvaskit-text-variant-selection.test.ts`, SFNT/font-plan 단위 시험, 동일 renderer-contract, `npm run e2e:canvaskit-font-coverage` |

Windows에서 배치를 한 번에 돌리는 진입점이 없어 `scripts/canvaskit_parity_batches.py` 를 추가했다. 이 드라이버는 명령을 모으고 JSON을 남긴다. **매니페스트를 쓰지 않는다.**

```text
python scripts/canvaskit_parity_batches.py --list --batches 2,3
python scripts/canvaskit_parity_batches.py --batches 2,3 --output mydocs/report/task_m100_5433_canvaskit_parity_batches.json
python scripts/canvaskit_parity_batches.py --batches 2,3 --heavy
python -m unittest scripts/tests/test_canvaskit_parity_batches.py
```

`--heavy` 는 readiness 캡처와 font-coverage e2e다. 네이티브 `rhwp` 바이너리, Chrome/puppeteer, WASM, 수 GB 디스크가 필요하다.

## 3. Windows 실행 기록

호스트: Windows, Node v24.14.1, Python 3.13.12, rustc/cargo 1.93.1.

sparse-checkout 기본 목록에 `rhwp-studio`·`docs`·`crates` 가 없어 워크트리에서 `git sparse-checkout add rhwp-studio docs rhwp-vscode npm crates tools tests` 를 한 뒤에야 계약 시험과 cargo workspace가 열렸다.

| 작업 | 상태 | 판정 |
| --- | --- | --- |
| 배치 맵 unittest 6건 | 통과 | 드라이버가 매니페스트를 출력으로 가리키지 않음 |
| Studio CanvasKit 단위 30건 | 통과 (3.5s) | 배치 2·3 해당 파일 전부 |
| `node e2e/renderer-contract.test.mjs` | 통과 | `renderer backend contract guard passed` |
| `cargo test --lib -- canvaskit_policy text_variants` | 차단 | `rhwp` lib test 컴파일 중 `rustc-LLVM ERROR: IO failure on output stream: no space on device`. C: 잔여 0 GB. 이후 `target/` 삭제로 2.48 GB 회복. 제품 실패로 보지 않음 |
| `renderer_baseline.py --readiness-only` | 생략 | heavy. 네이티브 바이너리·브라우저·디스크 부족. **매니페스트 미변경** |
| `npm run e2e:canvaskit-font-coverage` | 생략 | heavy. 브라우저+WASM |

Studio 30건 구성: document-preflight 6, font-plan 4, image-header 5, resource-key 1, SFNT face 4, text-variant-selection 10.

text-variant-selection이 고정한 계약: 디코딩 가능한 sidecar 단독 선택, 손상·누락·과대·파싱 실패 자원은 TextRun 폴백, 그룹 독립 선택, 완전한 GlyphRun 우선, 불완전/중복/혼합 multipart는 폴백, outline이 있으면 GlyphRun을 평가하지 않음.

## 4. renderer_baseline 매니페스트

기존 절차는 `rhwp-studio/e2e/renderer-contract.test.mjs` 가 매니페스트 샘플·임계값을 고정하고, 변경은 계약 시험과 리뷰를 통과할 때만 한다.

이번 실행은 임계값 위반을 새로 관측하지 않았고 readiness 캡처도 돌리지 않았다. 따라서 `scripts/renderer_baseline_manifest.json` 은 그대로 둔다.

재실행 명령 (디스크·Chrome·WASM 준비 후):

```text
python scripts/renderer_baseline.py --readiness-only --scope representative --browser-mode headless --profiles screen --output output/renderer-baseline/m11p-batch2
```

## 5. 빠진 글루 / 후속

- Windows 기본 sparse-checkout 만으로는 `rhwp-studio` 와 `crates` 가 빠져 배치 시험이 시작되지 않는다. 이번 워크트리에서 cone을 넓혔다. 저장소 기본 sparse 정책은 바꾸지 않았다.
- 로컬 디스크가 수 GB 비기 전에는 `cargo test --lib` 와 readiness 캡처를 재현할 수 없다. 명령은 위와 드라이버 `--list` 에 있다.
- #3772 ExtraLight PDF bold, #3773 svg2pdf SubsetError 는 다른 M11 작업이다.

## 6. 검증

- `python -m unittest scripts/tests/test_canvaskit_parity_batches.py`
- `node --test` CanvasKit 단위 30/30
- `node e2e/renderer-contract.test.mjs`
- `cargo fmt --all -- --check`
- `node scripts/rust-test-suite-manifest.mjs --check`
- `node scripts/rust-unit-test-tiers.mjs --check`
