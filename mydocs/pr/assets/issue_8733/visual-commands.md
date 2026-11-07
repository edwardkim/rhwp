# #8733 시각 측정 명령과 출처

- 검증 소스 head: `0352c9deee13130f374cfc8015384512ecc6256c` (제품 코드는 `e551fcc76e0794790e25a565f2509a4891eac6cb`와 동일).
- Native 빌드: Rust 1.93.1, `CARGO_TARGET_DIR=target/pr-review`, `CARGO_BUILD_JOBS=4`, `cargo build --locked --release --features native-skia --bin rhwp`.
- Native 실행 파일 SHA-256: `2284ff6defecbd40754af052d47d6bd63c2b2ab56a28b3ee3276552b8b5e15ec`.
- WASM: locked wasm-pack 0.15.0 release + wasm-opt -O. WASM SHA-256: `9a54d8e810e09e0a2e26de9a9f27ee10591765f350777408414a4dc3f5d7eb49`.
- Python에 Pillow, pypdfium2, fonttools를 사용했다. Node 24, Chrome 155, PDF 도구를 사용했다. `VISUAL_SWEEP_CHROME`은 `C:/Program Files/Google/Chrome/Application/chrome.exe`였다.
- 인쇄 profile, compat 2022, 96 DPI, PNG rasterizer 기본값, 페이지 제한 없음, page-workers 1, canvas scale 1.0이다.

## 고정 입력을 이용한 재현

아래 명령의 PDF 경로는 당시 출력과 SHA-256이 같은 공개 보존 사본이다. Native 측정은 이 고정 사본을 직접 사용했다. WASM은 확보 순서대로 1개·5개·1개로 나누어 당시 `output/pr-review/8733/hancom/` PDF를 사용했으며, 입력·출력 해시는 [Print manifest](hancom-print-manifest.json)와 일치한다. 마지막 표 HWP 리뷰만 연결 복구 후 `--resume`으로 마쳤다.

```powershell
$targets = @(
  @('cell-new-solid-hwp', 'cell-new-solid.hwp'),
  @('cell-new-solid-hwpx', 'cell-new-solid.hwpx'),
  @('table-new-solid-hwp', 'table-new-solid.hwp'),
  @('table-new-solid-hwpx', 'table-new-solid.hwpx'),
  @('imported-raw-zero-hwp', 'imported-raw-zero.hwp'),
  @('cell-cleared-zero-hwp', 'cell-cleared-zero.hwp'),
  @('table-cleared-zero-hwp', 'table-cleared-zero.hwp')
)
$targetArgs = @()
foreach ($target in $targets) {
  $targetArgs += @('--file-target', $target[0],
    "mydocs/pr/assets/issue_8733/$($target[1])",
    "pdf/issue_8733/observed_prints/$($target[0])-2020.pdf")
}
$common = @('--compat', '2022', '--dpi', '96',
  '--font-path', 'C:/Windows/Fonts', '--page-workers', '1')

python scripts/visual_sweep.py --rhwp-bin target/pr-review/release/rhwp.exe `
  @common @targetArgs --silhouette-only --out output/pr-review/8733/reproduce-native-scores
python scripts/visual_sweep.py --rhwp-bin target/pr-review/release/rhwp.exe `
  @common @targetArgs --out output/pr-review/8733/reproduce-native-review

python scripts/visual_sweep.py --rhwp-bin target/pr-review/debug/rhwp.exe --wasm-pkg pkg `
  @common @targetArgs --silhouette-only --out output/pr-review/8733/reproduce-wasm-scores
python scripts/visual_sweep.py --rhwp-bin target/pr-review/debug/rhwp.exe --wasm-pkg pkg `
  @common @targetArgs --out output/pr-review/8733/reproduce-wasm-review
```

WASM 실행 당시의 보조 CLI 경로는 위의 debug/rhwp.exe였다. 페이지는 fresh WASM Canvas2D로 그린다. 새 재현에서는 완성된 release/rhwp.exe를 보조 CLI로 사용해도 되지만 실제 측정 당시 경로와 구분한다.

## 판정 제한

각 backend 7쪽의 raw silhouette는 100.00%지만, raw0 대조군에는 양쪽 모두 가로 해치 누락이 있다. [Native 판독](native/inspection.json)과 [WASM 판독](wasm/inspection.json)을 함께 확인한다. 명시적 Print 100% 배율은 미확인이며 canonical 조판 baseline으로 승인하지 않았다. 설치된 Xpdf pdftotext 4.00의 `-bbox-layout` 미지원으로 marker 추출은 exit 99로 생략됐다. 글자가 없는 합성 셀의 PNG/PDFium 측정과 직접 판독은 수행했다.
