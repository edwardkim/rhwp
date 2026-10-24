# Issue #7909: 최종 검증과 제출

## 분석과 검증 범위

코드 후보 `fa4a0926594ec11fc65720044b85b116a2062255`, base
`def947c2149df5bc812d801e6e79be19a403b4d1`.
source checkout은 `/workspace/rhwp-source-7909`, detached review worktree는 `/workspace/rhwp`다.
공용 Cargo target은 `/workspace/rhwp/target/pr-review`로 유지한다.

최종 7개 검사 소스에 대해 base의 export 함수로 다시 실패를 재현하고,
후보의 같은 함수로 복원한 뒤 lint·focused/전체 회귀·fresh WASM을 순차 실행한다.
표시 export는 읽기 전용이며 parser/model/serializer/renderer의 생산 규칙을 수정하지 않는다.
모델 필드 보존을 전체 JSON 비교로 확인하고 실제 WASM에서도 표시 문자열/모델 오프셋을 구분한다.
공개 no-picture에는 atno가 없다는 원시 레코드 증거를 정상 대조에 반영했다.
별도 변형은 한컴 생성 원본과 구분하고 atno 보존을 파싱 후 다시 확인한다.

필수 게이트가 실패하거나 미실행이면 push/PR을 하지 않는다. 결과는 실행 후 아래에 기록한다.

## 확정된 결과

- 최종 7개 소스 + base export: 7건 실행, 4 FAIL / 3 PASS.
  표시 문자열 누락을 검사한 네 양성 사례가 실패하고 세 정상 대조는 통과했다.
- 후보 focused: 7건 실행, 7 PASS / 0 FAIL. focused filter에서 제외한 suite 검사 255건은
  통과 건수에 포함하지 않는다.
- fmt, native Clippy, WASM32 Clippy, workspace build, workspace/all-targets Clippy: 모두 PASS.
- manifest base 비교: 1687 sources / 7407 static test attrs / 28 suites + 20 exceptions,
  48/48 integration targets PASS.
- unit tier base 비교: 4101 tests / 296 modules, 정책 PASS.
- 문서 메타데이터 전체 검사 20건은 5개 파일이 base와 byte-identical임을 확인했다.
  이번에 추가·수정한 장기 문서의 메타데이터 오류는 0건, 상대 링크 오류도 0건이다.
- Git push dry-run은 성공했으며 원격 ref는 생성하지 않았다.

전체 release-test와 Native Skia의 결과는 아래와 같다.

전체 `cargo nextest run --locked --cargo-profile release-test --target-dir
/workspace/rhwp/target/pr-review --tests --test-threads 4 --no-fail-fast`:
**11,369 PASS / 0 FAIL / 기존 제외 45건**, 장시간 검사 20건도 모두 통과했다.
빌드 14분 06초, 실제 검사 1596.609초였다. 제외 45건은 이번 diff에서 추가하지 않았으며,
focused 새 7건도 전체 회귀 안에서 다시 모두 실행·통과했다.

## Native Skia 환경 의존성 보정

첫 `--features native-skia --lib` 실행은 검사 진입 전 링크에서 실패했다:
`unable to find library -lfreetype`, `-lfontconfig`. 실패 로그를 보존했다.
시스템 패키지 설치는 비관리자 계정 권한으로 거부됐다. `ldconfig -p`로 실제 런타임
`/lib/x86_64-linux-gnu/libfreetype.so.6`, `libfontconfig.so.1`이 이미 설치됐음을 확인했다.
개발용 unversioned `.so` 링크만 없으므로 작업 디렉터리 `work/issue-7909/native-libs/`에
각 SONAME을 가리키는 링크를 만들고 `LIBRARY_PATH`로 전달했다. 실제 라이브러리를 대체하거나
소스·검사·기준값을 바꾸지 않았다. 같은 두 라이브러리의 C 링크 probe는 통과했다.
Native Skia의 동일 검증 명령을 이 환경에서 재실행했으며, 첫 실패를 skip으로 처리하지 않았다.

Native Skia lib 재실행은 PASS다. root 3,859건, 내부 crate 15+165+2건으로
합계 **4,041 PASS / 0 FAIL / 기존 ignore 12건**이다. 시스템 라이브러리의 실제 SONAME을
연결한 뒤 동일 명령으로 실행했다. 누락 그림 회귀 2건과 직접 PDF 회귀 14건도 모두 PASS다.

## WASM 도구 의존성

최초 fresh WASM Rust 컴파일은 8분 15초로 성공했지만 wasm-opt 자동 다운로드가 실패했다.
최적화를 끄지 않고 동일한 공식 Binaryen version_117 archive를 curl로 내려받아 설치했다.
archive SHA-256: `3dc677006555b355ea2da5e82602065a161d5e83eaefd3f759afa00b96e83212`.
`wasm-opt --version`으로 version 117을 확인하고 PATH에 추가한 뒤 동일 build를 재실행했다.
첫 도구 다운로드 실패 로그도 보존했다. Rust 재컴파일은 동일 소스의 캐시를 재사용한다.

## 최종 WASM/API 결과

- 공식 wasm-opt 117의 `-O` 최적화를 포함해 fresh build PASS(재실행 6분 59초).
- `node tests/fixtures/issue7909/verify-wasm.mjs pkg`: 4개 입력과 각 HWPX 재저장,
  파일명 표시 확장·escaping·selection/cursor/hit/edit 모델 오프셋 PASS.
- 실제 Chromium 151.0.7922.173: 4개 입력의 번호 유무·문구·표시 경계·조회 전후 SVG 불변 PASS.
  첫 브라우저 스크립트는 없는 puppeteer 모듈 경로로 검사 시작 전에 실패했다.
  설치된 Playwright로 실행 경로를 바로잡아 동일 문서/API 검사를 재실행했다.
- pkg와 rhwp-studio/public의 JS/WASM bytes 일치. 해시는 결과보고서에 기록했다.
- `git diff --check base...HEAD`와 review worktree의 src/tests 변경 없음 검사 PASS.

필수 코드 검증 게이트의 최종 실행은 모두 PASS다. 초기 실패 로그를 보존했으며
검사를 skip/ignore하거나 최적화를 비활성화하지 않았다. 기존 metadata 오류 20건은 별도 FAIL이다.
최종 기록 commit은 문서만 추가한다. 실제 API 소비자의 표시/편집 인덱스 사용 검토는 남는다.
