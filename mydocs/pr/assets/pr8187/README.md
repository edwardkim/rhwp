# #8187 메인터너 검증 증거

최종 판정과 원인·제한은 [검토 기록](../../pr_8187_review.md)을 따른다.
`platy-native-editing/`은 원 기여자가 제출한 과거 증거이며, 이 디렉터리는 최신 devel 통합 후보의
독립 실행 결과다. 같은 파일명의 예전 점수를 현재 측정값으로 재사용하지 않았다.

- `base/`: 기준 devel의 원본 13문서·439쪽/backend TSV.
- `final/{native,wasm}/`: 후보 원본 13문서와 fresh WASM 저장 KPS HWPX, 14문서·516쪽/backend TSV 및 대표 PNG.
- `final-visual-summary.json`: 정확한 쪽별 raw, 입력·독립 PDF·TSV 해시와 base 대비 하락 목록.
- `pixel-preservation.json`: 같은 backend에서 base/후보 PNG가 바뀐 쪽, 저장 재열기의 바이트 보존.
- `final/crops/`: 같은 페이지·영역의 PDF/Native/WASM 확대. 원본 래스터를 잘라 정수 배율로 확대한 진단이다.
- `final/svg-{native,wasm}/`: KPS 65·66쪽의 별도 SVG 소비 경계. Native PNG/Canvas 전쪽 TSV를 대체하지 않는다.
- `boundary-execution.json`: 원본 저장 프레임·테두리·뒤 내용 관계의 실제 base FAIL / 후보 PASS.
- `base-producers.json`, `final-producers.json`: source/producer·글꼴 지문·명령·환경.
- `local-validation.json`, `frontend-validation.json`: 필수 명령·모듈별 결과·환경·로그 지문.
- `evidence-check.json`: 28개 원본 TSV의 해시·전쪽 누락/미달·439쪽/base 비교 무결성.
- `native-postgate-smoke.json`: 마지막 Native CLI에서 주요 5쪽 PNG의 바이트 보존.
- `minimal-font-used.sh`: 실제 로컬 회귀에 사용한 파일 시스템 격리 recipe.
- `browser-api.mjs`, `browser-api-results.json`: fresh optimized WASM을 실제 Chromium에서 호출한 36개 API·왕복·Canvas 검사.
- `check-boundary-output.py`: 전쪽 비교의 RenderTree에서 경계 관계를 재확인하는 진단 도구. 정식 Rust 회귀를 대체하지 않는다.

원본과 독립 PDF는 기존 정식 경로를 사용한다. 15개 로컬 검증 글꼴은 지문만 기록했으며 파일을
재배포하지 않는다. producer의 전체 경로는 실행 당시 위치이고, 재실행 환경에서는 경로를 조정한다.

## 재현

저장소 루트에서 공용 target과 별도 producer 경로를 사용한다. root pkg/Studio public 파일에
검토 빌드를 덮어쓰지 않는다.

```bash
export CARGO_TARGET_DIR="$PWD/target/pr-review"
export CARGO_BUILD_JOBS=4
cargo build --locked --profile release-test --features native-skia
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 scripts/wasm-pack-locked.sh --target web \
  --out-dir "$PWD/output/pr-review/pr8187/head-wasm"
VISUAL_SWEEP_CHROME=/확인한/Chromium/chrome \
  node mydocs/pr/assets/pr8187/browser-api.mjs "$PWD/output/pr-review/pr8187/head-wasm"
```

글꼴 폴더는 `base-producers.json`의 검증된 로컬 공급과 동일하게 준비한다. 한컴 Print를 다른
버전의 PDF로 바꾸지 않는다. 아래 Native 예시에 이어 WASM 예제를 실행한다.

```bash
venv/bin/python scripts/visual_sweep.py --silhouette-only --page-workers 2 \
  --hwp rhwp-studio/public/samples/kps-ai.hwp \
  --pdf mydocs/pr/assets/platy-native-editing/inputs/kps-ai.hwp.hancom.pdf \
  --key kps-ai --compat 2022 --dpi 96 --embed-fonts full \
  --font-path output/pr-review/pr8187/validated-subset-fonts \
  --rhwp-bin target/pr-review/release-test/rhwp \
  --out output/pr-review/pr8187/recheck-native
VISUAL_SWEEP_CHROME=/확인한/Chromium/chrome \
  venv/bin/python scripts/visual_sweep.py --silhouette-only --page-workers 2 \
  --hwp rhwp-studio/public/samples/kps-ai.hwp \
  --pdf mydocs/pr/assets/platy-native-editing/inputs/kps-ai.hwp.hancom.pdf \
  --key kps-ai --compat 2022 --dpi 96 --embed-fonts full \
  --font-path output/pr-review/pr8187/validated-subset-fonts \
  --rhwp-bin target/pr-review/release-test/rhwp \
  --wasm-pkg output/pr-review/pr8187/head-wasm \
  --out output/pr-review/pr8187/recheck-wasm
```

전쪽 실행은 `final-producers.json`의 모든 `--file-target`을 사용한다. 저장 KPS HWPX는 먼저 위의
브라우저 검사를 실행해 생성한다. 최소 글꼴 회귀는 검토 기록에 명시한 파일 시스템 격리에서 실행하며,
시각 비교용 상용 글꼴을 공급한 결과와 구분한다.
