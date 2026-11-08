# #7875 2단계 — feature 통합 보정과 검증

Issue: [#7875](https://github.com/edwardkim/rhwp/issues/7875)

원 PR: [#7876](https://github.com/edwardkim/rhwp/pull/7876)

## 검토 결론과 범위

**메인터너 보정 후 수용 가능 — #7875 해결 범위의 로컬 검증을 완료했다. 통합 PR #7981의 code candidate CI도 성공했다. 문서 trailing CI·merge는 별도 후속 게이트다.**
[1단계](task_m100_7875_stage1.md)에서 프로젝트 적합성과 최신 devel의 실제 시작/export 문제를 먼저 재현한 뒤 적용했다.
기존 git/path 라이브러리 소비의 범용 호환 개선에 해당한다. #5446의 신규 배포 채널 동결과 외부 Rust API 안정성 미확정 범위를 유지한다.

| 역할 | SHA |
| --- | --- |
| 기준 devel | `515bd72dcde0eecdeece333666b675354b7fa91d` |
| 원 contributor head | `b273e3fc44dd713f33d22846faa6fb1d98f01d3b` |
| 체리픽, 원 author/co-author와 `-x` 보존 | `1869234ed8c2e98138677bee2c76baeb64ed5c23` |
| 설명 보정·최종 기능 검증 코드 | `366654f29959c62088bee7206d77b8889e3ebcd4` |

branch는 `review/9ood2s-7876-20261017`이며 contributor fork는 변경하지 않았다.
원 PR의 과거 CI는 위 통합 코드의 CI로 재사용하지 않았다. 원 PR은 현재 devel의 Cargo 기본 feature와 충돌하므로 직접 merge 대상이 아니다.

## 원 기여와 메인터너 보정

- 원 기여: 기본으로 켜진 `wasm-api`가 facade와 JS export/start 속성을 선택한다. CLI는 이 facade를 사용하므로 `required-features`를 선언했다.
- Cargo 충돌: 기존 `pdf-import`와 그 의존성을 보존했다. 기본 feature는 `console_error_panic_hook`, `pdf-import`, `wasm-api`다.
- 설명: 일반 Rust main 자동 시작과 명시적인 wasm-bindgen start 연결을 구별했다. 여러 명시적 start는 함께 실행되며 순서는 보장되지 않는다.
- Rust API: `init_panic_hook`·`version`은 항상 Rust 함수로 남는다. 기본값을 끈 facade 소비자는 `wasm-api` opt-in이 필요하며 Cargo feature는 다른 의존성과 합쳐진다.
- 현재 프로젝트의 PDF 제외 안내: CLI는 `--no-default-features --features wasm-api`, Studio는 `--no-default-features --features console_error_panic_hook,wasm-api`를 사용한다. `--no-default-features` 단독은 라이브러리 모드이므로 Studio/CLI의 PDF만 제외하는 옵션으로 사용할 수 없다.
- 조판·parser/model·paint·font/resource·기준값은 변경하지 않았다. CI workflow와 기존 회귀 테스트도 변경하지 않았다.

## 구현 주장과 실행 증거

생산/소비 경로는 `Cargo feature 선택 → lib module·export/start 속성 → wasm-bindgen 생성/init → 소비 앱 main·JS API`다.
기본 모드는 같은 facade와 속성·함수 본문을 선택하며, 라이브러리 모드에서만 facade와 의존성의 panic-hook 자동 start/export를 제외한다.
Rust 함수 두 개의 직접 소비는 별도의 console-only WASM 소비 앱에서 타입 검사했다.

동일한 소비 앱 소스와 Git 입력으로 비교했다. 일반 main과 명시 app start는 소비 앱의 Cargo feature로 선택했다.
실제 rhwp resolve feature는 양쪽 모두 빈 배열이었다. console-only 소비 앱은 `console_error_panic_hook`만 활성화됐다.

| 검사 | 기준 devel | 보정 후보 |
| --- | --- | --- |
| 일반 main 실행 | false | true |
| 명시 app start 실행 | true | true |
| HwpDocument export | 492 | 0 |
| 일반 main 앱 WASM | 16,753,644 bytes | 2,580,226 bytes |
| 명시 app start 앱 WASM | 16,753,711 bytes | 2,580,269 bytes |
| HWP5/HWPX 파싱 | 각각 1 section·1 paragraph | 동일 |
| Chrome web-target | 시작 문제·export 잔류 재현 | Node와 같은 개선 확인 |

frozen 수정 전 Node 패키지를 같은 검사기로 다시 실행했다. 수정 전에는 main/불필요한 export 계약으로 FAIL, 수정 후에는 PASS였다.
초기 점검 스크립트의 CommonJS 변수명 오류는 제품의 수정 전 FAIL 증거로 세지 않았다.
여기서 측정한 크기는 공개 입력을 파싱하는 최소 앱에 한정한다. 기여자 사유 앱의 49.0→38.4MB 수치를 독립 재현했다고 쓰지 않는다.

## 기본 빌드 동등성

기준과 후보의 JS glue·두 타입 선언·package.json·README·LICENSE 6파일은 바이트 동일하다.
WASM import는 동일하며 export도 522개로 동일하다.

| WASM 산출물 | bytes | SHA-256 |
| --- | ---: | --- |
| 기준 devel 기본 빌드 | 22,305,631 | `ba855693573fea3dfc14b7ffaa637d616c0fbeebd5be8dc60e61e85fdbfad1a3` |
| 보정 후보 기본 빌드 | 22,305,358 | `8d334b3f0f4dc3bb9a64b00f67eefd7a723e07433b7971dfc489e21f5cb06600` |
| feature 이름만 추가한 기본 library 대조군 | 22,305,358 | `8d334b3f0f4dc3bb9a64b00f67eefd7a723e07433b7971dfc489e21f5cb06600` |

대조군은 후보 Cargo.toml을 유지하고 lib.rs만 기준 devel의 gate 없는 파일로 바꾼 working-tree variant다.
이 library 빌드에서 유효한 Cargo 차이는 새 feature의 등록·기본 활성화다. 같은 경로·compiler·profile·명령으로 빌드했다.
대조군과 후보의 패키지 7파일이 WASM까지 전부 바이트 동일했다. 기준과 후보 사이의 작은 바이너리 차이는 gate가 없는 이 대조군에서도 재현된다.
관측한 SHA는 아래 호스트·경로·도구 조건의 증적이며 다른 환경의 고정 golden으로 사용하지 않는다.

대조 빌드 뒤 원본 lib.rs를 정확한 바이트로 복원했다. 원본 SHA-256은
`c61ff228b693a3d47c15ef0ffa0b12922f2d4a05d4a2f7829e3dae1eba17623f`이며 Git clean도 확인했다.
기본 feature로 조건을 전개하고 일반 주석·빈 줄을 제외한 lib.rs도 기준과 동일했다.

추가로 Chrome에서 기존 HWP/HWPX 3문서의 전12쪽을 기준/후보 각각 실제 실행했다.
초기화·version·문서 열기·전체 쪽수·페이지별 렌더 트리·SVG가 동일했다. 이는 기본 동작 보존 검사이며 독립 Print와의 시각 일치 증거로 승격하지 않는다.

## PDF 기본·제외 모드 계약

| 소비 모드 | feature 선택 | 확인 범위 |
| --- | --- | --- |
| 기본 Studio/CLI | 기본값 | PDF 활성·전체 3쪽/선택 2쪽·원본/출력 대응, HWP/HWPX 유지 |
| 파서/모델 Rust 라이브러리 | `default-features=false` | 일반 main 실행, HwpDocument export 제외, HWP/HWPX 파싱 |
| 수동 panic hook Rust API | 위 모드 + `console_error_panic_hook` | Rust init_panic_hook/version 직접 참조의 WASM 타입 검사 |
| PDF 제외 Studio | `--no-default-features --features console_error_panic_hook,wasm-api` | PDF 비활성·constructor/importPdf의 PDF_IMPORT_DISABLED, HWP/HWPX 1쪽씩 유지 |
| PDF 제외 CLI | `--no-default-features --features wasm-api` | 기능 비활성 안내·help/capabilities·잘못된 인수·입력 보호 **4 PASS** |

PDF 브라우저 입력은 기존 `scripts/tests/pdf-import-wasm.test.mjs`의 합성 PDF 생성기다.
이 결과는 source-page 선택·기능/오류 전달 계약이며 한컴 PDF 출력이나 시각 정확성의 증거가 아니다.

## 검증 명령·결과

Rust 1.93.1·wasm-bindgen CLI 0.2.127·Chrome 155.0.8059.79에서 실행했다.
Cargo 작업은 jobs 8·공용 `/home/tsjang/rhwp/target/pr-review`에서 순차 실행했고, fresh WASM에는 `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`을 사용했다.
CARGO_INCREMENTAL=0, golden/baseline 갱신 옵션은 지정하지 않았다.

| 검사 | 결과 |
| --- | --- |
| suite prepare·fmt check·manifest check(base=515bd72dc) | PASS |
| 기본 Native·WASM Clippy, workspace build·all-target Clippy | PASS |
| Native no-default package / 명시적 API lib / WASM console-only lib | PASS |
| WASM no-default lib Clippy | PASS |
| Native no-default lib Clippy | 체리픽 1869234ed에서 PASS, 이후 일반 주석만 보정 |
| version 정확한 단위 검사 | 1 PASS |
| 전체 nextest | **10,808 PASS / 0 FAIL / 45 skipped**, 실행 683.980초 |
| Native Skia lib | **4,043 PASS / 0 FAIL / 12 ignored**(rhwp 3,861 + 내부 crate 182) |
| Native Skia placeholder | **2 PASS** |
| Native Skia 직접 PDF export | **14 PASS** |
| 라이브러리/기본/기본 대조군/PDF 제외 fresh WASM | 모두 build PASS |
| Node/Chrome 소비 앱, 기본 12쪽, PDF on/off 브라우저 | 모두 PASS |
| PDF 제외 CLI focused | **4 PASS**, 212 skipped |

재현 가능한 기본 명령은 다음과 같다. 전체 명령·UTC 일시·exit code는 실행 JSON과 로그에 기록했다.

```bash
export CARGO_BUILD_JOBS=8
export CARGO_TARGET_DIR=/home/tsjang/rhwp/target/pr-review
cargo nextest run --locked --cargo-profile release-test \
  --target-dir "$CARGO_TARGET_DIR" --tests --test-threads 12 --no-fail-fast
cargo test --locked --profile release-test --target-dir "$CARGO_TARGET_DIR" \
  --features native-skia --lib -- --test-threads 12
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- \
  --cargo-profile release-test --target-dir "$CARGO_TARGET_DIR" --features native-skia --test-threads 12
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- \
  --cargo-profile release-test --target-dir "$CARGO_TARGET_DIR" --features native-skia --test-threads 12
node scripts/run-rust-test.mjs issue_7911_pdf_cli -- \
  --cargo-profile release-test --target-dir "$CARGO_TARGET_DIR" --no-default-features --features wasm-api --test-threads 12
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 scripts/wasm-pack-locked.sh \
  --target web --out-dir output/pr-review/pr7876-20261017/browser-pdf-off-pkg \
  --no-opt -- --no-default-features --features console_error_panic_hook,wasm-api
```

Docker가 없는 호스트여서 wrapper `--no-opt` 진단 경로를 사용했다. 최적화한 Docker 배포 빌드를 실행했다고 쓰지 않는다.
Trunk 본체도 실행하지 않았으며 Cargo→wasm-bindgen Node/web→Node/Chrome을 직접 실행했다.
검증 출력은 별도 디렉터리를 사용했고 기존 tracked Studio public glue와 섞지 않았다.
첫 검사 구성의 출력 경로 보정으로 자체 native check를 중단한 1회는 실행 구성 오류로 구분했고 같은 검사는 재실행 PASS였다.

## 입력·조판 원칙·시각 범위

HWP5/HWPX 두 입력의 경로·bytes·SHA-256은 1단계 표에 고정했다. 추가 기본 모드 입력 `samples/hwpx/form-002.hwpx`도 실행 파일과 Git blob의 바이트 일치를 확인했다.
새 HWP/HWPX/PDF fixture·golden·px 회귀·baseline은 추가하거나 변경하지 않았다.

조판 원칙의 좌표/줄 높이/분할·이어받기 변경은 **비해당**이다. 생산/측정/배치/paint 값과 소비 조건을 바꾸지 않는다.
기본 선택 소스의 동일성·독립적인 feature-only 기본 library 빌드의 바이너리 동일성·기본 브라우저 12쪽 동등성과 기존 회귀를 근거로 실제 렌더링 변경이 없음을 확인했다.
따라서 독립 Print 기준의 Native/fresh WASM Visual Sweep·페이지 TSV는 이번 변경에서 비해당이다.
파싱 쪽수나 SVG 해시만으로 한컴 시각 일치를 판정한 것은 아니다.

## 기록·원격 단계

작업 중 raw 로그·JSON·frozen 패키지는 ignored `output/pr-review/pr7876-20261017/`에 있다.
전체 검사 전 output에는 pr-review만 있었음을 기록했다. 검사 종료 후 Cargo/Rust 및 열린 cwd/fd 참조가 없는 것을 확인하고 이번 검사로 생성된 중간 파일 178개(13,392,819 bytes)를 제거했다. 검증 증적과 tests/golden_svg는 보존했다.
원 PR의 첫 기여 author/co-author와 fork를 보존했다. reviewer는 jangster77이고 관련 이슈도 같은 계정에 할당했다.
원격 [통합 PR #7981](https://github.com/edwardkim/rhwp/pull/7981)을 생성했고 문서만 추가된 candidate `69f1b8e7ce2427fd642bec1b7a07a074f752264e`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/39030725781)·CodeQL·Render Diff·Adapter·Proptest·Controller와 36 check가 성공했다. 최종 [개별 review](../../pr/archives/pr_7876_review.md)와 [실행 기록](../../pr/archives/pr_7876_review_impl.md)을 같은 PR의 문서 trailing commit으로 추가한다. trailing CI·merge는 아직 실행 전이다.
정확한 통합 code head CI 성공을 확인한 뒤에만 원 PR #7876 기준 archive review/구현 기록과 오늘할일을 작성했다.
merge 후 첫 기여자에게 감사와 환영·실제 보정/검증 범위를 존댓말로 안내하고 fork branch는 보존한다.
