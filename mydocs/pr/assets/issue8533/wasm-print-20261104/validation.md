# 최종 로컬 검증

2026-11-04 KST. 제품·검사 후보 `6aa617c207`, 정책 base
`1924d371d6757392d063b8dc2e9bbe9a298be817`.
최종 fetch에서도 upstream/devel은 이 base와 같았다. 본인 계정의 열린 PR은 없었다.
이 항목은 최초 로컬 검증 완료 시점의 기록이며, 당시 원격 push·PR 생성·CI·merge는
수행 전이었다. 이후 #8583 CI에서 확인한 시험 글꼴 고지 누락과 보완은
`task_m100_8533_stage11.md`에 따로 기록했고 아래에 추가 패키징 결과를 반영했다.

## 실행 결과

| 범위 | 명령·실제 결과 |
| --- | --- |
| 전체 Rust 회귀 | `cargo nextest run --locked --workspace --cargo-profile release-test --tests --test-threads 8 --no-fail-fast`: 12,537 PASS / 0 FAIL / 45 skip, 테스트 615.378초 |
| focused 원본 프로그램 | `node scripts/run-rust-test.mjs issue_7445_original_font_program_ownership -- --cargo-profile release-test --test-threads 8`: 19 PASS |
| focused HFT 계약 | 같은 helper의 `hft_outline_font_contract`: 8 PASS |
| focused Canvas session | 같은 helper의 `issue_7084_canvas_metric_session`: 11 PASS |
| Rust lint | `cargo fmt --all -- --check`; Native Clippy; `-p rhwp --lib --target wasm32-unknown-unknown` Clippy; `--workspace --all-targets` Clippy: 모두 `-D warnings` 통과 |
| workspace | `cargo build --locked --workspace`: PASS |
| manifest·unit tier | 각각 `--check --base-ref 1924d371d6757392d063b8dc2e9bbe9a298be817`: PASS |
| Native Skia lib | `cargo test --locked --profile release-test --features native-skia --lib -- --test-threads 8`: root 3,857 PASS / 12 ignore, 나머지 libs 182 PASS |
| Native Skia PNG | `issue_2225_missing_picture_placeholder` focused `--features native-skia`: 2 PASS |
| Native Skia PDF | `render_p37_direct_pdf_export` focused `--features native-skia`: 22 PASS |
| fresh WASM | `CARGO_BUILD_JOBS=8 CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`: exit 0, 4분48초. Docker 최적화 빌드가 아님 |
| Studio unit | `node --test --test-concurrency=8 tests/*.test.ts ../npm/editor/tests/*.test.mjs ../rhwp-shared/pdf/*.test.js`: 2,718 PASS / 0 FAIL / 2 skip |
| TypeScript | `./node_modules/.bin/tsc --noEmit` (Studio cwd): exit 0 |
| 실제 WASM browser | 원본 프로그램·Print 기하·JPEG 프레임·문맥 경계 묶음 13 PASS / 0 FAIL |
| Print bridge | `node --test --experimental-transform-types rhwp-studio/tests/issue-8533-print-metric-context.test.ts`: 1 PASS. 실제 전환·오류 후 복원 검사 |
| Visual Sweep Python | `PYTHONPATH=scripts:scripts/tests venv/bin/python -m unittest discover -s scripts/tests -p 'test_visual_sweep*.py'`: 245 PASS |
| 시험 글꼴 고지·패키징 | `repositoryOnlyFonts` 등록 뒤 Studio·Chrome·Firefox build와 VS Code compile exit 0, `node --test scripts/frontend-font-assets.test.mjs`: 8 PASS / 0 FAIL |

Cargo는 `CARGO_TARGET_DIR=target/pr-review`, jobs8·threads8로 순차 실행했다. 전체 nextest는
현재 Native Rust 소스를 실제 컴파일한 결과다. 이후 보완한 Rust 파일은 WASM 전용
`web_canvas/hft_outline.rs`뿐이며 Native compiler 의존 목록에 없다. 그 경계는 새 WASM으로
재빌드해 실제 브라우저·전쪽 시각을 다시 검사했다. `dea51e0bec` 이후 현재 후보까지
Rust/Cargo 제품 소스 diff는 없고, 이후 Python 보완은 해당 245건·옵션 단독 전쪽으로 검증했다.
클릭 코드 철회 후 Studio unit·TypeScript·Print bridge도 새로 실행했다.

## 전쪽 시각 실행

Native·최소 글꼴 fresh WASM·일반 fresh WASM은 각각 383쪽을 새로 산출했다.
최저 raw는 각각 99.96465%, 99.96403%, 99.96401%·130쪽이며 #8389 하한 하락은 0쪽이다.
입력·PDF·profile·도구·실행 파일·JS/WASM·글꼴 프로그램 해시는
[시각 생산 신원](visual-validation.json), 전체 수치는 [TSV 목록](README.md#전쪽-결과)에 있다.
대표 쪽의 raw·review·overlay를 직접 판독했으며, 130쪽 기존 미세 차이는 남긴다.
raw 100%와 RGB 완전 동일을 구분한다. CanvasKit 출력 성공으로 주장하지 않는다.

추가로 최소 글꼴 격리 안에서 외부 글꼴 경로·환경을 주입하지 않은 옵션 단독 명령을 실행했다.

```bash
/usr/bin/sandbox-exec -f output/pr-review/issue8533-wasm-advance-20261104/public-fonts.sb \
  venv/bin/python scripts/visual_sweep.py --handbook-gate \
  --rhwp-bin output/pr-review/issue8533-wasm-advance-20261104/rhwp-final-native \
  --page-workers 8 --out output/pr-review/issue8533-wasm-advance-20261104/option-only-native-v2
```

자동 공급·Native 383쪽·쪽별 하한 판정이 207.3초·exit 0으로 완료됐다. 명시 공급 Native와
383쪽 raw PNG 해시가 전부 같았다. `--handbook-gate` 없이 일반 실행에 편람을 추가하지 않는
기존 옵션 계약도 Python 검사에서 유지했다.

## 증거와 범위

- 전체 run/게이트 정본: ignored `output/pr-review/issue8533-wasm-advance-20261104/` 아래
  `final-gates-completion.json`, `final-gates-driver.log`, `final-full-nextest.log`.
- Python 반례: `option-readable-font-before-valid.log` 실제 2 FAIL →
  `option-readable-font-after.log` 29 PASS → `final-sweep-python.log` 245 PASS.
  최초 import/class 호출 오류는 결함 검출 근거가 아니다.
- 실제 옵션 최초 실패는 `option-only-native/required-handbook-8153-gate.json`, 보정 후
  완료는 `option-only-native-v2/required-handbook-8153-gate.json`으로 구분한다.
- Studio 실제 HTTP 응답 JS·WASM은 root pkg·Studio public과 바이트 해시가 같았다.
  직접 PNG Print 소비와 Screen/Print 전환의 정상·오류 경계를 따로 확인했다.
- 3·4쪽 클릭·Esc 편집은 사용자 지시로 별도 [#8578](https://github.com/edwardkim/rhwp/issues/8578)이며
  이 PR에서 해결했다고 표시하지 않는다. 원래 미완성 mouse 변경은 base와 같게 철회했다.
- workflow·CI 정책·품질 문턱·원본·PDF·기준 TSV는 수정하지 않았다. 로그·중간 driver는
  ignored output에만 보존하고 최종 TSV·대표 이미지·검증 정본만 제출한다.
