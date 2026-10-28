# PR #7947 self-review — Studio PDF 직접 열기와 빌드 의존성 호환성

## 최종 판정

**승인 — 기능·로컬 검증·code head CI를 확인했고, 메인테이너가 빌드 전용 Ajv 경고의 위험 수용을 결정했다.**
CodeQL의 중간 심각도 경고 5건은 아래 근거와 사용자의 명시 결정에 따라 `won't fix`로
dismiss했다. 취약 코드 보정이나 오탐 판정으로 표시하지 않는다. 사용자가 병합·관련 이슈
종료·후속 처리를 승인했다. 문서 기록 충돌을 해소하고 마지막 head의 검사를 확인한 뒤 병합한다.

| 항목 | 검토 기준 |
| --- | --- |
| PR·작성자·base | [#7947](https://github.com/edwardkim/rhwp/pull/7947) / edwardkim / devel |
| 검토한 code head | `432ba702f743ae6468f12120af90abbc630aa3e1` |
| 제품 검증 source | `46155219766953d8f4cb348ad96c1c8e97434354`; code head까지 문서·대표 PNG 12개만 추가 |
| 고정한 devel | `4662c069e557e928ed7968d061121fde90b1f027` |
| 관련 이슈 | [#7946](https://github.com/edwardkim/rhwp/issues/7946) Studio UI, [#7855](https://github.com/edwardkim/rhwp/issues/7855) brace-expansion, [#7856](https://github.com/edwardkim/rhwp/issues/7856) fast-uri |
| 작성 시점 참고값 | Open·MERGEABLE/CLEAN·reviewer 요청 없음. 병합 직전 다시 확인해야 한다. |

기본 경로는 [collaborator self-merge](../../manual/pr_review/collaborator_self_merge.md), 보조 경로는
[intake](../../manual/pr_review/intake_and_review.md), [local validation](../../manual/pr_review/local_validation.md),
[visual fixture evidence](../../manual/pr_review/visual_fixture_evidence.md),
[review-only fast-pass](../../manual/pr_review/review_only_fast_pass.md)다.
사용자의 “CI가 성공하면 self-review로 진행” 지시에 따라 code head CI 성공 후 이 기록을 작성했다.

### CodeQL 경고 추가 검토 (2026-10-16)

사용자 지적으로 후행 head `13f73cdfbbdd735e976cac444cd2734caad284fb`의
`github-advanced-security / CodeQL` annotation을 조회했다. check는 success이나 중간 심각도
5건(`#278`–`#282`, `js/prototype-polluting-assignment`)을 게시했다. 최초 code head의
JavaScript 분석 `1965967426`(2026-10-16 17:45:43 KST)에서도 이미 같은 경고를 검출했다.
이전 검토가 check 상태만 보고 annotation을 확인하지 않은 누락이다.

현재 JavaScript 분석 `1966087255`, tested merge `83416a36f7fc3519992de94a9bc5d1c0f3dbe896`,
CodeQL 2.27.1의 SARIF와 실제 코드를 대조했다. 스키마 조회 후 정리 경로의 TS/JS 네 건과
codegen scope 한 건으로 나뉜다. 검출 위치는 upstream Ajv 배포본과 바이트가 같지만,
실제 설치본의 격리 실행에서 codegen prefix 경로가 prototype을 변경하는 것을 재현했다.
정상 prefix 대조군에는 변경이 없었다. 스키마 조회 대조군은 경고 sink 이전 오류로 종료돼
해당 삭제 경로의 prototype 변경을 재현했다고 주장하지 않는다.

현재 사용처는 Studio·VS Code의 빌드 도구 개발 의존성이다. PDF/HWP 문서 열기에서 직접
악용되는 경로를 입증한 결과는 아니다. 객체 사전의 상속 키 조회와 scope 저장을 보정하고
reserved-key 반례·정상 스키마/코드 생성·빌드 호환성을 검증해야 한다. 경고 일괄 dismiss나
query/path 제외로 해결하지 않는다. 재현 상세·원 SARIF는 ignored 로컬 증적에만 보존한다.

후행 CI·CodeQL workflow도 success이나 이 경고는 미해결이다. 추가로 최신 devel
`cdef0073bb24512daf0622dec5051ac62cb8d57e`와는 `mydocs/orders/20261016.md` 한 파일이
충돌한다. push 전 양쪽 기록을 보존해 문서 범위에서 해소하고 merge tree를 재검증해야 한다.

### 메인테이너의 경고 처리 결정 (2026-10-16)

후속 범위 확인에서 Ajv가 Studio·VS Code의 Node 빌드 도구용 devDependency임을 확인했다.
Studio 배포 JS/map 9개·VSIX 55파일에 검출 코드가 없고 Chrome·Firefox의 production 모듈
그래프도 각각 285개 모듈·3개 chunk 중 Ajv/fast-uri 포함 0건이었다. 확장의 별도 runtime
복사 소스에도 관련 import가 없었다. 실제 prototype 변경 재현은 빌드용 codegen API를 직접
호출한 진단으로 구분한다.

사용자가 “Dissmiss alert 결정하겠습니다.”로 종료를 명시했다. 이에 경고 #278–#282를
`won't fix`로 dismiss하고 배포 미포함·빌드 범위 위험 수용을 한국어 사유에 기록했다.
경고 상태와 처리자 `edwardkim`을 다시 확인했으며 해당 PR의 열린 경고는 0건이었다.
실제 코드 변경으로 경고를 해결하거나 CodeQL query/path를 제외한 결과가 아니다.
후행 `13f73cdfb`의 Full CI·CodeQL·Render Diff·Adapter·Proptest 및 정책 status도 모두 성공했다.
이 결정으로 보안 경고에 따른 보류 사유를 해제한다. 앞선 분석·재현 증거는 유지한다.
상세 증거는 ignored `pr-preparation/codeql-deployment-scope-ko.md`,
`codeql-extension-module-graphs.json`, `codeql-dismiss-results.json`에 보존했다.

## 코드와 구현 주장 대조

PDF 입력은 Rust의 `supportsPdfImport()`가 실제 feature-on일 때만 허용한다. 이전 bundle의
`importPdf` export 존재만으로 기능을 켜지 않는다. 같은 capability를 native picker·fallback
input·drop이 소비한다. 기존 생성자 → 공통 IR → `convertToEditable` → 원자적 문서 교체 →
Canvas 경로를 사용하며 PDF→HWPX 중간 저장은 없다. 후보 생성 실패 뒤 기존 문서 보존과
정상 입력 복구는 실제 Studio에서 검증했다.

PDF source handle은 읽기·동일 파일 확인에 유지한다. `resolveSaveTarget`의 첫 저장 강제 분리,
HWP/HWPX 선택, `assertValidSaveHandle`의 쓰기 전 동일 entry 거절을 확인했다. 저장을 마친
출력 handle만 변환 완료로 표시하고 다음 저장에 재사용한다. PDF가 `.hwpx`로 이름 붙은
반례에서도 원본 쓰기가 0회였다. 실제 OS 대화상자의 수동 조작 검증으로 확대하지 않는다.

의존성은 FileList 2.0.2의 공식 minimatch 10 소비 경로와 Ajv 전용 URI 어댑터로 보정했다.
구형 함수형 export에 새 named API를 강제하지 않으며 Ajv codegen 표식은 어댑터가 소유한다.
Ajv 8에만 로컬 배포본을 적용하고 Ajv 6은 보존했다. 원본 LICENSE·파일 원장과 다섯 변경
upstream 파일을 대조했다. metrics의 버퍼 증가는 파일 선택을 바꾸지 않으며 동일 입력 분석도
같았다. CI에는 실제 소비 회귀·metrics 실행만 추가했고 권한·trigger·required check는 유지했다.

## 공통 조판 원칙·입력 확인

| 항목 | 판정·실행 증거 |
| --- | --- |
| 원인 계층·일반성 | 충족: 입력 capability·저장 대상·상위 의존 계약의 보정이다. 문서 ID·고정 px 예외 없음. |
| 생산·측정·실제 배치 | 기존 공통 IR·Canvas 경로 재사용. `convertToEditable` 전후 12쪽 SVG가 동일했다. 새 높이·원점·줄 구성 생산/덮어쓰기 없음. |
| 분할·이어받기·저장 LineSeg | 비해당: parser/lowering·pagination·paint·LineSeg 수용 조건을 변경하지 않았다. |
| 실제 backend | 충족: Native Skia와 fresh Docker WASM 전쪽 비교 및 실제 Studio Canvas를 각각 검증했다. 기존 형식 on/off 5쪽 SVG도 동일했다. |
| 신규 회귀·기준값 | UI·의존 계약 검사는 기존 코드에서 각각 4·7 FAIL → 수정 후 PASS. 조판 fixture/golden·px 핀·baseline·허용치의 추가/완화 없음. 해당 PDF를 사용하는 기존 절대 위치 핀도 검색되지 않았다. |
| 주장·검사 의미 | 첫 저장 분리·원본 쓰기 금지·실패 복구·실제 파일 선택/URI 의미를 검사했다. 쪽수·해시만으로 배치 정확성을 대신하지 않았다. |
| 입력 커밋 확인 | 충족: 아래 네 파일의 실행 바이트와 검토 code head의 Git blob SHA-256이 같았다. |

| 저장소 입력 경로 | 출처·역할 / SHA-256 |
| --- | --- |
| `pdf/issue5875/nested_table_text_caption-2020.pdf` | 기존 한컴 Print, 직접 PDF 입력·독립 raster 기준. 12쪽·525,733 bytes, Hwp 2020 / Hancom PDF 1.3.0.550. `6308671fc56b1bfd9e72660edd6edf574a4e16f75e84e0bf35399339d9ce3702` |
| `samples/hwp_table_test.hwp` | 기존 HWP 대조군 3쪽. `dc3e57d7577447ae4b47f59bdb0f499a5ac29066b62af8a650bcdd8d70eee32a` |
| `samples/hwpx/footnote-tbox-01.hwpx` | 기존 HWPX 대조군 1쪽. `e7e6d109beee222f01f089b014c42d8d0f540564b57c7dd47384c1b5ae19e2f7` |
| `samples/hml/formatting_table.hml` | 기존 HML 대조군 1쪽. `177b93de7c79462bef7850e15b018dbc8138fbe713b9c7e045b6125cc7527d35` |

## 검증과 직접 판독

[1단계](../../working/task_m100_7946_stage1.md)의 clean 재설치·Node 24.15.0/22.18.0·URI 타입/소비
계약, [2단계](../../working/task_m100_7946_stage2.md)의 제품 호출 경로·실제 Docker/Studio 검사,
[3단계](../../working/task_m100_7946_stage3.md)의 최종 전체 회귀와 devel 전쪽 대조를 실제 코드·로그와 확인했다.

- 전체 nextest **10,435 PASS·0 FAIL·45 skip**, Native Skia library **4,043 PASS·12 ignored**, 그림 2·직접 PDF 4 PASS, PDF-on 집중 130 PASS.
- fmt·Native/WASM/workspace Clippy·workspace build·정책, Studio **2,021 PASS·2 skip**·타입·PWA build, VS Code compile·VSIX package가 성공했다.
- 의존성 7 PASS, 실제 Worker **85 assertion·5 Worker**, 실제 Studio **15항목**과 HWP/HWPX/HML 문서 교체가 통과했다. 자동 native picker/출력 handle 대조군과 실제 앱/Canvas 실행을 구분했다.
- 제품 source·test가 같으므로 완료된 전체 회귀·Docker WASM을 반복하지 않았다. 생성 suite·로그·TSV·private font는 커밋하지 않았다.

원본 PDF·Print profile·compat 2022·96 DPI·실제 HANBatang/HANBatangB 공급을 고정했다.
독립 Print와 1–12쪽을 비교한 최저값은 Native **90.14609%**, fresh WASM **90.16832%**,
미달·누락은 양쪽 0이며 모두 12쪽이었다. 고정 devel과 같은 조건의 24개 backend/page 쌍에서
최대 하락 **0.00000%p**, 0.01%p 이상 하락 0쪽이었다. 각 backend의 전쪽 PNG도 기준/후보가 같았다.

1쪽 표 외곽·본문/그림 순서, 3쪽 오른쪽 crop·캡션·다음 행, 11쪽 그림 9/10·캡션과 간격을
직접 판독했고 사용자가 실제 Studio에서 시각 통과로 판정했다. 문자 간격·raster 차이는 남으며
모든 renderer의 픽셀 동등성이나 B10-1 저장본 한컴 재조판의 해결을 주장하지 않는다.
[대표 PNG 8개](../assets/issue_7946_studio_pdf/)를 code head와 바이트 대조했고 PR 본문에서
실제 이미지가 모두 표시됨을 Chrome으로 확인했다. 전쪽 TSV 원문 네 개도 본문에 게시했다.
후행 문서 commit 이후에도 동일 source의 검증을 재사용하고 이미지 URL만 해당 head로 고정한다.

원 로그는 ignored `output/pr-review/issue7946/`에, 최종 입력/producer/Decimal 비교와 공개 확인은
`pr-preparation/input-commit-check.json`, `devel-comparison.json`, `published-verification.json`에 보존했다.
기준 Docker cache의 잘못된 첫 실행은 제외했고 실제 source/API가 일치하는 수정 실행만 사용했다.

## 정확한 code head의 GitHub Actions

2026-10-16 17:59:59 KST, `432ba702f743ae6468f12120af90abbc630aa3e1`의 **35 check = 31 success·4 정책상 skipped**,
실패·대기 0과 별도 `CI Impact Policy` success를 확인했다.

| 실행 | 실제 결과 |
| --- | --- |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/38875285233) | archive A–D 빌드·회귀, Native Skia, lint, frontend package 및 Build & Test 성공 |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38875285162) | Actions·JavaScript/TypeScript·Python·Rust Analyze 성공, 동일 head의 GHAS CodeQL도 success |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38875285439) | Canvas visual diff 성공 |
| [Adapter](https://github.com/edwardkim/rhwp/actions/runs/38875285180) / [Proptest](https://github.com/edwardkim/rhwp/actions/runs/38875285226) | 모두 성공 |
| [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/38877330688) | exact head의 full 실행 감사 success |

현재 보호 설정의 required check는 Build & Test, strict=false였다. 위 범위상 필요한 실행도 전부
확인했으며 required 한 개의 녹색만으로 나머지 검증을 생략하지 않았다. 정책상 skipped와
로컬 PDF-on fresh Docker WASM의 실제 실행 증거는 별개다. 원 응답은 `pr-preparation/code-head-ci-success.json`이다.

## 남은 범위와 병합 조건

기본 PDF 비활성 및 `parserAllocationBoundsProven=false`를 유지한다. #7911의 B01–B07·B09·
B10-1, OCR·완전한 문단 복원·전체 Worker 편집기·npm/브라우저 확장 배포는 후속이다.
audit의 기존 browserslist·baseline-browser-mapping·Studio @babel/core 발견은 1단계에 분리했다.
대상 두 패키지 보정 통과를 전체 의존성 무취약으로 표현하지 않는다.

이 review와 [오늘할일](../../orders/20261016.md)만 single-parent trailing commit으로 추가한다.
push 전에 최신 base/head의 merge tree에서 공백·문서 링크·기존 오늘 기록 보존을 검사한다.
사용자의 “이제 머저하고 close 절차를 진행합니다” 지시로 병합과 후속 처리를 승인받았다.
최신 devel `cdef0073bb24512daf0622dec5051ac62cb8d57e`와의 오늘할일 EOF 충돌은 이번
PR section만 공통 내부 경계로 옮겨 해소한다. 최신 base의 타 PR 기록 보존·링크·공백을 실제
merge tree에서 확인한다. 제품 code·test·workflow·PNG는 변경하지 않는다.
마지막 trailing head의 실제 fast-pass·required checks·mergeability를 확인한 뒤 일반 squash merge한다.
검증된 code head CI 성공을 아직 실행 전인 trailing head CI 성공으로 표시하지 않는다.

### Merge 후 PR comment 계획

병합 승인을 받은 뒤 최종 merge SHA·CI URL과 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md),
실제 12쪽 결과·직접 판정·남은 범위를 한국어로 남긴다. 대표 이미지는
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7946_studio_pdf/`
아래 동일 PNG로 고정하고 `--body-file` 게시 뒤 API로 확인한다. 현재 댓글 게시·merge·이슈 종료는 수행 전이다.
