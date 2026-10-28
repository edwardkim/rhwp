# Issue #7909 결과보고서

Issue: https://github.com/edwardkim/rhwp/issues/7909

## 범위와 원인

최신 upstream/devel `def947c2149df5bc812d801e6e79be19a403b4d1`에서 독립
`codex/issue-7909-display-text` branch를 시작했다. 조사 기준 SHA와 동일함을 fetch로 재확인했다.
연결 계정과 이슈 작성자는 postmelee이며, 중복 열린 PR은 검색에서 발견되지 않았다.
검증한 코드 후보는 `fa4a0926594ec11fc65720044b85b116a2062255`다.
이후 제출 기록 commit은 문서만 추가하며 Rust/test/fixture 입력은 이 후보와 같다.

기존 renderer는 모델 text와 표시 display_text를 분리하지만 getPageTextLayout export는
모델 문자열만 내보냈다. 공통 JSON export에 optional displayText/displayCharX를 추가했다.
기존 text/charX/charStart, 문서·셀 식별자와 bbox는 보존했다. 표시 경계는 같은 renderer의
replay_positions_for(display)를 소비한다. 선택·커서·편집의 모델 오프셋은 바꾸지 않는다.
parser/model/serializer/조판/paint 코드를 수정하지 않았으며 #7910 구현을 포함하지 않는다.

## 입력과 수정 전후 재현

공개 #7472 ZIP SHA-256:
`1c4415a9b550fc865d79a832424510e3d656d21ccbafc0856b1961c02ab25918`.
원본 두 HWP의 bytes는 사용자 첨부와 일치한다. fixture README에 파일별 해시를 보존했다.
첨부 control_footer_nopicture.hwp는 footer controls=0이며 raw atno 레코드도 없다.
이를 번호 없는 음성 대조로 보존했다. 원본의 도형·그림만 제거하고 atno를 유지한 별도
HWPX/HWP 생성본을 양성 대조로 추가했으며 한컴 원본으로 취급하지 않는다.
생성 코드는 tests/cases/issue_7909_text_layout_display.rs에 있다.

- 최종 7개 소스 + 수정 전 base export: **4 FAIL / 3 PASS**.
- 수정 후 같은 7개 검사: **7 PASS / 0 FAIL**. 전체 회귀 안에서도 7개를 다시 모두 실행했다.
- 원본 69런 / 첨부 대조본 70런: 새 두 키를 제외한 전체 JSON이 수정 전후 동일하다.
- 원본 쪽번호의 표시 경계 X≈369.5는 기존 SVG X≈369.54와 대조했다.
  API y≈1039.6은 bbox 상단이며 SVG baseline≈1049.84와 구분한다.
  절대 좌표·SVG 해시 golden은 추가하지 않았다.
- 일반 문구·자동번호 없는 문단·HWP/HWPX·PAGE/TOTAL_PAGE·파일명 확장·escaping을 검사했다.
  selection/cursor 응답은 export 전후 같고 모델 오프셋 1의 삽입 결과는 marker+XAB다.

초기 첨부 no-picture에 번호를 기대한 검사는 잘못된 기대값이었다. raw CTRL_HEADER와
parser 결과를 대조해 교정했고, 번호 보존 양성 변형을 별도로 추가했다.
교정 전 실패를 결함 검출 증거로 세지 않았으며 최종 7개 소스로 base를 다시 실행했다.

## 환경과 실행 명령·결과

별도 cloud session, Debian 13 x86_64, Rust/Cargo 1.93.1, Node v24.19.0,
wasm-pack 0.15.0, cargo-nextest 0.9.149. 논리 CPU 5개, Cargo jobs 4개,
전체 nextest test threads 4개. source checkout과 detached review worktree를 분리했다.
모든 Cargo target은 공용 /workspace/rhwp/target/pr-review를 사용했다.
아래 상대 경로 명령은 review worktree /workspace/rhwp에서 실행했다.

| 명령 | 실제 결과 |
| --- | --- |
| node scripts/rust-test-suite-manifest.mjs --prepare | 파생 suite 준비, 생성물 미커밋 |
| node scripts/run-rust-test.mjs issue_7909_text_layout_display -- --cargo-profile release-test --target-dir target/pr-review --no-fail-fast | base 4 FAIL/3 PASS → 후보 7 PASS |
| cargo fmt --all; cargo fmt --all -- --check | PASS |
| cargo clippy --locked --target-dir target/pr-review -- -D warnings | PASS |
| cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings | PASS |
| cargo build --locked --workspace --target-dir target/pr-review | PASS |
| cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings | PASS |
| node scripts/rust-test-suite-manifest.mjs --check --base-ref def947c2149df5bc812d801e6e79be19a403b4d1 | 1687 sources / 7407 attrs / 48 targets, PASS |
| node scripts/rust-unit-test-tiers.mjs --check --base-ref def947c2149df5bc812d801e6e79be19a403b4d1 | 4101 static tests / 296 modules, PASS |
| cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 4 --no-fail-fast | 11,369 PASS / 0 FAIL, 기존 제외 45건; slow 20건도 통과 |
| cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib | 4041 PASS / 0 FAIL / 기존 ignore 12건 |
| node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/pr-review --features native-skia --no-fail-fast | 2 PASS |
| node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/pr-review --features native-skia --no-fail-fast | 14 PASS |
| CARGO_TARGET_DIR=/workspace/rhwp/target/pr-review CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 scripts/wasm-pack-locked.sh --target web --out-dir pkg | 최적화 포함 fresh build PASS |
| node tests/fixtures/issue7909/verify-wasm.mjs pkg | 실제 WASM의 원본·음성 대조·번호 보존 HWP/HWPX 및 재저장·모델 편집 PASS |
| node work/issue-7909/browser.mjs | 실제 Chromium에서 동일 fresh package의 4개 파일/API, SVG 조회 전후 불변 PASS |
| git diff --check def947c2149df5bc812d801e6e79be19a403b4d1...HEAD; git diff --check | PASS |

