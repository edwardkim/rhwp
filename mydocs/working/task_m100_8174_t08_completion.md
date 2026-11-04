# #8174 T08 — 최종 통합 검증 완료

2026-10-28. [완료 계획](../plans/task_m100_8174_t08_completion.md)의 **G01–G06을 한 단위로 완료**했다.
T08 필수 잔여는 0건이다. 전체 이슈의 PR 제출에는 T09의 최신 devel 통합·기능 보존·제출 확인이 남는다.
실제 registry 게시·토큰/owner 조작·원격 push·PR 생성은 수행하지 않았다.

## 변경과 고정 기준

엔진·CLI·WASM의 일반 실행 코드·조판 규칙·기능 기본값·자원 한도는 바꾸지 않았다. source unit 33개의 공개 계약
대응과 필요한 검사 보강, 기존 integration 원본 19개의 동일 바이트 이동, Native CI의 실제 실행
누락 연결을 처리했다. 원본 보존·공개 API로 주입할 수 없는 private 합성 callback의 구분과
HWP3/column-width 선행 분석은 [unit 이관 기록](task_m100_8174_t08_unit_migration.md)에 있다.
33개를 같은 수의 공개 검사로 단순 이동했다고 설명하지 않는다.

- 정책 base: `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`. base 확인만으로 rebase하지 않았다.
- Rust 검증 source: `483ef2e58d`. root의 공개 WASM 검사 보강은 `751f705aee`, 제품·포장 기준은
  `a8023ede7051534c9d85de4dfebb1cfde60588b7`이다. tracked Rust·manifest·lock 4,087개가
  두 체크아웃에서 동일함을 `rust-validation-source-cohort.json`으로 확인했다.
- 공용 target은 `target/pr-review` 한 곳이며 Cargo 실행은 순차다. 검증 워크트리는 저장소 밖에
  두고 생성된 suite/manifest를 stage하지 않았다. Root `Cargo.lock`은 변경하지 않았다.

## G01–G03: 실행 경로·제품과 통합 게이트

| 검사 | 현재 결과 |
| --- | --- |
| 최신 고정 base의 정책 | unit 정적 집계 4,099·suite 28 + exception 20 = 48 target, PASS. 기준선 상향 없음 |
| 전체 nextest | **11,695 PASS**, 46 skipped. 실제 실행 1,356.148초. 기존 사용자 지정 `page_border_fill_reference_sources_keep_paper_and_page_basis` 제외만 유지 |
| Native Skia 전체 library | root **3,857 PASS**, 기존 ignore 12·사용자 제외 1. 관련 library 182 PASS |
| 집중·Native Skia 연결 | frontend admission 2·HWP3 1·glyph replay 10·암호/코덱·요청 수명/자원·missing picture·direct PDF 실제 실행 PASS |
| Rust lint/build | fmt, Native Clippy, WASM32 library Clippy, workspace build·전체 target Clippy, feature-off library, 관련 library PASS |
| workspace 밖 검사 | PDF backend **5,938 PASS·기존 ignore 2**, JPEG **7 PASS** |
| CI 연결 검증 | workflow/포장 preflight 계약 92·impact classifier 45·test 배치 계약 PASS. actionlint 1.7.12 PASS (`shellcheck`·`pyflakes` 별도 실행은 아님) |
| Studio | Node **2,473 PASS·2 SKIP**, TypeScript PASS. 새 WASM 생성 후 배포 빌드도 다시 PASS |
| 실제 브라우저 | **271 assertions·17 Workers**. 메모리 실패·engine trap 대조 포함. 실제 Studio 오류/취소/만료·세대 경쟁·복구와 양쪽 저장/재열기 PASS |
| Native 격리 host | 일반 사용자·user namespace root에서 실제 열기·HWPX 저장·재검증 PASS |

