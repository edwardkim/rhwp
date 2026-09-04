# Task M100 #3930 Stage 6 - CharShape provenance 기반 선택적 canonicalization 분석

- 이슈: [#3930](https://github.com/edwardkim/rhwp/issues/3930)
- 브랜치: `fix/issue-3930-save-layout-input`
- 시작 기준: `c93e861f6` (`refactor(hwp): CharShape 직렬화 모듈 분리`)
- 기록일: 2026-08-05 KST
- 상태: 완료 (source-derived 선택 기준 없음, production serializer 변경 없음)

## 목표

Stage 5에서 전역 inactive sentinel canonicalization이 direct HWP raw 값 913건과 논리적으로
일치하면서도, 기준 HWPX PDF 대비 Stage 4의 7쪽/631픽셀 개선을 되돌리는 것을 확인했다. oracle
HWP의 raw 값을 runtime에 이식하지 않고도 안전하게 식별 가능한 HWPX provenance가 있는지 판정한다.

## 시작 상태

- Stage 4 fail-closed probe는 unique semantic key 733건만 치환해 7쪽을 단조 개선했다.
- Stage 5 전역 candidate는 937개 CharShape를 변경했고 기준 PDF 수치가 Stage 2 baseline과 같았다.
- direct HWP와 logical-normalized key 비교는 exact 913, unmatched 23, ambiguous 1이다.
- 현재 production serializer는 Stage 2와 HWP 전체 SHA-256까지 byte-identical이다.

## 범위

- HWPX header IR, HWP5 DocInfo `CHAR_SHAPE`, BodyText `PARA_CHAR_SHAPE` 참조를 함께 분석한다.
- Stage 4에서 바뀐 7쪽과 204개(23 unmatched + 181 ambiguity 해소 대상) style의 사용 위치를
  document/page/paragraph 수준으로 추적한다.
- direct HWP oracle이 없는 일반 사용자 HWPX에도 적용 가능한 source-derived 조건만 채택한다.
- HWP 2020 MCP server/client, PDF 전송, 기존 Stage 5 serializer 동치 코드는 수정하지 않는다.

## 수행 계획

1. 기존 raw semantic key보다 attr의 논리 상태를 보존하는 audit key를 정의하고, direct HWP와
   baseline의 exact/unmatched/ambiguous CharShape 목록을 stable ID와 함께 생성한다.
2. `PARA_CHAR_SHAPE` 참조와 section 문단을 따라 각 목록의 실제 사용 위치를 찾고, 7개 raster
   변경 페이지와의 교집합을 기록한다.
3. HWPX XML에서 해당 style의 명시/생략 속성, font/color/border-fill 참조를 비교해 source만으로
   안전하게 구별할 수 있는 provenance가 있는지 판단한다.
4. 조건이 문서별 direct oracle 또는 record ID 순서에 의존하면 production 변경을 하지 않고
   결론을 기록한다. source-derived 조건이 있으면 그 조건과 fail-closed fallback을 다음
   implementation stage에 계획한다.

## 성공 기준

- 7쪽의 Stage 4 개선이 어느 CharShape 사용과 연관되는지 재현 가능한 산출물로 남긴다.
- direct HWP 파일 자체나 고정 record ID를 runtime 입력으로 요구하지 않는다.
- 새 production serializer 변경은 source-derived 판별 기준과 focused test가 준비된 경우에만
  별도 Stage에서 시작한다.

## 테스트 결과

### 1. 진단 명령 구현 및 단위 검증

- 변경:
  - `src/diagnostics/hwp5_char_shape_audit.rs`를 추가했다.
  - `hwp5-char-shape-audit`는 Hancom HWP와 rhwp 생성 HWP의 `CHAR_SHAPE`를 비교해 raw semantic
    key와 inactive underline/strike/shadow를 정규화한 logical key를 별도로 분류한다.
  - 생성 HWP의 `PARA_CHAR_SHAPE`를 따라 각 style의 run 수, 문단 수, text sample, HWP5
    `PARA_LINE_SEG` 첫 줄 표식 기반 누적 쪽 추정도 기록한다. 이 누적 값은 한컴 PDF 쪽번호와
    같다고 가정하지 않는다.
  - 명령은 보고서만 쓰며 serializer, HWP 파일, MCP 호출을 변경하지 않는다.
- 명령: `cargo fmt && git diff --check && cargo test --lib diagnostics::hwp5_char_shape_audit -- --nocapture`
- 결과: exit code `0`. Rust formatting 및 diff whitespace 검사를 통과했고, inactive sentinel
  정규화, active decoration 보존, `PARA_CHAR_SHAPE` ID decode, `PARA_LINE_SEG` page flag decode
  단위 검증을 포함해 모듈이 빌드됐다.
- 판정: Stage 4 probe의 raw oracle 비교와 source-derived 후보 탐색을 혼동하지 않도록, 실제
  style 사용 위치를 먼저 재현 가능한 보고서로 고정할 기반을 마련했다.

### 2. 실제 HWP audit 실행 시도

- 명령: `cargo build --bin rhwp && target/debug/rhwp hwp5-char-shape-audit output/task3930-stage1/mcp/2025-행정업무운영-편람-mcp.hwp output/task3930-stage2/hancom-single-odd-picture/2025-행정업무운영-편람-rhwp-single-odd-picture.hwp --out output/task3930-stage6/char-shape-audit.md`
- 관측: Cargo가 기존 artifact directory file lock을 대기한 뒤 build를 마쳤고, `&&`로 연결한
  `char-shape-audit` 보고서가 생성됐다. 생성 확인 명령 `test -f output/task3930-stage6/char-shape-audit.md`
  는 exit code `0` 및 `report-ready`를 반환했다. 따라서 build와 audit 실행도 exit code `0`이다.
- 생성 파일: `output/task3930-stage6/char-shape-audit.md`
- 핵심 결과:
  - Hancom `915`개, rhwp `937`개 CharShape 중 raw semantic key 기준 `unique_different`가 `733`,
    `ambiguous`가 `182`, `unmatched`가 `22`다. Stage 4의 733개 fail-closed replacement와 같다.
  - inactive underline/strike/shadow를 제거한 logical key 기준은 `equivalent 914`, `unmatched 23`이다.
    즉 전역 logical canonicalization 후보가 넓지만, Stage 5 PDF 회귀 때문에 이 숫자만으로 production
    변경을 정당화할 수 없다.
  - `unique_different` 733개 각각의 실제 `PARA_CHAR_SHAPE` run/문단/text sample을 보고서에 기록했다.
- 한계 확인: rhwp 생성 HWP의 `PARA_LINE_SEG`에는 `bit 0`(페이지 첫 줄) 표식이 없어 누적 쪽수가
  `0`으로 나왔다. 따라서 이 HWP 내부 line segment만으로 Stage 4의 raster 변경 PDF 7쪽을 style과
  직접 연결할 수 없다. 보고서에서 style ID `79`, `82` 등이 검색되는 것은 PDF 쪽번호와의 연관이
  아니라 우연한 record ID 일 수 있으므로 근거로 사용하지 않는다.
- 판정: audit은 raw/logical 분류와 실제 text 사용 위치에는 유효하지만, 페이지 provenance에는
  불충분하다. Stage 6에서는 HWPX header의 명시/생략 속성을 별도 추적해 source-derived 조건의
  존재 여부를 확인해야 한다.

### 3. HWPX source 표현 탐색 1차

- 명령:
  - `rg -n -C 4 'charPr|CharShape|char_shape|underline|strikeout|shadow' src/hwpx src -g '*.rs'`
  - `unzip -l 'samples/2025 행정업무운영 편람(최종).hwpx'`
  - `unzip -p ... Contents/header.xml | rg -o '<hp:charPr[^>]*'`
- 결과:
  - 저장소에 `src/hwpx` 디렉터리는 없어서 첫 경로 가정은 실패했다. HWPX parser 실제 위치를
    별도로 찾아야 한다.
  - archive listing은 대량 BinData 항목으로 잘렸고 `Contents/header.xml` stream 추출은 `hp:charPr`
    시작 태그를 찾지 못했다. 이 결과만으로 header가 없거나 `charPr`가 없다고 결론 내릴 수 없다.
- 판정: 다음 탐색은 zip entry 이름을 `rg`로 한정하고, 실제 parser module 및 XML namespace를 확인한
  뒤 수행한다. 아직 source-derived 조건의 유무는 미판정이다.

### 4. HWPX source 표현 탐색 2차

- 명령:
  - `rg --files src | rg 'hwpx|hwp_x|xml'`
  - `unzip -Z1 'samples/2025 행정업무운영 편람(최종).hwpx' | rg '(^Contents/|header|section)'`
  - `unzip -p ... Contents/header.xml | rg -o '<[^>]*charPr[^>]*>'`
- 결과:
  - 실제 parser는 `src/parser/hwpx/header.rs`이며 HWPX header entry는 `Contents/header.xml`이다.
  - header의 `<hh:charProperties itemCnt="937">` 아래에 `<hh:charPr id="0" ...>`부터 937개가
    존재한다. rhwp 생성 HWP CharShape 수 `937`과 ID 공간이 동일하다.
  - `charPr` start tag에는 `height`, 색, `shadeColor`, `useFontSpace`, `useKerning`,
    `borderFillIDRef` 등이 명시되고 underline/strike/shadow는 child element 또는 생략으로
    표현될 가능성이 있다.
- 판정: source HWPX가 CharShape ID를 보존하므로 record 순번 자체는 source-derived다. 그러나 Stage
  5가 ID 전역 canonicalization의 PDF 회귀를 보였으므로, 다음에는 각 `charPr` child의 explicit/default
  표현과 parser가 이를 IR로 낮추는 규칙을 비교해야 한다.

### 5. HWPX `charPr` child와 parser lowering 확인

- 명령:
  - `sed -n '1,360p' src/parser/hwpx/header.rs`
  - `unzip -p ... Contents/header.xml | sed 's/></>\\n</g' | awk '/<hh:charPr id="0" /,/^<\\/hh:charPr>/'`
  - 같은 방식으로 `id="23"` block 추출
  - `rg -n -C 3 'parse_char|underline|strike|shadow' src/parser/hwpx/header.rs ...`
- 결과:
  - source `charPr 0`, `23`은 모두 `<hh:underline type="NONE" shape="SOLID">`,
    `<hh:strikeout shape="NONE">`, `<hh:shadow type="NONE" color="#C0C0C0" offsetX="10"
    offsetY="10">`를 명시한다.
  - parser는 `underline NONE`을 model `UnderlineType::None`, `strikeout NONE`을
    `strikethrough=false`, `shadow NONE`을 `shadow_type=0`으로 낮춘다. shadow color 및 offset은
    type NONE에도 보존한다.
  - 현재 model은 이 child들이 원본에 명시됐는지, 어떤 placeholder 문자열이었는지를 provenance로
    보관하지 않는다. 그러므로 현재 serializer만으로는 source의 `NONE/SOLID`와 일반 model default를
    구별할 수 없다.
- 다음 코드 변경 계획: audit 명령에 선택 `--source-hwpx <file.hwpx>`를 추가해 raw header의
  charPr child signature를 ID별로 읽어 보고서에 합친다. 이는 production model을 바꾸지 않고 source
  표현이 733개와 ambiguous/unmatched 204개를 구별하는지 확인하는 진단이다.

### 6. HWPX decoration signature audit 구현 및 단위 검증

- 변경:
  - `hwp5-char-shape-audit`에 선택 `--source-hwpx <원본.hwpx>`를 추가했다.
  - ZIP의 `Contents/header.xml` 원문을 읽어 charPr ID별 underline/strikeout/shadow child의
    attribute signature를 보존하고 raw CharShape 분류와 교차 집계한다.
  - production parser/model/serializer는 수정하지 않았다. 이 명령은 source XML이 충분한
    선택 조건인지 판정하는 진단 전용이다.
- 명령: `cargo fmt && git diff --check && cargo test --lib diagnostics::hwp5_char_shape_audit -- --nocapture`
- 관측: Cargo test/rustc 프로세스가 완료되어 더 이상 실행 중이지 않다. 이 실행 환경의 출력 stream에는
  최종 test summary가 남지 않아 pass 수를 해당 stream만으로 확정할 수 없다.
- 판정: 형식과 compile 결과를 아직 성공으로 선언하지 않는다. 다음에는 이미 생성된 test binary 또는
  lock이 없는 Cargo 재실행으로 명시적인 exit code를 확인한 뒤 실제 HWPX audit을 한 번 실행한다.

### 7. 단위 검증 재실행

- 명령: `cargo test --lib diagnostics::hwp5_char_shape_audit -- --nocapture`
- 관측: Cargo/rustc 프로세스는 종료됐다. 그러나 이 실행 환경은 다시 최종 test summary를 반환하지
  않았다. 아직 pass라고 선언하지 않는다.
- 판정: 현재 target에 새로 생성된 rhwp lib test binary를 직접 찾아 동일 filter로 실행해 명시적인
  pass/fail 결과를 확인한다. 이는 Cargo 재빌드를 반복하지 않는 검증이다.

### 8. test binary 식별 실패

- 명령: target/debug/deps/rhwp-7afa96d71b7995e1 diagnostics::hwp5_char_shape_audit --exact --nocapture
- 결과: exit code 2. 이 파일은 lib test binary가 아니라 CLI binary여서
  알 수 없는 명령 diagnostics::hwp5_char_shape_audit를 반환했다.
- 판정: 코드 또는 테스트 실패가 아니다. target의 다음 executable(rhwp-7bda83202b26df44)을
  lib test binary로 식별해 같은 filter를 재실행한다.

### 9. lib test binary 확인, filter 수정 필요

- 명령: target/debug/deps/rhwp-7bda83202b26df44 diagnostics::hwp5_char_shape_audit --exact --nocapture
- 결과: exit code 0, lib test runner가 실행됐다. 다만 --exact filter가 module prefix만 가리켜
  3277개 중 0개 test가 실행됐다.
- 판정: test runner와 binary 식별은 정상이다. --exact를 제거하고 module substring filter로
  diagnostic의 5개 단위 test를 실행한다.

### 10. 기존 lib test binary 부분 확인

- 명령: target/debug/deps/rhwp-7bda83202b26df44 hwp5_char_shape_audit --nocapture
- 결과: exit code 0, 기존 binary의 4개 diagnostic test는 모두 통과했다.
- 한계: 새 source HWPX signature test가 목록에 없다. 이 binary는 새 test 추가 전 build일 수 있으므로
  source 변경 검증 근거로 충분하지 않다.
- 판정: 현재 source 기준으로 Cargo test를 다시 수행하고, 새 test 이름이 포함된 최종 결과만 Stage 6
  성공 근거로 채택한다.

### 11. 최신 source Cargo test 재실행

- 명령: cargo test --lib diagnostics::hwp5_char_shape_audit -- --nocapture
- 관측: Cargo/rustc는 종료했지만 최신 source(15:15 KST)보다 lib test binary(15:06 KST)가 오래돼
  새 signature test가 link되지 않았다. 즉 이 실행은 compile error 또는 비정상 종료로 끝났을
  가능성이 높으며, 성공으로 취급하지 않는다.
- 판정: pseudo-TTY로 동일 Cargo test를 실행해 compiler stderr 전체를 수집하고, 오류가 있으면
  다음 코드 수정 전에 정확한 메시지를 Stage 문서에 기록한다.

### 12. pseudo-TTY Cargo test도 결과 stream 누락

- 명령: cargo test --lib diagnostics::hwp5_char_shape_audit -- --nocapture (pseudo-TTY)
- 관측: `Compiling rhwp` 뒤 Cargo/rustc가 종료했지만 stdout/stderr의 최종 결과가 반환되지 않았고,
  lib test binary timestamp도 갱신되지 않았다.
- 판정: test failure인지 host resource/runner 종료인지 구분되지 않는다. 다음에는 파일시스템 여유와
  kernel/OOM 로그를 확인해 compiler 종료 원인을 먼저 판별한다. 코드 변경은 하지 않는다.

### 13. host resource 점검

- 명령:
  - df -h /home/tsjang
  - free -h
  - journalctl -k 및 system journal에서 최근 OOM/no-space/cargo/rustc 검색
- 결과:
  - root filesystem는 146G 중 139G 사용, 가용 약 1.0G(사용률 100%)다.
  - 메모리 available 약 6.3GiB, swap available 약 2.7GiB이며 최근 kernel/system journal에서
    OOM killer 또는 no-space 메시지는 찾지 못했다.
- 판정: OOM 근거는 없지만, rhwp link/test 산출물 갱신 중 1G 여유는 부족할 수 있다. 다음 Cargo test는
  output 하위 로그에 stdout/stderr를 보존해 실제 compiler 실패 원인을 확정한다. 기존 target이나
  사용자 산출물은 삭제하지 않는다.

### 14. compiler stderr 보존 재실행

- 명령: CARGO_BUILD_JOBS=1 cargo test --lib diagnostics::hwp5_char_shape_audit -- --nocapture
- 로그: output/task3930-stage6/cargo-test-source.log
- 결과: compile 실패. Rust E0716 at src/diagnostics/hwp5_char_shape_audit.rs:263에서
  event.name().as_ref()가 임시 BytesEnd/BytesStart name value를 가리켜 local name slice가
  statement 이후 dangling borrow가 됐다.
- 원인: quick-xml event name의 borrowed slice를 local 변수에 유지하려면 name object 자체를
  같은 scope binding으로 유지해야 한다.
- 다음 코드 수정: Start event branch에서 event.name()을 name binding에 저장한 뒤
  xml_local_name(name.as_ref())를 사용한다. Empty event와 End event에도 같은 패턴을 적용한다.

### 15. quick-xml borrow 보정 및 재검증

- 변경: Start/Empty/End event 처리에서 event.name() 결과를 name binding으로 유지한 뒤 local name을
  추출하도록 수정했다.
- 명령: cargo fmt && git diff --check && CARGO_BUILD_JOBS=1 cargo test --lib diagnostics::hwp5_char_shape_audit -- --nocapture
- 로그: output/task3930-stage6/cargo-test-source.log
- 결과: exit code 0. formatting/diff 검사를 통과했고, rhwp test profile 재빌드(2m 53s) 후
  5개 target test가 모두 통과했다. 새 HWPX explicit NONE child signature test도 포함됐다.
- disk 관측: 빌드 후 root filesystem 가용 공간은 약 379MB다. target 또는 기존 산출물은 삭제하지
  않았으며, 후속 검증은 새 full Cargo build를 피하고 이미 생성된 binary를 재사용한다.

### 16. 실제 source HWPX decoration signature 교차 집계

- 입력:
  - source HWPX: `samples/2025 행정업무운영 편람(최종).hwpx`
  - 기존 HWP audit: `output/task3930-stage6/char-shape-audit.md`
- 방법: `Contents/header.xml`의 937개 `charPr`에서 underline/strikeout/shadow child attribute
  signature를 ID별로 읽고, 기존 보고서의 generated CharShape raw 분류와 교차 집계했다.
- 결과:
  - 가장 큰 signature `underline NONE/SOLID/#000000`, `strikeout NONE/#000000`,
    `shadow NONE/#C0C0C0/10,10`는 682개지만 `unique_different 539`, `ambiguous 134`,
    `unmatched 9`를 동시에 포함한다.
  - 두 번째 signature(동일하되 shadow `#B2B2B2`) 206개도 `unique_different 185`,
    `ambiguous 16`, `unmatched 5`로 섞인다.
  - 그 외 underline BOTTOM 또는 strikeout 3D signature도 대부분 unique/ambiguous를 함께
    포함한다. 단일 상태가 되는 작은 signature가 있더라도 Stage 4의 733개 replacement를 일반화할
    수 있는 충분 조건은 아니다.
- 판정: 현재 HWPX의 child 명시 장식 signature는 raw sentinel의 source-derived production 선택
  기준이 될 수 없다. Hancom HWP record나 고정 ID에 의존하지 않는다는 Stage 6 제약을 지키기 위해
  serializer는 변경하지 않는다.

### 17. CLI 등록·문서 및 실행 가능성 확인

- 변경: `src/main.rs` dispatch, `capabilities`, `--help`에 `hwp5-char-shape-audit`를 등록하고,
  `mydocs/manual/cli_commands.md` §4에 입력 계약, `--source-hwpx`, exit code, 한계를 기록했다.
  `--out report.md`처럼 parent가 빈 상대경로인 경우는 폴더 생성을 건너뛰도록 보정했다.
- 명령: `CARGO_BUILD_JOBS=1 cargo build --bin rhwp`
- 결과: source 오류가 아니라 root filesystem 여유가 약 300MB인 상태에서
  `librhwp.rlib` archive 생성이 `No space left on device`로 실패했다. 기존 target 또는 사용자
  산출물은 삭제하지 않았다.
- 보완 근거: Section 15의 최신 source 모듈 test 5건은 exit code 0으로 통과했다. parent 경로
  guard 추가 뒤에는 `cargo fmt --check`와 `git diff --check`를 통과했다. 새 CLI binary의
  `--source-hwpx` 실제 실행과 최신 source 재컴파일은 디스크 용량을 확보한 뒤 다음 Stage에서
  재실행한다.

## Stage 결론

- Stage 4의 7쪽/631픽셀 개선은 Hancom oracle을 이용한 probe 결과일 뿐, 현재 source HWPX의
  decoration provenance로 일반화할 수 없다.
- production CharShape serializer는 Stage 2 byte-identical 상태를 유지한다.
- 새 diagnostic은 다음 fixture에서 source 기반 조건이 발견되는지 재현 가능하게 확인하는 용도로만
  남긴다. 실제 source fixture에 대한 `--source-hwpx` 보고서 실행은 디스크 확보 뒤의 별도 Stage
  검증 항목이다.
