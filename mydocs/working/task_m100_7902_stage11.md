# #7902 11단계 — 옛한글 우선 face의 paint 소유 보존

Issue: #7902. [10단계](task_m100_7902_stage10.md)의 고정 producer 전쪽 검증과 전체 Rust는
계속 진행한다. 그 결과를 아래 추가 보완의 fresh WASM 검증으로 대신하지 않는다.

## 확인한 결함과 근거

Canvas의 기존 `draw_text_positioned`는 옛한글 자모 cluster에서
`CanvasTextFont.old_hangul_descriptor`로 font를 전환한다. 그러나 새 HFT 윤곽 helper는
`style.font_family`의 프로그램을 사용했다. **원래 family가 현재 paint face라는 가정이 잘못됐다.**
원래 CFF에도 같은 자모가 있으면 우선 선택한 `RHWP Old Hangul` face를 덮어썼다.

- 원래 독창적인 사각형 CFF에 U+A960 cmap만 더한 font-only 입력으로 실제 WASM에서 재현했다.
  `RHWP Old Hangul`은 기존 공개 OTF 원본을 실제 FontFace에 적재했다. 한컴 정확도 표본은 아니다.
- 원래 painter는 자모 윤곽을, source `3a6c7d0e0`는 사각형을 그렸다. 같은 HWPX·두 font bytes로
  확인했고 PNG·SHA·glyph 경계는 ignored `stage10-oldhangul-boundary/`에 보존했다.
- 정식 browser 검사도 같은 source의 실제 WASM에서 의도한 원인으로 **FAIL**했다.
  Print/U+A960 등록 전 `fillText`의 font는 `RHWP Old Hangul`이었고 등록 뒤 호출이 사라지며
  실제 PNG가 달랐다. 빌드·환경 실패가 아니다. `stage10-oldhangul-regression-before.log`에 기록했다.

## 보완과 소비 경로

`cluster → 기존 contains_old_hangul_jamo → old_hangul_descriptor 선택 → glyph paint`의
소유를 유지한다. 같은 cluster에 전용 face를 고른 경우 원래 family의 HFT 윤곽 replay를
사용하지 않고 기존 `fillText`가 계속 그린다. 문서명·문서 ID·임의 좌표 조건은 추가하지 않는다.
수정은 WASM 전용 자식 `web_canvas/hft_outline.rs`에만 둔다. Native depfile에서 해당 파일의
참조0건을 확인했으며 Native paint·측정·배치 코드는 바꾸지 않았다.

## 정식 검사와 배선

`scripts/tests/hft-outline-font-selection.browser.mjs`는 실제 fresh WASM·HML import·FontFace·
Canvas를 실행한다. 합성 입력은 메모리의 단일 문자이며 시험지 fixture나 활성 문서 corpus를
추가하지 않는다. 두 face의 실제 glyph가 있는 입력으로 우선 선택을 검증한다.

- Screen/Print/HighQuality의 자모 PNG가 등록 전후 동일하고 기존 전용 face로 그려야 한다.
- 일반 `가`의 Print/HighQuality는 HFT 윤곽 경로를 계속 사용해야 한다. 전체 기능을 꺼서
  자모 검사만 통과하는 잘못된 보완도 검출한다. Screen은 기존 painter를 유지한다.
- CI가 fresh WASM을 빌드한 뒤 이미 실행하는 `visual-sweep-canvas-fonts.browser.mjs`에서
  새 검사를 import한다. workflow는 변경하지 않으며 기존 adapter 검사를 유지한다.
- 로컬은 `RHWP_SWEEP_TEST_WASM_PKG=<fresh package> node --test
  scripts/tests/visual-sweep-canvas-fonts.browser.mjs`로 실행한다. CI 기본 package는 `pkg/`다.
- fixture generator는 `--jamo-only`로 두 번째 원본 프로그램만 생성할 수 있다. 기존 CFF의
  바이트는 그대로이며 새 사각형 font도 저장소 MIT 라이선스다.

## 실행 상태