`skia_native_glyph_replay`는 기본 feature의 전체 suite에서 실행되지 않았다. 고정 base의 Native CI에도
호출이 없음을 확인해 기존 Native job의 release-test/release 분기에 한 호출씩 추가하고
classifier 소유 목록을 연결했다. 새 job이나 중복 전체 suite 호출은 만들지 않았다. 게시 사전 점검의 실제 CI 누락도 확인해
기존 Lint의 workflow 계약 단계에 source/publish Python 검사 한 묶음을 연결했다.
실제 GitHub CI를 새로 실행한 기록은 아니며 위 수치는 로컬 실행 결과다.

공개 SVG 검사에서 UTF-8 XML 선언·제한 subtree 뒤 gradient를 실제 paint/raster로 검증했다.
참조 cycle·지수적 use 확장을 거절하고 새 정상 요청으로 복구했다. WASM의 제한 요소 뒤 gradient는
정상 gradient 및 독립 Chrome SVG 출력과 **전 픽셀이 동일**하다.

### 제품과 실제 반영

표준 locked wrapper의 web package, mixed `rlib+cdylib`, release·`wasm-opt` 경로를 사용했다.
`RHWP_WASM_CDYLIB_ONLY=0`, 별도 RUSTFLAGS 없음, release codegen units 16이다.
Native는 메모리 여건에 맞춘 release-test/root opt-level 1·codegen units 64의 기능 검증 제품이다.
Native의 일반 release 최적화 성능으로 설명하지 않는다.

- fresh WASM: **25,600,261 bytes**, SHA-256
  `5f5f7128db9f3f3aaa09adcd720d6faa575ffc0f5266f7832a04c7d95f8031e7`.
- root `pkg/`·Studio `public/`·`dist/`의 JS/WASM hash가 일치한다. 실제 Studio HTTP 응답도
  같은 WASM hash를 확인했다. T06 표준 WASM 대비 **+164 bytes**다.
- 실패/취소 중 기존 편집·autosave draft·recent·파일 핸들을 보존했다. 정상 PDF와 JPEG 가져오기,
  편집한 HWP/HWPX의 재열기와 실제 UI의 `T08EDIT`·그림 표시를 확인했다.

## G04: 전체 페이지와 직접 판독

동일 원본·독립 PDF·기존의 실제 글꼴 공급으로 재출력했다. 각 runtime의 전체 쪽수와
TSV 누락·90% 미만·보호 페이지 98% 조건을 확인했다.

| 범위 | 문서/쪽수 (runtime마다) | Native 최저 raw | fresh WASM 최저 raw |
| --- | --- | --- | --- |
| CFF·Type3·본문/셀·clip·비공개 입력 | 17 / 49 | 98.34664% | 98.79246% |
| 인라인 흐름·저장 LineSeg | 4 / 38 | 아래 전쪽 원장 | 아래 전쪽 원장 |
| JPEG sampling·progressive·alpha | 17 / 17 | 100% | 100% |
| 합계 | **38 / 104** | **98.34664%** | **98.79246%** |

총 **208개 현재 페이지 PNG가 T07과 바이트 단위로 동일**하다. 같은 페이지의 표·셀·그림·캡션,
후속 내용·인라인 개체/빈 줄·명시적 개행 경계와 CFF ligature/악센트/그리스 문자,
Type3의 숫자·단위·±·기호 간격을 비교·overlay·확대 crop으로 직접 판독했다.

독립 PDF와 엄격 RGB 100% 또는 모든 글꼴 피델리티의 완료를 주장하지 않는다. 예를 들어
`distribution` 3쪽 WASM 목록의 획은 PDF/Native보다 얇으며, 이 기존 T05–T07 차이도 이번
출력과 동일하다. Type3 획·JPEG 보간/색상 경계의 기존 차이도 남는다. 이번 테스트 정리로 생긴
배치·줄바꿈·내용 누락이나 새 차이는 관찰하지 않았다. 판독·잔여 차이는 `direct-review.json`,
현재 PNG 동일성은 `t07-page-byte-equivalence.json`, 원본 TSV는 `visual/`에 보존했다.

## G05: 빌드와 시각 검증 종료 후 비용