Node manifest 배정 계약은 sandbox 밖에서 23/23 PASS였다. sandbox 안의 자식 프로세스 실패를
공식 통과로 세지 않고 정상 실행 경로에서 다시 실행했다.
변경 문서의 상대 링크·메타데이터 오류는 0건이다. 전체 문서 metadata 검사는 기존 5개 파일의
누락 20건으로 FAIL이며, 해당 파일 bytes는 base와 동일함을 확인했다. 전체 성공으로 기록하지 않는다.

### Native Skia의 초기 링크 실패와 재검증

최초 실행은 freetype/fontconfig의 개발용 링크 이름이 없어 검사 진입 전에 실패했다.
실패 로그를 보존했다. 시스템 apt 설치는 계정 권한으로 거부됐다.
실제 런타임 SONAME 두 개가 이미 설치돼 있음을 ldconfig로 확인하고 작업용 native-libs에
libfreetype.so → /lib/x86_64-linux-gnu/libfreetype.so.6,
libfontconfig.so → /lib/x86_64-linux-gnu/libfontconfig.so.1 링크를 만들었다.
LIBRARY_PATH=/workspace/rhwp/work/issue-7909/native-libs를 전달해 동일 명령을 재실행했다.
소스·기대값·허용치·검사 선택을 변경하지 않았고 실제 라이브러리를 사용했다.
위 Skia 통과 건수는 이 보정 뒤 실제 실행 결과다.

### WASM 도구 다운로드 실패와 재실행

첫 fresh WASM의 Rust 컴파일은 8분 15초로 성공했지만 wasm-opt 자동 다운로드가 실패했다.
최적화를 생략하지 않고 공식 Binaryen version_117 archive를 curl로 설치했다.
archive SHA-256: `3dc677006555b355ea2da5e82602065a161d5e83eaefd3f759afa00b96e83212`.
wasm-opt version 117을 확인하고 PATH로 전달해 같은 build를 재실행했다.
동일 소스의 Rust 산출물은 재사용했고 실제 wasm-opt -O를 끝까지 실행했다.
첫 다운로드 실패 로그도 보존했다. 위 fresh build PASS는 재실행 종료 결과다.

브라우저 검사의 첫 실행은 없는 puppeteer 모듈 경로로 검사 진입 전에 실패했다.
설치된 Playwright로 도구 경로를 수정해 같은 4개 문서/API를 재실행했다.
실제 Chromium 151.0.7922.173에서 PASS했으며 첫 실패 로그도 보존했다.

### fresh WASM 산출물

- pkg/rhwp.js SHA-256: `4942d1b79e8405fe962b4e46222adafb43ba4598af73e31e299b42f9fb99d518`
- pkg/rhwp_bg.wasm SHA-256: `ed6588e45ce28dc493aaf941513bedb5ce77ad41e3808401ed353874893d53e8`
- 루트 pkg와 rhwp-studio/public의 JS/WASM bytes가 각각 동일함을 확인했다.
- 실제 API에서 새 display 필드와 marker 모델 보존을 확인해 stale bundle을 배제했다.

## 미검증 범위와 사용자 직접 확인

실제 다운스트림 앱의 표시 인덱스와 편집 오프셋 관계는 사용자 검토가 남는다.
비공개 HWPX 원본, HWP3 자동번호 별도 focused 검사, 한컴 Print PDF와의 시각 일치는
검증했다고 주장하지 않는다. #7910의 배치 개선·merge·release·운영 배포·이슈 종료는 수행하지 않는다.

1. PR branch의 fresh WASM에서 공개 원본과 footer-no-picture-keep-auto.hwp/.hwpx를 연다.
2. JSON.parse(doc.getPageTextLayout(0))에서 displayText ?? text로 번호 1을 확인하고,
   displayCharX ?? charX와 런 x/y로 표시 위치를 확인한다.
3. 첨부 control_footer_nopicture.hwp에는 번호가 없으므로 일반 문구 유지만 확인한다.
4. 긴 파일명 필드처럼 모델 1자가 여러 표시 글자로 보일 때, 기존 hit-test/selection/cursor API의
   모델 오프셋을 사용해 필드 뒤에 글자를 삽입한다. 표시 문자열 인덱스를 charStart에 더하지 않는다.

상세 계약은 mydocs/tech/text_layout_display_contract.md, 입력 provenance는
 tests/fixtures/issue7909/README.md, 단계별 기록은 task_m100_7909_stage1.md와 stage2.md를 따른다.
PR URL과 최종 제출 head는 PR 본문 및 작업 반환 결과에 기록한다.
