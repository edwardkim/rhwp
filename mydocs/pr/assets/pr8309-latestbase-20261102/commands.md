# 실제 실행과 재현

제품 source·실행 파일·입력·글꼴 해시는 `producer.json`과 `inputs.json`을 따른다.
기본 작업공간은 `/home/edward/mygithub/rhwp`, 검토 worktree는
`/home/edward/mygithub/rhwp-worktrees/pr-8309-textbox-vertical-align`이었다.
모든 Cargo 실행은 기본 작업공간의 `target/pr-review`를 순차 재사용했다.

## API 저장 입력

```bash
node mydocs/pr/assets/pr8309-latestbase-20261102/reproduce.mjs \
  /home/edward/mygithub/rhwp-worktrees/pr-8309-textbox-vertical-align/pkg \
  /home/edward/mygithub/rhwp/output/pr-review/pr8309-latest/reproduce-new
```

출력 폴더는 새 폴더로 지정한다. 기존 파일을 덮어쓰지 않는다. 같은 명령의 실제 실행으로
fixture 8개가 보존본과 바이트 단위로 같음을 확인했다(`reproduction.json`). 수정 전 package로
실행하면 `preserved=false`가 나오고 exit 1이다. 이는 재현 결과이며 코드 검증 통과가 아니다.

## Native와 fresh WASM

Native base와 후보는 각각 해당 runtime source로 다음 명령을 실행하고 실행 파일을 별도로
보존했다. Python bubblewrap wrapper로 공개 fallback 5개만 노출한 회귀 환경에서 실행했다.

```bash
export CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review
export CARGO_BUILD_JOBS=14
cargo build --locked --profile release-test --features native-skia --lib --bin rhwp \
  --config profile.release-test.package.rhwp.opt-level=1 \
  --config profile.release-test.package.rhwp.codegen-units=64
```

```bash
CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review \
  CARGO_BUILD_JOBS=14 \
  bash scripts/wasm-pack-locked.sh --target web --out-dir pkg --release --mode no-install
```

wasm-pack은 release build 후 실제 wasm-opt 117 최적화를 완료했다. 기본 작업공간의 사용자
Studio package를 후보로 덮어쓰지 않았다. 초기 화면에서 소비한 실제 WASM 바이트 해시는
`studio-roundtrip.json`에 있다. 새 binding의 JS와 d.ts 해시는 base와 같고 WASM 구현만 바뀐다.

## 시각 비교

실제 새 입력 비교의 한 예다. 실행 파일·package·출력 폴더를 base/후보와 Native/WASM에 맞게
분리했다. Native 명령은 `--wasm-pkg`만 제외한다. 기존 #8122는 README에 기록한 글꼴 두 개와
`--compat 2022`를 명시했다.

```bash
VISUAL_SWEEP_CHROME=/home/edward/.cache/puppeteer/chrome/linux-155.0.8059.79/chrome-linux64/chrome \
  /home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py \
  --rhwp-bin /home/edward/mygithub/rhwp/output/pr-review/pr8309-latest/candidate-rhwp \
  --wasm-pkg /home/edward/mygithub/rhwp-worktrees/pr-8309-textbox-vertical-align/pkg \
  --embed-fonts full \
  --font-path /home/edward/mygithub/rhwp/output/pr-review/pr8309-latest/fonts/HANBatang.ttf \
  --compat 2024 --dpi 96 --page-workers 1 \
  --file-target saved_default \
    tests/fixtures/pr8309-latest/textbox-saved-default.hwp \
    pdf/pr8309-latest/textbox-saved-default-2024.pdf \
  --out output/pr-review/pr8309-latest/review-new
```

base는 `--silhouette-only`로 직접 출력 TSV를 생성했다. 후보의 일반 비교 PNG에서는 다음
canonical 명령으로 같은 화소를 재측정했다. `not_evaluated` 상태 자체를 승인으로 쓰지 않고,
쪽수·내용·직접 확대 판독을 별도로 수행했다.

```bash
/home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py \
  --silhouette-only --page-workers 1 --key saved_default \
  --png-pair output/pr-review/pr8309-latest/review-new/saved_default/rhwp_png \
    output/pr-review/pr8309-latest/review-new/saved_default/pdf_png \
  --out output/pr-review/pr8309-latest/review-new/saved_default/tsv
```

독립 Print는 프로젝트의 한컴 CLI `start → status → download` 순서로 실행했다. 실제 job,
engine·PrintMethod·출력 방식·입력/PDF 해시는 두 Print execution JSON에 보존했다. 인증 파일과
endpoint·capability는 공개하지 않는다.

## 로컬 필수 gate

`rust-gates.json`은 최소 글꼴 격리에서 순차 실행한 각 명령·종료 코드·시간을 기록한다.
release-test 계열은 위 root package opt-level=1·codegen-units=64 override를 동일하게 썼다.
전체 integration은 `cargo test --tests`로 대체하지 않았다.

- manifest prepare·base SHA 지정 check, fmt check.
- Native Clippy, WASM32 Clippy, workspace build, workspace all-target Clippy.
- `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast`.
- Native Skia 전체 `--lib`, `issue_2225_missing_picture_placeholder`, `render_p37_direct_pdf_export`.
- 원 정렬 회귀, #8172 storage bit, #8225 grouped CTRL_DATA, #8122 no-LineSeg 정렬,
  #8174 DocInfo·public Storage boundary, 새 base의 #8335 번호 범주·#8399 어울림·#8302 lineage 집중 검사.

개별 integration은 `scripts/run-rust-test.mjs`의 기존 suite 라우팅을 사용했다. 파생 suite와
manifest를 source commit에 추가하지 않았다. `studio-gates.json`은 TypeScript·전체 npm 검사·
Studio build 실행이며, 별도 실제 브라우저 저장·재열기는 `studio-roundtrip.json`이다.