Native/PDF와 WASM은 같은 구성의 보존 T06 제품과 비교했다. Root lock의 외부 dependency
버전과 기능 구성을 유지하며 본체 버전 0.8.7 → 0.9.0 및 T08 source 정리가 있는 비교다.
네 번 교대 실행하고 첫 회를 제외한 세 번의 중앙값을 사용했다. Native는 CLI 프로세스 실행
전체 시간, WASM은 import/export/reopen **각 중앙값의 합**이다.

| 입력 | Native 전체 ms | WASM 각 구간 중앙값 합 ms | Native GNU time max RSS KiB |
| --- | --- | --- | --- |
| CFF 1쪽 | 20.2 → 25.2 | 5.4 → 5.0 | 31092 → 31348 |
| Type3 12쪽 | 379.5 → 385.5 | 503.4 → 507.7 | 48084 → 48180 |
| 비공개 15쪽 | 2353.7 → 2352.0 | 3487.1 → 3497.3 | 114028 → 114228 |

세 입력의 저장 결과 bytes는 대조/후보가 동일하고 실제 재열기 SVG도 일반 재열기와 일치한다.
Native CFF는 약 +5ms, Type3 +6ms, 비공개 입력은 거의 같았다. WASM Type3는 약 +4.3ms,
비공개 입력은 약 +10.2ms다. 작은 반복 수의 관측이며 보편적 속도 향상/무영향 보장은 아니다.
WASM의 이 연속 측정에서 유지된 linear memory 최대는 양쪽 **704,512,000 bytes**다.
이는 live heap·RSS 또는 전체 프로세스 메모리 상한 증명이 아니다. Native RSS도 GNU time의
관측값이며 프로세스 그룹 동시 합계가 아니다.

### 다페이지 실제 Studio

보존 S09 표준 WASM `1e7282ad64fae37bbf9684c1e77008e132473075`와 최종 제품을 같은 현재
Studio frontend에서 교대로 사용했다. warmup 6회 + 측정 18회 = 24회이며 문서마다 세 쌍이다.
현재 HTTP WASM hash를 확인했고 텍스트·커서·115/415쪽·deferred pagination 완료를 실제 검사했다.
별도 서버와 browser는 종료했다. 이 항목은 S09 이후의 누적 변화이며 T08 테스트 정리만의
효과로 분리하지 않는다.

| 문서 | UI 로딩 중앙값 ms (대조 → 후보) | 변화 | WASM load 중앙값 ms |
| --- | --- | --- | --- |
| 115쪽 HWP | 4,964.3 → 4,896.2 | -1.37% | 1,338.6 → 1,316.5 |
| 115쪽 HWPX | 8,146.0 → 8,013.0 | -1.63% | 1,198.7 → 1,174.7 |
| 415쪽 HWP | 15,485.0 → 14,589.9 | -5.78% | 3,852.5 → 3,658.5 |

| 입력 위치 | 동기 입력 중앙값 ms | 두 번째 rAF 중앙값 ms | rAF 변화 |
| --- | --- | --- | --- |
| 115쪽 HWP 영문 | 2.2 → 2.4 | 806.4 → 825.3 | +2.34% |
| 115쪽 HWP 한글 IME | 13.8 → 13.6 | 830.2 → 803.9 | -3.17% |
| 115쪽 HWPX 영문 | 3.5 → 3.3 | 808.6 → 785.2 | -2.89% |
| 115쪽 HWPX 한글 IME | 15.3 → 14.8 | 844.7 → 834.7 | -1.18% |
| 415쪽 1쪽 | 5.2 → 5.2 | 727.7 → 1,363.9 | +87.43% |
| 415쪽 208쪽 | 15.0 → 14.3 | 788.9 → 776.3 | -1.60% |
| 415쪽 415쪽 | 5.2 → 4.8 | 1,381.0 → 705.3 | -48.93% |

