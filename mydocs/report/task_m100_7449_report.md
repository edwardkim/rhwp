# 선택 실행 증적 보고서 구현 결과

Issue: #7449

최신 upstream/devel `def947c2149df5bc812d801e6e79be19a403b4d1`에서 독립
`codex/issue-7449-e2e-report` 브랜치로 시작했다. 원격 조회 기준도 같은 SHA였다.
본인 기존 열린 PR은 없었으며 #7449에 연결된 기존 구현 PR도 없었다. #3512 종료·#7283 병합과
자동 CI 철회 합의를 확인했다. 다른 이슈의 구현·미병합 변경을 포함하지 않았다.

## 원인과 변경 동작

기존 결과는 사람이 JSON·suite 로그·실패 JSON/PNG를 따로 열어야 했고 파일의 실행 소속을
검증할 manifest가 없었다. 실행기에 runId와 증적 SHA-256, 로그 머리말의 run/suite 식별,
진단 JSON runId를 추가했다. 결과 JSON 갱신은 원자적 rename을 쓴다.

독립 보고서 CLI는 기존 실행 결과만 읽는다. 원본 실행 상태와 보고서 증적 완전성을 구분하며,
legacy v1·누락·변조·다른 실행·미완료를 pass로 승격하지 않는다. 후보 inventory 해시와 도구 SHA,
환경·시각·suite 결과를 표시한다. 후보 빌드 SHA는 미관측이며 검사 도구 SHA와 다르다.
독립 SHA 기대값 옵션과 source 해시에 묶은 수동 확인 기록을 지원한다.

공유 요약(report.html/report.json)은 allowlist 필드만 포함하고 원본 로그·JSON·PNG·절대 경로·
다운로드 정보를 복사하지 않는다. 비공개 private.html은 검증된 원본의 상대 링크만 제공한다.
원문 문서와 사용자 프로필은 수집하지 않는다. 서명·출처 인증이나 자동 게시·배포 기능은 없다.

## 검증과 수정 전후 대조

| 명령·대상 | 실제 결과 |
| --- | --- |
| `npm --prefix rhwp-chrome ci --no-audit` | 44개 설치. 누락 의존성으로 실패한 첫 page-budget 검사는 설치 후 재실행 |
| `node --test rhwp-chrome/e2e/report.test.mjs rhwp-chrome/e2e/run.test.mjs rhwp-chrome/e2e/tab-monitor.test.mjs rhwp-chrome/e2e/failure-diagnostics.test.mjs rhwp-chrome/e2e/page-budget.test.mjs` | 32 PASS / 0 FAIL / 0 SKIP |
| 같은 run.test의 실행 소속 assertion을 base run.mjs에 적용 | runId undefined로 1 FAIL. 수정 후 같은 assertion PASS |
| `node rhwp-chrome/e2e/report-fixtures.mjs --output output/pr-review/7449/examples-final` | pass/fail/error 보존, fail 뒤 not-run, running과 로그 누락은 incomplete |
| 같은 report CLI를 같은 입력에 두 번 실행 | report.html/report.json 바이트 동일, pass exit 0·fail/incomplete exit 1 |
| schemaVersion·후보 inventory·도구 SHA·독립 기대값, 누락 suite/log/PNG, 잘못된 JSON, hash/runId, symlink·혼입·손상 메타데이터 | 계약 검사에서 모두 확인. 실패/누락을 성공으로 기록하지 않음 |
| 공유 요약의 경로·다운로드 ID·원문·임의 텍스트 제외, source 해시가 다른 수동 기록 거부 | 계약 검사 통과 |
| Chromium 151 HTML 표시 | 1200px/390px 가로 overflow 없음. 실패 표·단계·미확인·제외 정보 직접 확인 |
| `git diff --check`와 변경 문서 링크 검사 | 통과 |

