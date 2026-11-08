# PR #7905 메인테이너 검증 — 2026-10-29

- 원 PR code head: `b3f4b511053bc6bbb8d24f703ee687c5a574df1d`.
- 기준 devel: `f125c735c7d2d44eaa44b78c70465bc89af45fb1`.
- 실제 빌드한 통합 tree: `debf6b0ff3a5ce626e8f2e01235d7e89cc8f38da`.
- 입력과 독립 한컴 2018 Print: [원 기여자 증적](../legal_hwpx_cellsz_host_line/README.md). 동일한 공개 원본 6쪽.
- 모든 TSV는 96dpi, print profile, compat 2022, 색상 경계 보정 전 2px raw 실루엣의 실행 원본이다.
- Native: 실제 Skia PNG. WASM: 최적화 release와 Chrome 155.0.8059.79의 실제 Canvas2D PNG.
- 글꼴: 존재를 확인한 `/mnt/c/Windows/Fonts`와 `/mnt/c/Program Files (x86)/Hnc/Office 2020/HOffice110/Shared/Fonts`를 명시적으로 공급했다. 동일 카탈로그를 기준/후보 브라우저에 사용했다. HFT를 TTF로 바꿔 목표 PDF에 맞추거나 폰트/비교 허용치를 조정하지 않았다. 로컬 글꼴 파일은 공개하지 않는다.
- Native 후보 SHA-256: `493e020b9911935e61d0580328c6a541d8ca85c3470c969d496f08a21f34ba94`.
- WASM 후보 SHA-256: `6386c907b88f60e1f810db25307981179e6891c5a867994d39ac8ea5fa4c98bd`.
- WASM 기준 SHA-256: `6bbd2a8bbefb62180f984b6a3ddc01443af7d3ef0b446dd3ab4df01bd4f8b34e`. 기존 #8174의 `fa254` 산출물이며 해당 생산 source와 고정 devel 사이 Rust/Cargo 입력 차이가 없음을 확인했다.
- Native 기준은 검토 시작 시점의 기존 실행 파일을 보존해 재사용했다(SHA-256 `49d764e33237f7c7e867dcd10579a7e0d841b3b5abf1d62633dc513c77ec71fb`). 새 기준 Native 빌드와 구분한다. 기준 WASM의 정식 생산 기록과 동일 코드 입력을 함께 대조했다.

## 판독과 예외

Native와 WASM 모두 첫 페이지의 host 줄 누락이 개선됐다. 각 backend의 2~6쪽 PNG는 기준/후보 **바이트 동일**다. 최신 devel의 기존 3~6쪽 내용 소속 차이와 첫 페이지 QR 크기 차이는 해결했다고 주장하지 않는다. 1/2/3/6쪽 Native 및 1/3쪽 WASM 비교를 직접 확인했으며, 첫 페이지 본문 시작·마지막 본문/꼬리글, 2/3쪽의 제4조의5 본문과 그림 소속, 6쪽의 후속 본문 위치를 구분했다.

메인테이너가 2026-10-29에 **기존 결함에 한정한 예외와 원 PR merge**를 승인했다. 전쪽 90% 자동 gate는 기존 3~6쪽 때문에 계속 `re_review_required`다. 이 결과를 통과로 바꾸지 않았으며, 일반 정책·branch protection·다른 PR의 기준을 완화하지 않는다.

`native-controls/`는 `tests/fixtures/issue_7642/no-ls/`의 같은 이름 HWP와 `-print-2020.pdf` 13쌍·26쪽의 Native 원본 TSV다. 최저 99.99%이며 대상 법령 문서의 기존 차이를 상쇄하는 근거가 아니다.

## 재현

```bash
CARGO_TARGET_DIR=target/pr-review CARGO_BUILD_JOBS=8 \
  CARGO_PROFILE_RELEASE_TEST_OPT_LEVEL=1 CARGO_PROFILE_RELEASE_TEST_CODEGEN_UNITS=64 \
  cargo build --locked --profile release-test --features native-skia --bin rhwp
CARGO_TARGET_DIR=target/pr-review CARGO_BUILD_JOBS=8 \
  scripts/wasm-pack-locked.sh --target web --out-dir <검증용 pkg> --release
venv/bin/python scripts/visual_sweep.py --rasterizer png --silhouette-only \
  --rhwp-bin <검증용 Native> --compat 2022 --dpi 96 --embed-fonts full \
  --font-path /mnt/c/Windows/Fonts \
  --font-path '/mnt/c/Program Files (x86)/Hnc/Office 2020/HOffice110/Shared/Fonts' \
  --out <출력> --file-target jongbu-law \
  mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx \
  mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law_hancom2018_print.pdf
```

WASM은 같은 입력·글꼴 카탈로그·Canvas 옵션으로 `scripts/export-wasm-png-for-sweep.mjs`를 실행하고, 실제 PNG와 같은 PDF PNG를 `visual_sweep.py --silhouette-only --png-pair`로 비교했다. Chrome 경로를 `VISUAL_SWEEP_CHROME`으로 지정했다. compare/overlay/review는 같은 PNG에서 정본 `visual_sweep.py`의 함수를 호출해 생성했으며 점수·위치·크기를 바꾸지 않았다.

## TSV SHA-256

| 파일 | SHA-256 |
| --- | --- |
| `base-native.tsv` | `e148ed0b3948badf0e450a911843be798400c24ce66815b67e26c0a061277574` |
| `base-wasm.tsv` | `7aabc73507c95fe1a9f08bff47e1d184b6f67bea295f712fa154b1942a34a2a7` |
| `candidate-native.tsv` | `0baaaf360509455a674cdb97f3f81a3073c21ce71d8f776e49dd99a2ece831e8` |
| `candidate-wasm.tsv` | `19c708965ac3e32c907cc55618feb3959c1f370c4537e40711efe37732c31b95` |