화면 갱신의 긴 대기는 남는다. 특히 415쪽의 1쪽 +87.43%와 마지막 쪽 -48.93%를 숨기거나
일괄 개선으로 묶지 않는다. 타임스탬프를 대조하면 다음 입력의 긴 작업이 앞 입력의 두 번째
rAF 전에 들어가는 경우가 섞인다. 1쪽에서 이 겹침은 대조 4/9·후보 5/9, 마지막 쪽은 6/9·4/9다.
1쪽 첫 rAF 중앙값은 691.6 → 677.5ms, 해당 입력을 포함한 긴 작업은 690 → 678ms였고,
마지막 쪽은 각각 687.1 → 675.0ms·686 → 674ms다. 같은 긴 작업이 두 번째 rAF 지표의
여러 입력 구간에 중복 포함될 수 있어 그 비율만으로 WASM 실행 비용의 지속적 악화를
단정하지 않는다. 원래 표본과 `summary.tsv`는 수정하지 않았다.

관측 결론은 **로딩은 소폭 감소, 동기 입력은 대체로 유사하며 메인 thread의 긴 작업/화면 대기는
계속 남음**이다. 이전 조사에서 확인한 문서 전체 Canvas 메트릭 수집 문제를 해결한 것으로
표시하지 않는다. 이번 범위에서 무관한 편집/렌더 최적화를 추가하지 않았다.

## G06: 실제 archive와 독립 소비

새 실제 `.crate` **14개·16,245,899 bytes**를 생성했다.
13개 자식의 일반 source payload는 T07과 같고 변경은 VCS provenance 및 일부 생성 lock의 내부
package checksum이다. 생성 lock의 외부 버전/SourceID 변경은 0건이다. 본체는 cfg(test)·지원
항목 정리와 기존 column 계약 파일 추가가 있으며, 일반 생산 텍스트는 동일하다.
Root manifest/lock·라이선스·고지·runtime 자료의 source 파일을 완화하거나 빼지 않았다.

