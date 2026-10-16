---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_7911_stage4.md
last_verified: 2026-10-16
---

# #7911 4단계 — 최신 devel 통합 검증

## 1. 범위와 기준

메인테이너의 다음 절차 진행 승인에 따라 3단계 checkpoint `1e763f3f7`에 최신 devel을
병합하고 최종 회귀를 확인한다. 저장 HWP의 한컴 재조판 1·11쪽 차이는 승인된
[B10-1](../plans/task_m100_7911_backlog.md#21-b10-1--저장-hwp의-한컴-재조판-시-111쪽-배치-차이)
후속 구현으로 유지한다. 추가 보안 감사·그림 조각 병합 등 백로그 소진을 이번 통합의
선행 조건으로 확대하지 않는다. 기본 PDF feature 비활성 및
`parser_allocation_bounds_proven=false` 상태를 유지한다.

- 원격 devel: `84764fa21c96cd5f8a0e158658e4162a4fb3ce9e`.
- 기존 base `f5251e7de` 이후 #7941의 렌더러·HFT 글꼴·표 배치 보정 1개 커밋이 추가됐다.
- 기본 작업공간 devel은 위 SHA로 fast-forward했다.
- 통합 commit: `54d039dfb9c96fa3eb866901f07bdab206c36074`. 코드 충돌 없이 병합했다.
- 검증용 `rhwp-studio/public/rhwp.js`는 백업 후 기존 바이트를 보존했다.
- 증적: `output/pr-review/issue7911/integration-20261016/`.

## 2. 검증 계획과 현재 상태

승인된 로컬 통합 검증을 완료했다. 기본 Rust lint 묶음·base 고정 manifest/tier 정책·전체
release-test, PDF 활성 집중 검사·Native Skia 3종, Studio 타입/테스트와 실제 Worker를
확인했다. 병합으로 공통 렌더러가 바뀌었으므로 새 Native 및 Docker WASM을 생성해 같은
12쪽 원본 PDF와 전쪽 비교했다. 승인된 B10-1 미달은 별도로 유지한다.

공유 target은 `/home/edward/mygithub/rhwp/target/pr-review`를 재사용한다. 실행 전 다른
Cargo 작업이 없음을 확인했다. Docker는 기존 `rhwp_wasm-target`을 사용한다.
이번 단계는 로컬 통합 검증이며 원격 push·PR 상태 변경·merge는 수행하지 않는다.

## 3. 검증에서 발견한 보완

workspace/all-targets Clippy에서 `tools/diagnostics/font_metric_coverage_worker.rs`의
`FileFormat::Pdf` match 누락으로 E0004가 발생했다. HWP/HWPX 전용 격리 진단 도구의
기존 계약에 맞춰 PDF를 명시적 `unsupported` 분기로 추가했다. 제품의 PDF 지원 범위를
진단 도구로 확대하지 않는다. 보정 후 같은 workspace/all-targets Clippy가 통과했다.
수정 전 로그는 `workspace-clippy-before.log`, 수정 후는 `workspace-clippy.log`다.
기본 Native/WASM Clippy·workspace build·fmt·base 고정 manifest/unit-tier도 통과했다.

전체 nextest 첫 실행은 10,435건 중 10,430 통과·5 실패·45 skip이었다. 실패는 새
`import-pdf` 명령의 콘텐츠 제작 프로필, `diagnostics` 필드 사전, 자동 생성 명령 장,
출처 카탈로그·fixture 연결 누락이다. 테스트 기대값을 바꾸지 않고 실제 등록을 보완했다.
명령 장은 현재 바이너리 자기서술과 `tools/gen_agent_codex.py::command_chapter`로 생성하고
향후 생성기의 가족 목록도 갱신했다. 첫 실패는 `full-regression-before.log`에 보존한다.

release build와 release library 3,986건(12 ignored)은 통합된 파서·저장·렌더 소스에서
통과했다. 그 뒤 변경은 위 메타데이터 등록과 문서뿐이므로 이 라이브러리 증거를 재사용하고,
최종 metadata를 포함하는 전체 release-test와 Rust lint는 다시 실행했다. 빌드 프로필과
검증 소스의 차이를 숨기지 않는다.

PDF 활성 workspace/all-targets Clippy는 자원 검사 helper의 `write!` 단일 줄바꿈 사용을
검출했다. 생성 PDF의 바이트를 유지하는 `writeln!`로 보정하고 해당 자원 111건을 다시
실행해 통과했다(`resource-after-lint.log`). 제품 파서·저장·렌더 소스나 기대값을 바꾼
수정이 아니므로 이미 통과한 전체 회귀 10,435건과 PDF 집중 130건은 그 범위에서 재사용한다.
최종 lint 묶음과 feature-on workspace 검사도 보정 후 통과했다.

실제 Worker 검사는 83개 assertion을 마친 뒤 CDP Chrome의 테스트용 HTTP 연결이 남아
서버 종료를 기다리다 180초 timeout으로 취소됐다. 테스트 탭과 CDP 연결은 이미 닫혔지만
해당 Node 프로세스의 fixture HTTP socket 1개가 남은 것을 확인했다. 테스트 소유 서버의
`close()` 호출 후 `closeAllConnections()`로 잔여 연결을 정리하도록 보정했다. 같은 WASM으로
재실행한 검사는 4.57초에 정상 종료했다. `worker-teardown-before.log`와 `worker.log`에
실패/성공을 보존하며, 제품 변환 실패나 제한 완화로 처리하지 않았다. 이 JS cleanup 수정은
Rust 제품 소스를 바꾸지 않으므로 Rust 전체 검사·Docker 빌드는 재사용했다.

## 4. Rust·Studio 통합 결과

최종 기록은 `validation-steps.json`, `final-lint.json`, `runtime-results.json`에 실제 명령과
종료 코드로 남긴다. 기본 구성의 전체 nextest는 **10,435 통과, 0 실패, 45 skip**이다.
앞서 누락됐던 다섯 계약은 같은 검사에서 FAIL→PASS를 확인했다. 기대값·baseline·ignore는
완화하지 않았다.

| 검증 | 결과 | 로그 |
| --- | --- | --- |
| fmt·기본 Native/WASM Clippy·workspace build/all-targets | 최종 묶음 통과 | `final-*.log` |
| base 고정 manifest·unit tier | `84764fa21` 및 최종 `375aff294` 기준 통과 | `final-base-manifest.log`, `final-base-unit-tier.log` |
| release build·library | 3,986 통과·12 ignored; 후행 metadata/test-helper 변경과 구분해 재사용 | `release-build.log`, `release-lib.log` |
| 전체 release-test | 10,435 통과·45 skip | `full-regression.log` |
| Native Skia library | 4,043 통과·12 ignored | `native-skia-lib.log` |
| Native Skia 그림·직접 PDF | 각각 2·4 통과 | `native-skia-picture.log`, `native-skia-pdf.log` |
| doc test | 8 통과·3 ignored | `doctest.log` |
| Studio typecheck·test | 타입 통과, 2,014 통과·2 skip | `studio-types.log`, `studio-test.log` |
| PDF 활성 library Clippy·집중 회귀 | lint 통과, 130 통과 | `pdf-clippy.log`, `pdf-focused.log` |
| PDF 활성 workspace/all-targets | helper lint 보정 후 통과 | `pdf-workspace-clippy.log` |
| 보정한 자원 helper | 111 통과 | `resource-after-lint.log` |

로컬 nextest는 0.9.137이며 저장소 권장 0.9.140보다 낮다는 경고가 있다. 사용하지 않은
`ci-duration-observation` profile의 `report-skipped` 키만 무시됐으며 default 검사는 위와 같이
완주했다. 이 로컬 결과를 아직 실행하지 않은 새 head GitHub CI 성공으로 쓰지 않는다.
Docker WASM·대조군·전쪽 비교는 다음 절에서 판정한다.

변경 문서 7개의 내부 상대 링크 검사는 통과했다. 별도 전체 문서 metadata 검사는 기존
`mydocs/tech/` 문서 4개에서 필수 필드 16개 누락을 보고했다. 같은 검사와 문서를 최종
devel `375aff294`에서 대조해 오류 목록이 정확히 같음을 확인했다. 현재 변경에서 생긴
실패는 아니며 미해결로 남긴다(`docs-metadata-comparison.json`). 해당 전체 검사를 통과로
표시하거나 무관한 문서를 이번 변경에 섞지 않았다.

## 5. 기존 포맷 대조와 제품 실행

`84764fa21`의 기본 작업공간에서 Native Skia CLI를 별도로 빌드해 같은 입력을 현재
통합본과 대조했다. `hwp_table_test.hwp` 3쪽, `footnote-tbox-01.hwpx` 1쪽,
`formatting_table.hml` 1쪽의 **전체 SVG 바이트가 동일**했다. 이는 이 3개 대조군의
출력 보존 증거이며 전체 corpus의 한컴 일치 판정으로 확대하지 않는다.
`native-controls.json`에 양쪽 바이너리·입력·전쪽 SVG 해시를 고정했다.
후행 harness 호출은 같은 바이너리/입력 해시를 확인하고 이 결과를 재사용한다.
WASM에서도 같은 3개 문서 5쪽의 전체 SVG 바이트가 devel과 동일했다.

후행 devel `375aff29498166260a22796d54e67f508c84aada`는 #7942의 검토 기록·오늘할일
두 파일만 추가했다. 기본 devel을 fast-forward하고 작업 branch에
`4665335997911f667d4c49d699da4de0fb45ca91`로 병합했다. 제품 소스 해시는 같으므로
컴파일·전체 회귀를 반복하지 않았다. manifest/unit-tier 정책은 이 최종 base SHA로 다시
실행해 통과했다(`final-base-manifest.log`, `final-base-unit-tier.log`).

기본 Native CLI와 Docker WASM은 PDF 입력을 `PDF_IMPORT_DISABLED`로 거부하고 기존 HWP를
열었다. feature-on 공개 API/실제 Worker는 **83항목·5개 Worker**에서 전체 12쪽과 선택 쪽,
입력 오류·예산 오류·deadline·취소·기존 세션 보존·다음 정상 요청 복구를 확인했다.
추가 브라우저 검사 19항목에서는 실물 PDF의 `번호` 12개를 `순번`으로 바꾸고 HWP/HWPX로
저장·재열기해 편집 텍스트, 전체 12쪽, 각 쪽의 그림 개수가 보존됨을 확인했다. 이는 해당
편집 여정의 계약 검증이며 모든 편집 기능 또는 편집본의 한컴 시각 일치를 뜻하지 않는다.
증거: `default-cli-controls.json`, `wasm/browser-result.json`, `browser-controls.json`.
바인딩·editor 검사 3건도 통과했다(`bindings.log`).

## 6. 전쪽 시각 검증과 재사용 경계

원본은 기존 Git 입력 `pdf/issue5875/nested_table_text_caption-2020.pdf`(12쪽,
SHA-256 `6308671fc56b1bfd9e72660edd6edf574a4e16f75e84e0bf35399339d9ce3702`)다.
새 Native importer의 저장 HWP/HWPX는 이전에 한컴에서 확인한 보정본과 바이트가 동일했다.
재현 입력은 `samples/issue7911/pdf-import-nested-table.{hwp,hwpx}`에 보존한다.

| 저장 입력 | 전체 쪽수 | Native 최저 | fresh Docker WASM 최저 | 90% 미만·누락 |
| --- | --- | --- | --- | --- |
| HWP | 12 | 90.14609% (11쪽) | 90.16832% (11쪽) | 없음 |
| HWPX | 12 | 90.14609% (11쪽) | 90.16832% (11쪽) | 없음 |

4개 전쪽 TSV는 `{native,wasm}-source-scores/{hwp,hwpx}/silhouette.tsv`다. 새 head에서
출력했으며 각 backend의 12쪽 RGB 픽셀은 기존 사용자 승인 P3 출력과 모두 동일했다.
Native 3쪽은 96.84817%, WASM은 96.82916%다. 대표 1·3·11쪽을 다시 직접 읽어 3쪽
그림의 오른쪽 숨김 영역, 세 조각 연결과 셀 내부 배치, 11쪽 그림 9·10과 캡션 보존을
확인했다. 글꼴·래스터의 미세 차이는 남아 있으며 완전한 픽셀 일치를 주장하지 않는다.
기존 사용자 시각 판정과 통합 후 출력 보존을 연결하며 사용자에게 같은 판정을 반복 요청하지 않는다.

새 저장본 SHA-256은 HWP `ac245bae37b408b5c4af69ab39f6ef53505a47433034fc34288714704a107d99`,
HWPX `032e87dc63af02de4330e6f19b0578abaabf521d17f38015b1a9d3da9523603b`다.
동일 저장본의 기존 한컴 2020 Print를 `pdf/issue7911/pdf-import-{hwp,hwpx}-hancom2020.pdf`에
보존했다. 저장 바이트와 양쪽 렌더 픽셀이 모두 같으므로 Print·재조판 비교는 중복 실행하지 않았다.
**원본 PDF와의 통과 수치를 저장본 한컴 Print와의 통과로 바꾸지 않는다.** B10-1의 1쪽
68.40217% / 68.72149%, 11쪽 78.54899% / 78.74329%(Native/WASM)는 그대로 남는다.
이는 메인테이너가 승인한 후속 구현이며 배포 준비 또는 다른 미달의 포괄 예외가 아니다.

실행은 `runtime-results.json`에 전체 argv와 종료 코드를 기록했다. 재현 명령의 핵심은
`scripts/visual_sweep.py --silhouette-only --hwp <저장본> --pdf
pdf/issue5875/nested_table_text_caption-2020.pdf --key <hwp|hwpx> --rhwp-bin <검증 CLI>
--compat 2022 --dpi 96 --embed-fonts --font-path <p11/fonts> --out <증적 경로>`이며
WASM에는 `--wasm-pkg pkg`를 추가했다. 같은 조건의 일반 모드 `--pages 1,3,11`로
대표 review·overlay를 생성했다. 실제 존재하는 공급 폰트와 설치 Chrome 경로를 사용했다.

Studio `http://localhost:7791/?review=issue7911-integrated-20261016&lang=ko`에서 저장 HWP
12쪽과 3쪽 표시, 오류 없음, 실제 fetch WASM SHA를 확인했다. `pkg/`와 `rhwp-studio/public/`의
JS/WASM도 동일하다. Windows 확인 자료는 `E:\tmp\issue7911-integration-20261016\index.html`에
모았으며 원본 PDF·저장 HWP/HWPX와 상대 경로 PNG를 포함한다.

## 7. 산출물과 관측 비용

최종 Rust 제품은 merge `466533599`와 3절의 보정을 포함한다. 이 보고서와 같은 로컬
checkpoint에 source·검사·문서·대표 증거를 함께 커밋한다. `integration-state.json`과
`final-evidence.json`에서 수정 파일·입력·산출물 해시를 연결한다.

| 산출물 | bytes | gzip bytes | SHA-256 |
| --- | ---: | ---: | --- |
| Native CLI | 53,325,752 | 비측정 | `28bd5498dfec8a7609fd26fa3c2c2e404ecc66e65b5b8d7fe2e7b8428b3ca9b3` |
| default-off WASM | 12,595,965 | 4,697,583 | `a9af265d47540202e841c8279049e9e31241ea310274ce98fdaf187d7be5f602` |
| PDF-on WASM | 17,131,699 | 6,506,119 | `47964d1a4a18bd9770c5ad524c5e6a1ebe996699b1e5000e695709393f0f742b` |

Docker 안에서 feature-on WASM Clippy가 통과했고 locked wrapper의 default-off web build는
8분 33초, feature-on은 9분 20초였다. 기존 Docker target을 재사용한 이번 관측이며
깨끗한 환경 빌드 시간이나 성능 목표로 일반화하지 않는다.

Native PDF→HWP는 0.24초·peak RSS 40,248 KiB, PDF→HWPX는 0.28초·38,128 KiB였다.
Chrome 155의 cold/warm import는 545.6/251.2ms, 첫 전쪽 render는 205.2/97.7ms,
편집·양 형식 저장·재열기까지 합계는 1,974.2/1,300.7ms다. 관측 WASM linear memory
capacity는 48,889,856 bytes이며 peak heap/RSS가 아니다. 추출·인식·lowering별 시간은
공개 API에 별도 계측이 없어 미측정이다. 이 구분과 일반 자원 상한 감사는 B07에 남긴다.

## 8. PR 갱신 준비와 남은 절차

로컬 검증과 갱신 본문 `integration-20261016/updated-pr-body-ko.md`를 준비했다.
원격 #7913은 아직 Draft·head `bc5ff569022bd296c3bfe2f904118b9e81d488e4`이며,
현재 로컬 결과를 그 원격 head의 CI 통과로 표시하지 않는다.

- 원격 push·본문 갱신·필요한 CI 실행 승인 후 정확한 새 code head의 CI를 확인한다.
- code head CI 성공 뒤 self-review·오늘할일을 문서-only trailing commit으로 기록하고
  후행 CI, Ready·merge 승인을 순서대로 확인한다.
- B10-1과 B01–B09 등 후속 일감은 유지한다. 현재 기능의 로컬 검증 완료와 후속 개선·
  비신뢰 입력 배포 준비 완료를 구분한다. 이슈 본문은 `Refs #7911`로 범위를 드러낸다.

PR 본문·향후 merge 댓글에 사용할 최종 asset은
`mydocs/pr/assets/issue_7911_pdf_import/{native,wasm}-{review,overlay}-{003,011}.png`
8개다. 3쪽 자르기·11쪽 그림 보존과 최저 점수 페이지를 보여준다. 임시 산출물은
`integration-20261016/{native,wasm}-review/hwp/`에 있다. 본문에는 실제 최종 code SHA를,
향후 댓글에는 실제 merge SHA를 사용하는 `edwardkim/rhwp` raw URL로 PNG를 직접 표시한다.
후속 self-review에서 이 계획과 B10-1의 남은 차이를 확인하며, 게시 승인 후 `--body-file`과
API 재조회로 본문·asset·SHA 정합을 검증한다. 중간 로그·TSV·JSON·생성 WASM/JS는 커밋하지 않는다.