초기 새 CLI 검사 실패는 fixture 환경 객체 공유로 인한 테스트 오염이었다. 객체 복사로 수정하고
전체를 재실행했다. 로그는 ignored `output/pr-review/7449/`에 보존했다. 합성 예제의 SHA·버전·
화면은 실제 브라우저 실행 증거가 아니다. 검증한 code SHA는 `e22fda49fab94fd6bdb64092aed870912ffff9fd`다.
[PR #8178](https://github.com/edwardkim/rhwp/pull/8178)과
[self-review](../pr/archives/pr_8178_review.md)에 연결했다.

## 미검증·비해당

Chrome for Testing 설치는 `storage.googleapis.com` EAI_AGAIN으로 실패했다. 설치된 Chromium으로
HTML만 확인했으며, 정책으로 막힌 file URL 대신 동일 HTML의 setContent를 썼다. 실제 확장 전체
E2E를 재실행하지 않았다. 스토어 설치·업데이트, Edge/Firefox, 사용자 프로필·실제 문서 표시·인쇄
품질과 배포 판단은 검증 범위가 아니다. 실행기/보고서 계약과 구분한다.

Rust·Studio·렌더·빌드·workflow 변경이 없으므로 Cargo·fresh WASM·Visual Sweep 비해당.
HWP/HWPX/PDF 파일을 검사에 사용하지 않았다. GitHub CI 완료는 로컬 검사로 주장하지 않는다.
사용자가 요청한 보고서 읽기 편의성·공유 전 제외 결과 판정은 미확인이다.

## 사용자가 직접 확인할 절차

[매뉴얼 3.10](../manual/chrome_edge_extension_build_deploy.md#310-선택-실행-증적-보고서-7449)의
동일 명령으로 예제를 생성하고 각 상태의 report.html을 비교한다. fail의 뒤 두 suite가 not-run인지,
원본 pass에서 로그가 빠진 예제가 incomplete인지 확인한다. private.html에서 원본 단계·로그·PNG를
대조하되 이를 공개하지 않는다. 공유할 두 파일에서 경로·다운로드·개인 정보 제외와 읽기 편의성을
검토한다. 수동 JSON의 pass/fail을 실제 판정 후 작성하고 `--manual-review`로 새 보고서를 생성한다.
두 공유 파일만 사람이 게시한다. 새 이슈·자동 CI·merge·release·배포·issue close는 수행하지 않는다.

## 재검토 보정

기존 head `8a478c7562354da86e3a1797c7b288bc656fdb74`의 재검토에서 확인한 네 항목을
[보정 기록](../working/task_m100_7449_stage2.md)에 따라 수정했다.

| 항목 | 수정 전 | 보정 후 |
| --- | --- | --- |
| 서명만 남은 PNG·IEND 누락·끝의 불필요 데이터 | verified | invalid-png / incomplete |
| 전체 오류 필드가 있는 pass | pass | inconsistent-overall-pass / incomplete |
| 필수 후보 파일 누락과 재계산한 해시 | pass | candidate-required-files-missing / incomplete |
| 대문자·혼합 script 태그에 대한 음성 대조 | 검출 실패 | 검사 통과, 실제 DOM에 능동 콘텐츠 없음 |

보정 전 회귀는 32 PASS / 4 FAIL / SKIP 0이었다. 보정 후 아래 명령에서 계약 39개와
Chromium DOM 검사 1개가 모두 통과했다. 계약은 Node 24.19.0과 Node 22.23.4에서 각각 재검증했다. 기존 32개 결과와 구분하며, 새 code SHA와 원격 CodeQL
결과는 PR 본문에 고정한다.

```bash
node --test rhwp-chrome/e2e/report.test.mjs rhwp-chrome/e2e/png-integrity.test.mjs \
  rhwp-chrome/e2e/run.test.mjs rhwp-chrome/e2e/tab-monitor.test.mjs \
  rhwp-chrome/e2e/failure-diagnostics.test.mjs rhwp-chrome/e2e/page-budget.test.mjs
RHWP_REPORT_BROWSER=/usr/bin/chromium node --test rhwp-chrome/e2e/report-browser.test.mjs
node rhwp-chrome/e2e/report-fixtures.mjs --output output/pr-review/7449/corrected-examples
```

PNG 구조·CRC·압축 스캔라인 검사에는 외부 의존성이 없다. 압축 해제 데이터는 128 MiB로 제한하며,
초과는 incomplete다. 픽셀 내용의 시각적 정확성은 판정하지 않는다. 실제 확장 전체 E2E와 사용자
수동 확인의 미검증 범위는 위와 같다.