새 checksum을 기존 Cargo cache에 덮어쓰지 않고 새로운 로컬 sparse origin으로 검증했다.
인덱스의 내부/외부 dependency registry 구분은
[Cargo 공식 인덱스 계약](https://doc.rust-lang.org/cargo/reference/registry-index.html)을 따랐다.
실제 다운로드한 14개의 SourceID·체크섬·작업 트리 밖 경로를 대조했다. path/patch 의존성은 없다.

- 독립 Rust의 HWP/HWPX 열기·저장·편집·재열기와 typed error PASS.
- **새 archive의 Native CLI를 직접 빌드**하고 이를 연결한 공개 정책/요청 수명/코덱/암호
  **54개 계약 PASS**. 예전 설치 CLI로 대체하지 않았다.
- 별도 WASM 소비자의 facade on/off **현재 archive 컴파일 PASS**. 이 단계에서 새 archive
  소비자 전체의 optimized/browser matrix를 다시 실행했다고 설명하지 않는다. 표준 Root 제품의
  최적화·Worker/Studio 검증은 위 G03에서 수행했다.
- 독립 소비자는 새 lock으로 해석해 T07과 외부 버전 집합이 60개 이름에서 달랐다.
  `final-package/consumer-dependency-delta.json`에 실제 graph를 보존했다. 메인 Root lock과 성능 비교의
  고정 dependency 구성을 갱신한 것이 아니다.

### 최종 컨테이너 무결성 보정

초기 생성본을 전체 gzip 스트림 끝까지 검사해 **6개 archive의 잔여 바이트**를 검출했다.
Root는 12,671 bytes, 나머지는 1–4 bytes였다. tar 원본·manifest·VCS·원장 checksum과
라이브러리 소비가 통과해도 컨테이너 전체가 유효하다고 결론 내릴 수 없음을 실제로 확인했다.

실행 중인 Cargo가 없는 상태에서 이번 실행의 정확한 14개 공유 `.crate` 출력만 ignored
`final-package/previous-shared-package/`로 보존하고, 파일이 없는 경로에서 순서대로 재포장했다.
target/프로필/cache를 삭제하지 않았다. 최종 **14개 모두 단일 gzip 종료·CRC·잔여 바이트 0** 및
기존 파일/정규화/VCS 검사를 통과했다. 새 origin `48179/alternate/`에서 실제 다시 내려받고
Rust·Native CLI·54개 계약·WASM on/off를 실행했다. 초기 origin과 cached 파일을 덮어쓰지 않았다.
최종 기준은 **`final-package/packages.json`·`final-package/crates/`**이며 초기 `crates/`는
실패 증거로만 보존한다.

읽기 전용 게시 사전 점검기에 bounded chunk gzip 검사를 추가했다. 동일 checksum의 잔여 바이트,
추가 gzip member, 잘린 footer, 잘못된 CRC의 회귀 4개는 **기존 코드 4 FAIL → 보정 후 PASS**다.
source/publish 사전 점검 36개와 workflow 계약을 함께 실행해 **92 PASS**를 확인했고,
최소 CI 연결과 실제 포장마다 이전 출력 파일을 보존하는 운영 가이드를 보강했다.
최종 원본 source payload는 초기 소비 원본과 동일하며 생성 lock의 내부 checksum 변경을 구분했다.
컨테이너 보정/재빌드는 앞선 성능 측정과 겹치지 않았고 측정 제품·Root lock은 바뀌지 않았다.

검증 스크립트의 합쳐진 Cargo 로그 JSON 처리·sparse SourceID 형식·외부 계약의 CLI 경로/feature
전달을 보정했다. 통과한 immutable archive/소비 결과는 유지하고 실패한 준비 단계만 이어 실행했다.
생산 코드나 계약 assertion을 바꾸어 우회하지 않았다. 로컬 검증이며 실제 crates.io upload,
공개 registry의 publish/dry-run·이름 예약·owner 확인의 완료를 뜻하지 않는다.

## 증적과 종료

검증 당시 원시 증적은 ignored `output/pr-review/issue8174/t08-completion-20261028/`에 생성했다. 임시 자료는 종료 정리 대상이며, 아래 기록은 당시 검증의 source·명령·결과를 남긴다.
비공개 15쪽 입력과 식별 정보는 로컬 검증 자료로 유지한다.

| 원장 | 내용 |
| --- | --- |
| `gates.json`·`focused.json`·`frontend.json` | 정확한 명령·작업 경로·시간·결과 |
| `source-freeze.json`·`rust-validation-source-cohort.json`·`production-cohort.json` | source/생산 코드 대응 |
| `native-producer.json`·`candidate-wasm-producer.json`·`fresh-frontend-finished.json` | 제품·동기화 hash와 실제 배포 빌드 |
| `runtime-checks.json`·`studio-t08.json`·`browser/` | 실제 UI·Worker·저장/복구·HTTP hash |
| `visual/`·`direct-review.json`·`t07-page-byte-equivalence.json` | 전쪽 TSV·현재 PNG·직접 판독 |
| `native-cost.json`·`cost.json`·`pdf-cost-summary.json` | 교대 비용·관측 메모리·동일 저장 결과 |
| `large-doc/measured-results.json`·`large-doc/summary.tsv`·`large-doc/raf-task-attribution.json` | 원시 24회·18회 측정 집계·겹친 작업 근거 |
| `final-package/source-preflight/`·`final-package/crates/`·`final-package/consumer-gates.json`·`final-package/selected.json` | 실제 archive·다운로드 source·독립 소비 |
| `final-package/package-delta.json`·`final-package/normalized-lock-delta.json`·`final-package/initial-source-payload-equivalence.json` | 포장 차이·생성 lock·동일 feature graph |

문서 메타데이터의 기존 5개 파일 20건은 T07과 같으며 새 오류는 0건이다. 생성 suite/manifest,
임시 로그·PNG·PDF·비공개 자료는 commit하지 않는다. 최종 증적 원장과 뒤따르는 문서 commit에서
source·포장 입력 바이트 유지 여부를 다시 고정한다. 다음 절차는 **T09**다.