helper 보완 뒤 fresh WASM을143초에 빌드했다. 실제 새 검사와 기존 browser 검사까지
**4 PASS**다. 새 검사는 Screen/Print/HighQuality×일반 HFT/옛자모6조합을 실행했다.
동일 정식 검사의 수정 전 FAIL/후 PASS를 `stage10-oldhangul-regression-before.log`와
`stage11-font-browser-tests.log`에 연결한다. CI frontend package worker의 기존 실행 파일에서
import하므로 새 검사를 수동 실행 전용으로 두지 않았다.

10단계 전체 Rust의 실패1건은 새 API 분류 누락이었다. `SessionState` 분류를 명시하고 등록·
재등록·교체 후 원본 IR 전체와 실제 저장 HWP 바이트가 같다는 검사로 근거를 보완했다.
또한 전체90%·104쪽 양쪽98%·확대 판독을 마친 HWPX 원본에서 HFT 선언의 실제 paint style
전달, 독립 Print의 세 본문 기준선·그림 네 변과 앞뒤 순서·비겹침을 검사하도록 보완했다.
Native producer와 이 producer/원점 경로는 추가 WASM 전용 보완에서 바뀌지 않았다.

무효화 분류·저장 bytes·완료 쪽의 focused nextest는 **20 PASS**다. 분류 가드5개와 HFT
세션 저장 불변, 완료 페이지14개 검사를 실제 실행했다. guard를 삭제하거나 허용치를 완화하지 않았다.
104쪽 추가 검사는 같은 핵심 경계인 표 종료→제목·본문→그림→후속 문단을 보호하므로
다른 의미의 높은 점수 쪽으로 대체하지 않았다.

`stage11-final-full`에서 새 WASM19문서·835쪽 재검증을 완료했다. 분류 수정 뒤 전체 Rust도
다시 실행해 최종0실패를 확인했다. Native Skia3종과 필수 lint를 이어 확인한다. 기존 Native 전쪽
재사용은 WASM 전용 자식이 실제 소비 경로에 포함되지 않는 근거로만 판단한다.


## HWP/HWPX104쪽의 같은 핵심 경계

HWP104쪽도 Native99.95192%·WASM99.93860%다. 두 형식의 각 실제 PNG가 backend별로
SHA-256 동일하며 각각의 독립 PDF raster도 같다. 두 PDF에서 별도로 추출한 세 기준선과
그림 네 변도 일치했다. `stage11-hwp-hwpx-104-reading-identity.json`에 실제 hash·독립 좌표를
보존했다. 같은 실제 raster의 확대 판독을 연결하고 HWP·HWPX 원본을 모두 같은 정식 검사에서
실행하도록 보완했다. 14개 완료 페이지 **검사**가 보호하는 문서·쪽 수와 검사 함수 수는 구분한다.


## 보완 후 전체 결과

- 수정 후 전체 release-test: **10,427 PASS·45 skipped·실패0**, exit0.
  compile4분40초·실행697.251초다. 새 양식별104쪽 검사도 실제 PASS했다.
- 최종 WASM19문서·835쪽은 전쪽90% 이상·쪽수 일치·누락0이며 Native835쪽과 함께
  devel 대비0.01%p 이상 하락0쪽이다. 두 backend 전체 최대 하락0.00846%p다.
- 새 WASM의835개 실제 PNG와19개 TSV는 frozen3a 산출물과 각각 바이트까지 같았다.
  새 producer로 재생성한 사실과 이전 같은 raster의 직접 판독을 구분해 연결한다.
  `stage11-final-develop-comparison.json`, `stage11-rerender-pixel-identity.json`에 기록했다.
- 최저 쪽102의 Native/WASM review를 추가 생성해 본문 줄 구성·부호·목록·영문 괄호와
  하단 표를 판독했다. 하단 표의 남은 괘선 높이 차이는 base에도 있고 현재와 그 영역 픽셀·
  표 bbox가 같다(`stage11-page102-existing-region.json`). 이 변경으로 해결했다고 주장하거나
  기준 좌표를 새로 고정하지 않는다. 동일한 초기 출력 차이와 현재 회귀를 구분한다.
- Native Skia 공식3종은 exit0이다. 라이브러리3861+15+165+2=**4043 PASS·12 ignored**,
  필수 그림/직접 PDF와 HFT·TTF 원점 통합 검사는 **11 PASS**다. 컴파일과 실행의 최종 종료를
  확인했으며 필터0건·중간 빌드 출력을 통과로 세지 않았다.
