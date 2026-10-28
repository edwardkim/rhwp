# #8174 T06 — 의존성·JPEG 호환

2026-10-28. **E01–E05 완료, 필수 잔여 0건.**
[확정 범위](../plans/task_m100_8174_t06_completion.md)를 구현·검증·가이드·완료 보고까지 한 단위로 처리했다.
기준 source는 T05 `f7ede1d54eb018bea2f49405b3d1cba030e9360d`, 계획 커밋은 `cd0f1d9d1d`다.
이 문서와 함께 완료 커밋을 기록하고 최종 source/제품/증적을 대조했다. T07–T09와 전체 PR 준비는 남는다.

## E01 — 좁힌 의존성 갱신과 암호 호환

root lock의 chacha20만 **0.10.1 → 0.10.2**로 갱신했다. 버전·checksum 두 항목 외의
의존성은 변경하지 않았다. 공식 crates.io metadata와 archive checksum을 다시 대조했다.
후보 checksum은 `65c35e4b699c7e15ccbe7ee35c005e4fc0a278d22238a2857e6ce2dadeda1b06`이다.
[upstream #580](https://github.com/RustCrypto/stream-ciphers/pull/580)의 실제 통합
`075ccbd357ad7cbd9ec934a011b9a379966d42a2`와 0.10.2의 SSE2 보정도 확인했다.
현재 feature graph와 실행하지 않은 ISA/OS의 보증을 혼동하지 않는다.

T05의 실제 Native library로 agent 봉인·비밀번호 트레일러·PQ 트레일러 대조군을 만들고
생산 binary/library·자료 hash를 고정했다. 새 Native 제품이 기존 자료를 열고 기존 제품이 새
자료를 여는 양방향 계약을 통과했다. 잘못된 암호·키와 AAD/암호문 변조는 거절하며 평문/가시층은
보존한다. `tests/fixtures/issue8174_crypto_compat/`의 키·암호·평문은 모두 공개 시험 데이터다.
정식 공개 API 회귀 2개도 통과했다. 독립 Rust/WASM 소비자는 같은 공개 API로 기존 자료와
새 WASM 자료를 열고 오암호·키 불일치·AAD/암호문 변조를 거절했다. 새 WASM 자료를 기존
Native 생산자로 다시 열어 양방향 호환까지 실제 실행했다.

root와 독립 소비자의 Native/wasm32 feature graph 4개 모두 chacha20 0.10.2의 `cipher`·
`xchacha`를 선택하고 `rng`·`legacy`는 선택하지 않는다. 독립 소비는 저장소 밖 별도 Rust
프로젝트의 path 소비이며 14개 최종 registry archive 소비 완료로 확대하지 않는다.

## E02 — 공통 frame sampling 기하 보정

원 도입 PR #7913의 실제 통합은 `4662c069e557e928ed7968d061121fde90b1f027`이다.
이때 들어온 `vendor/rhwp-zune-jpeg/src/misc.rs`의 setup, 후속 요청 예약 패치와
[S06](task_m100_8174_v2_stage6.md)의 52개 독립 cjpeg/djpeg 입력·실패를 현재 source에 대조했다.
착수 시 fetch한 `upstream/devel`은 `af6bb05b26887135fb9da0fa4e54ca7205564fca`이며
그쪽도 같은 setup과 chacha20 0.10.1이었다. 아직 해결된 변경을 다시 구현한 것이 아니다.
최신 devel의 전체 동기화·다른 기능 보존은 기존 T09 범위로 남긴다.

| 실제 생산·소비 | 원인과 보정 |
| --- | --- |
| SOF 성분 → `setup_component_params` | 모든 성분을 읽기 전에 running Hmax/Vmax로 앞 성분의 stride를 확정했다. 전체 성분의 최댓값·MCU 격자를 먼저 확정한 뒤 각 성분의 크기와 stride를 계산하도록 바꿨다. |
| progressive coefficient plane → IDCT | 계수 배열은 최종 MCU 개수로 할당했지만 IDCT는 잘못된 앞 성분 stride를 소비했다. 예를 들어 9×9의 Y 1×2 / Cb·Cr 2×2에서 최종 MCU 폭은 16, Y stride는 8이어야 하는데 기존은 16이었다. 같은 격자의 `component.w2`를 stride로 사용한다. |
| baseline MCU → post-process | 같은 setup을 쓰므로 일부 baseline도 잘못된 행을 읽었다. 두 encoding을 공통 원인에서 함께 보정했다. 문서 ID·호출 지점·backend별 예외가 없다. |
| component 높이 | vertical factor와 최종 Vmax로 계산한다. 현재 이 필드는 메타데이터이며 IDCT 소비 경로가 아니다. 올바른 식에 새 Clippy 진단이 생겨 `div_ceil`로 보정한 뒤 제품을 다시 빌드했다. |
| 실제 예약과 손상 입력 | coefficient/MCU/upsampling의 기존 allocation hook과 progressive slice guard를 유지한다. guard 제거·clamp·다른 decoder 우회로 수용 여부를 맞추지 않았다. |

기존 52개에서 progressive **6건 거절 → 52개 전체 성공**을 확인했다. 성공하던 46개 중
42개는 픽셀 bytes가 같다. baseline 4개도 같은 원인으로 잘못된 행을 읽던 문제가 보정됐다.
독립 djpeg와의 RGB 채널 평균 절대 오차는 다음과 같다.

| mixed baseline | 기존 | 보정 |
| --- | --- | --- |
| 9×9 | 46.185 | 1.379 |
| 17×17 | 54.287 | 3.353 |
| 33×19 | 54.532 | 3.696 |
| 65×33 | 62.604 | 4.299 |

이는 색상 오차의 측정값이며 임의 허용치나 새 golden이 아니다. 독립 djpeg와 보간/IDCT의
모든 픽셀이 같다고 주장하지 않는다. 같은 source 계수의 baseline/progressive 26쌍은 보정
codec 안에서 픽셀이 같다. 독립 검정색 1×1 대조군은 RGB `[0, 0, 0]`과 일치한다.
정확한 decoded 예산·1바이트 부족·sticky 실패·손상 sampling 거절·새 요청 복구와
실제 PDF의 픽셀/alpha·양쪽 저장을 보호하는 공개 회귀 3개도 통과했다.

`tests/fixtures/issue8174_jpeg_compat/generate.py`로 별도 ignored 폴더에 재생성해 JPEG
52개와 PDF 17개의 bytes가 모두 보존된 입력과 같음을 확인했다. codec 계약과 독립 PDF의
시각 증거를 구분하며 새 렌더링 golden·좌표 pin·한도 완화는 없다.

## E03 — 실제 제품 소비와 편집 보존

최종 Native 제품에서 JPEG PDF 17개의 HWP/HWPX 저장·재열기 **34경로**를 통과했다.
그림의 크기·내용·alpha와 쪽수를 확인했다. 반투명 입력은 alpha 128을 저장·재열기 후에도 유지한다.

실제 최적화 fresh WASM의 공통 Worker에서는 같은 17개 PDF와 **18개 Worker**를 실행했다.
그림 픽셀/alpha, 양쪽 저장·재열기와 실제 `PdfEditorReopen` 결과가 유지됐다. codec 할당 한도
1의 실패 후보는 기존 활성 문서를 바꾸지 않았고 다음 정상 요청도 성공했다.
기존 전체 Worker 계약 **267 assertions / 17개 Worker**도 현재 제품에서 통과했다.

실제 Studio에서는 기존 편집 내용·저장 draft·recents·핸들의 보존, 입력/재열기 한도·취소·
deadline/generation 경쟁과 정상 15쪽 복구를 확인했다. 보정 대상/alpha·정상 대조군 JPEG PDF
8개를 UI로 열었고, 공개 편집 API로 `T06EDIT`를 넣고 reflow한 뒤 양쪽 저장·재열기에서 글자와
그림이 함께 유지됐다. 실제 편집 HWPX를 다시 UI로 열어 글자·그림 표시도 직접 판독했다.
SVG의 글자는 glyph별 `<text>`로 나뉘므로 단순 raw 문자열 검색 대신 실제 text node를 모아 검사한다.

## E04 — 현재 제품·전쪽 시각과 필수 게이트

현재 최적화 WASM은 **25,600,097 bytes**, SHA-256
`eb688ad46302ca66588b28625e005fdebd4c1a289d82317cc8294994aef59e68`이다.
Native 기능 제품은 공용 target의 release-test·root opt-level 1/codegen-units 64로 빌드했다.
공용 `target/pr-review`의 Cargo 작업은 순차 실행하며 jobs/test threads는 4다.
이 Native 프로필을 T08의 최종 성능 빌드로 보고하지 않는다.

| 전쪽 범위 | Native 최저 raw | fresh WASM 최저 raw |
| --- | --- | --- |
| 기존 PDF 17개·49쪽 | 98.34664% | 98.79246% |
| 일반 HWPX Print 대조군 4개·38쪽 | 99.66991% | 99.17555% |
| 새 JPEG PDF 17개·17쪽 | 100% | 100% |

각 runtime **38개 문서·104쪽**이다. 기존 21개·87쪽의 PNG 174개는 T05와 bytes 동일하다.
전쪽 쪽수·누락·90% 미만·측정 불가를 확인했고 현재 compare/standalone overlay/review를 만들었다.
Type3의 단위·기호·작은 간격, inline 개체/빈 줄 경계와 JPEG의 색상 패턴·alpha·경계를 직접
판독했다. 기존 획/글리프 차이와 새 JPEG의 보간 색상 차이를 raw 실루엣과 구분한다.
기존 글꼴/선 두께 차이는 T05와 동일하며 새 배치·줄바꿈·누락 결함은 관찰하지 않았다.

최종 제품의 Native/WASM Clippy, 정식 suite/unit 범위, 집중 검사와 Native Skia 3종,
Studio TypeScript·Node 2,473 PASS/2 skipped·production build는 통과했다.
전체 nextest는 **11,726 PASS**, 46 skipped다. 기존 사용자 지정 제외
`page_border_fill_reference_sources_keep_paper_and_page_basis`만 유지했다. 최종 workspace gate와
독립 소비와 비용 측정까지 완료했다.

| 최종 검증 | 결과 |
| --- | --- |
| Rust 필수 lint/build | Native/WASM32 Clippy, workspace build/all-target Clippy, feature-off check, fmt 모두 PASS |
| suite/unit 범위 | 공식 prepare → fmt → prepare → manifest 검사 후 실제 실행. 최종 manifest/fmt도 PASS, T06 source unit 증가 0 |
| 공개 집중 검사 | 암호 2·JPEG 3·요청 수명 43·기존 PDF 자원 제한 134 PASS |
| 전체 release-test | **11,726 PASS**, 46 skipped. 기존 사용자 지정 제외 외의 추가 제외·기대값 완화 없음 |
| Native Skia 3종 | 전체 library 3,891 PASS·기존 ignored 12/사용자 제외 1, 관련 workspace library 182 PASS, missing-picture 2 PASS·direct-PDF 14 PASS |
| 독립 JPEG crate | 실제 standalone library unit **7 PASS** |
| Studio | Node 2,473 PASS·기존 skipped 2, TypeScript·Vite/PWA build·실제 UI PASS |
| 독립 Rust/WASM 소비 | 현재 source에서 빌드한 Native/WASM 각각 JPEG 52개·정확한 예산/부족/복구 PASS, 두 target의 픽셀 bytes 일치. 기존/새 암호 자료와 변조 거절·기존 Native의 새 WASM 자료 열기 PASS |
| Native 감독·CLI 복구 | 일반 사용자/user namespace root의 실제 허용·차단 PASS. 출력/진단/잔여 예산 실패 6건은 기존 출력과 빈 stdout을 보존하고 정상 후속 요청 PASS |
| 출력·첫 저장 | 기존 페이지 PNG 174개·Native 저장 37개가 T05와 bytes 동일. 실제 재열기 28쪽의 모든 SVG가 일반 생성자와 같고 handoff HWPX 3개도 before/candidate bytes 동일 |

독립 소비자의 Native 픽셀 52개는 앞서 독립 djpeg와 대조한 진단 결과의 픽셀 bytes와도 같음을
최종 source에서 확인했다. 위 오차 수치를 다른 source의 결과로 대체하지 않았다.

추가한 vendor 단독 Clippy의 `-D warnings`는 기존 upstream 스타일 진단 때문에 기준/후보
각각 exit 101·128건이다. 이번에 생긴 새 진단 1건을 고쳤고, 진단 집합 대조에서 **신규 0건**이다.
원시 실패와 차등 결과를 보존하며 이를 단독 Clippy 전체 PASS로 표시하지 않는다. lint 억제나
기존 스타일의 대량 수정 없이, 정본이 요구하는 root Native/WASM/workspace lint와 구분한다.
변경한 장기 문서의 metadata와 변경 Markdown 7개의 링크도 통과했다. 전체 metadata 검사의
기존 20건·5개 파일은 T05와 bytes가 같으며 신규 오류는 0건이다.

## 비용

Cargo·시각 작업 종료 후 같은 Chromium에서 before/candidate 순서를 번갈아 4회 실행했다.
첫 cold 회를 구분하고 warm 3회의 각 단계 중앙값을 합산했다. 아래는 한 호스트의 실측이며
일반 문서 편집 속도·전체 제품 latency나 속도 개선 보장이 아니다.

| 입력 | before import/export/reopen ms | 후보 import/export/reopen ms | 중앙값 합계 변화 |
| --- | --- | --- | --- |
| CFF 1쪽 | 3.2 / 1.2 / 1.0 | 2.9 / 1.2 / 1.0 | −0.3 ms |
| Type3 12쪽 | 317.0 / 100.7 / 76.5 | 312.9 / 98.6 / 89.5 | +6.8 ms, +1.38% |
| 비공개 대조군 15쪽 | 1,745.7 / 927.0 / 871.4 | 1,640.6 / 913.4 / 906.6 | −83.5 ms, −2.36% |

큰 문서 처리 뒤 retained WASM 선형 메모리 최대값은 양쪽 704,512,000 bytes로 같았다.
이는 다음 입력에서도 module이 보유한 capacity이며 문서별 live heap/RSS나 절대상한이 아니다.
최적화 제품은 **25,598,495 → 25,600,097 bytes**, +1,602 bytes(약 0.0063%)다.
기존에 실패하던 JPEG의 거절 시간과 정상화한 뒤의 처리 시간을 속도 비교로 사용하지 않았다.

## E05 — 증적과 완료 판정

최종 source의 증적은 ignored
`output/pr-review/issue8174/t06-completion-20261028/final/`에 모은다. 부모 폴더는 새 lint 보정
전 source의 제품·실행과 진단 시도 보존용이며 최종 결과를 대신하지 않는다.
생산 source·lock/graph·공식 archive·Native/WASM 제품·입력/폰트·정확한 명령/exit·전쪽 TSV·
대표 PNG·비용과 완료 커밋을 대조했으며 **E01–E05 필수 잔여는 0건**이다.

- `production-freeze.json`, `test-leaf-freeze.json`, `native-producer.json`, `candidate-wasm-producer.json`:
  현재 source·공개 검사·실제 제품 hash. pkg·Studio public·dist·실제 HTTP 응답의 WASM hash도 같다.
- `product-builds.json`, `final-gates.json`, `studio-gates.json`, `runtime-checks.json`:
  정확한 명령·exit·시간. `vendor-clippy-delta.json`은 위 추가 진단의 원시 실패와 신규 0건을 구분한다.
- `consumer-builds.json`, `consumer-producer.json`, `consumer-runtime.json`, `consumer-browser.json`,
  `crypto-native.json`: 독립 소비 graph·실제 암호/픽셀·예산·기존 생산자의 역방향 복호 검증.
- `jpeg-native.json`, `jpeg-browser.json`, `studio-t06.json`, `studio-jpeg-success.png`:
  실제 Native·Worker·편집/저장 UI. `codec-pixel-lineage-bridge.json`은 현재 source와 픽셀 오차 측정을 연결한다.
- `visual/candidate/`, `visual/inline/`, `visual/jpeg/`의 전쪽 TSV·현재 PNG,
  `full-page-reviews/`, `inline-reviews/`, `jpeg-reviews/`, `direct-review.json`의 현재 비교·overlay·확대 판독.
- `cost.json`, `cost-summary.json`, `quiet-cost-result.json`, `save-byte-equivalence.json`:
  마지막 단독 측정·보존 bytes. `completion-checks.json`, `final-source-files.json`,
  `evidence-manifest.json`, `validated-commit.json`: 최종 source·증적 hash·완료 커밋의 연결.

다음은 [TODO](task_m100_8174_pr_remaining_todo.md)의 **T07 최종 게시 계약·포장·독립 소비**다.
T06 필수 항목을 그쪽으로 이관하지 않았다. 기존 T08의 누적 source unit 정책 이관·최종 통합 성능과
T09의 최신 devel 보존·PR 제출 게이트는 그대로 남는다.

비공개 문서는 로컬 검증만 하고 공개 기록에는 집계만 남긴다. 원격 push·PR/comment·
registry 게시·토큰 설정·태그는 수행하지 않았다. T07–T09의 기존 잔여와 전체 heap/RSS·
동기 main thread 선점의 비보장은 유지한다. 이번 변경을 14개 최종 registry 소비나 전체 PR
제출 준비 완료로 확대하지 않는다.
