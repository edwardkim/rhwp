# planet6897 열린 PR 통합 검토 - Stage 3

## 목적

통합 브랜치가 각 원 PR의 본문과 후속 코멘트에서 약속한 범위를 유지하는지, 전체
회귀·WASM·브라우저·Windows 검증으로 최종 확인한다. 이 단계에서는 PR review 문서,
오늘할일, 원격 push를 만들지 않는다.

## 검증 범위

1. `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`와
   `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`를 수행한다.
2. `rhwp-studio` WASM build와 #2664 TypeScript 단위 테스트를 수행한다.
3. #2671의 `export-svg --embed-fonts` 산출물을 Blink 기반 브라우저에서 직접
   렌더링해 BinData 글꼴 data URI가 실제로 사용 가능한지 확인한다.
4. `win10-ted`의 기본 SSH 셸, cmd, PowerShell에서 한양 4종 identity 아티팩트
   검증 스크립트를 수행하고, 필요한 개발 도구 설치 내역을 기록한다.
5. #2663/#2665/#2669 visual sweep의 잔여와 원 PR 범위를 구분해 최종 리뷰의
   merge 판단 근거로 정리한다.

## 완료 기준

- 전체 Rust 회귀와 clippy, WASM build, Node 단위 테스트가 성공한다.
- 브라우저 SVG 렌더에서 임베디드 글꼴이 두부 문자나 로컬 폴백 없이 표시된다.
- Windows 3개 셸 검증의 결과가 일치한다.
- 잔여 조판 fidelity가 구현 범위를 넘어서는 경우에도, 원 PR 본문에 이미 명시된
  비목표인지 구분해 merge 보류 여부를 판단할 수 있다.

## 결과

### macOS 전체 검증

- 기존 `target`을 지우지 않고 `target/pr2706-validation`을 새로 사용해 캐시 영향을
  분리했다.
- `CARGO_TARGET_DIR=target/pr2706-validation CARGO_INCREMENTAL=0 cargo test
  --profile release-test --tests`가 cold build 포함 exit 0으로 완료됐다.
- `CARGO_TARGET_DIR=target/pr2706-validation CARGO_INCREMENTAL=0 cargo clippy
  --all-targets -- -D warnings`가 성공했다.
- `rhwp-studio`에서 `wasm-pack build --target web --out-dir pkg`가 성공했고,
  `node --test tests/flow-image-clip.test.ts`도 5/5 통과했다.

### SVG 임베디드 글꼴 산출물

- `target/debug/rhwp export-svg samples/render-p35-font-native-bitmap.hwpx
  --embed-fonts -o target/planet6897-svg-font-20260721 -p 0`가 1쪽 SVG를 생성했다.
- 산출 SVG의 `RHWP Bitmap SVG Glyph Smoke` 정확한 `@font-face` 규칙은
  `data:font/ttf;base64,...`를 사용하고 `local(...)` fallback을 포함하지 않는다.
- 인앱 브라우저는 보안 정책상 로컬 `file://` SVG 접근을 차단했다. 정책 우회는 하지
  않았으므로 실제 Blink 화면 렌더는 이 환경에서 검증하지 못했고, native 회귀 테스트와
  생성 SVG 검사를 근거로 남긴다.

### Windows 검증

- `win10-ted`에 사용자 범위 Python 3.12.10을 설치했다.
- 최신 `upstream/devel`에서 `tools/task2430/gen_metrics.py --verify`를 기본 SSH 셸,
  `cmd`, PowerShell 각각 실행해 한양 4종 identity 결과가 모두 `95/95 exact match`로
  일치함을 확인했다.

### 검토 범위 재확인

- Stage 3 도중 planet6897의 새 PR #2714가 열렸다. 기존 8건의 검증 결과를 이
  커밋으로 고정한 뒤, 새 PR은 다음 단계에서 따로 체리픽·검토한다.