- Python Visual Sweep152개, HFT 준비10개, CI Node224개·workflow Python44개,
  수정 workflow2개의 actionlint가 모두 통과했다. 기존 source unit-tier policy도 확인했다.
- 최종12개 review와12개 standalone overlay를 기존 실제 PNG로 합성했다. SVG는 생성하지
  않았다. [증적 설명](../pr/assets/issue7902-final-20261016/README.md)에 입력·원본 SHA·페이지와
  직접 판독을 연결한다. 원본19개·독립 PDF19개 모두 실제 HEAD Git blob과 SHA가 같다.
- PR 준비 중 fetch한 devel은 여전히 `f5251e7de`다. 변경이 없어 리베이스하지 않았다.
  추가 Native feature의 글꼴 추적·원본 내용 경계16개도 모두 PASS했다.
- 필수 fmt/재확인, 기본 Clippy, WASM Clippy, workspace build, workspace all-target Clippy,
  고정 base manifest 검사가 전부 exit0이다. 순차 Cargo jobs8·공용 target을 유지했다.
  `stage11-final-lint.json`에 각 명령·종료값·시간을 보존했다. source unit-tier도 exit0이다.
- [기준/후보 전체 TSV76개](https://gist.github.com/jangster77/c62ec80625417cef370c6258132baee2)를
  압축 없이 제출했다. API에서 README 포함77파일의 바이트/SHA 일치를 재확인했다.
  최종 GitHub CI·merge는 이후 실제 head에서 확인하며 로컬 통과와 구분한다.

완료 페이지54출력의 최종 점수도 모두98% 이상이다. 기존 확대 판독과 최종 PNG가50/54개
바이트까지 같고 독립 PDF raster는54개 모두 같다. 달라진4개는 Native/WASM×HWP/HWPX
104쪽이며 위 새 직접 판독과 같은 형식별 raster 동일성으로 연결한다.
`stage11-completed-page-raster-reuse-audit.json`에 점수·원본/최종 PNG와 PDF SHA를 보존했다.
따라서 과거 producer의 판독을 새 출력으로 바꾸어 보고하지 않으며 실제 같은 raster만 재사용한다.

## 실행 명령과 제출 경계

저장소 루트에서 `CARGO_BUILD_JOBS=8`,
`CARGO_TARGET_DIR=/home/tsjang/rhwp/target/pr-review`를 사용했다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/tsjang/rhwp/target/pr-review \
  --tests --test-threads 12 --no-fail-fast
cargo test --locked --profile release-test \
  --target-dir /home/tsjang/rhwp/target/pr-review \
  --features native-skia --lib -- --test-threads 12
RHWP_SWEEP_TEST_WASM_PKG=output/pr-review/issue7902-render-repair-20261014/stage11-fresh-wasm \
  node --test scripts/tests/visual-sweep-canvas-fonts.browser.mjs \
  scripts/tests/visual-sweep-font-registration-browser.test.mjs
```

fresh WASM은 같은 source의 dev/no wasm-opt 빌드다. 최적화한 배포 package로 보고하지 않는다.
전쪽 `visual_sweep.py` 실행의 문서별 원본·Print·compat·9개 공급 font 경로와 실제 argv는
`stage9-final-full-native-result.json`, `stage11-final-full-wasm-result.json`에 보존한다.
공통 옵션은 `--rasterizer png --embed-fonts full --dpi 96 --page-workers 2 --silhouette-only`다.
WASM은 고정 `--wasm-pkg stage11-fresh-wasm`을 추가했고 Native CLI도 고정 복제본을 사용했다.
정확한 명령·SHA를 보존하는 이 최종 원시 기록은 output 정리에서도 재현 자료로 유지한다.

로컬 준비는 완료됐다. 사용자 승인한 collaborator self PR로 제출하고 정확한 code head의 CI를
확인한 뒤 merge한다. 개별 PR review·오늘할일은 사용자 선택 옵션2의 별도 문서 PR로 처리한다.
`#7445` 하위 이슈의 다른 Print/축소본·인쇄 배율·편집/캐시 계약을 이19문서 증거로 종료하지 않는다.
