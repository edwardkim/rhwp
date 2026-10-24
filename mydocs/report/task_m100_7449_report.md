# 선택 실행 증적 보고서 구현 결과

Issue: #7449 · PR: https://github.com/edwardkim/rhwp/pull/8178

최초 upstream/devel `def947c2149df5bc812d801e6e79be19a403b4d1`에서 독립
`codex/issue-7449-e2e-report` 브랜치로 시작했다. 최종 code candidate는
`393a59dc5e3991a1b90ac15589322eab53e90b8b`, 최신 병합 비교 base는
`3536e72819e165dbe959ee42cfd4e4cf8db33ce7`다. #3512/#7283의 병합 결과를 소비하며
다른 이슈의 구현·자동 확장 E2E CI·배포 기능은 추가하지 않았다.

## 원인과 변경 동작

기존 E2E 결과는 JSON·suite 로그·실패 JSON/PNG를 따로 열어야 했고 파일의 실행 소속을 확인할
manifest가 없었다. 실행기에 runId·파일 SHA-256·로그 식별 머리말·진단 runId와 원자적 JSON 갱신을
추가했다. 보고서 생성기는 선택한 기존 결과만 읽으며, 원본 실행 상태와 증적 완전성을 구분한다.
누락·변조·혼입·파싱 오류·지원하지 않는 스키마·미완료를 성공으로 바꾸지 않는다.

공유 report.html/report.json은 allowlist 요약만 포함한다. 원본 절대 경로·다운로드 정보·로그·
진단·PNG·자유 텍스트는 제외하고 private.html에 검증된 원본의 상대 링크를 제공한다.
후보 inventory·도구 SHA·독립 기대값을 대조하며 후보 빌드 SHA는 미관측으로 구분한다.
해시는 내부 일관성 근거이며 출처 인증이 아니다. 원문 문서·사용자 프로필·원본 증적을 공유본에 복사하지 않는다.

사용자에게 제시한 초기 HTML은 긴 메타데이터 JSON이 결과를 가렸다. 가독성 판단을 위임받은 뒤
상태 → 확인할 문제 → 검사별 결과 순서로 바꾸고, 한국어 오류 설명·접을 수 있는 추적 상세·
모바일 세로 표를 제공했다. 시간 초과를 관측하지 못한 초기 오류도 없음 대신 미관측으로 표시한다.
수동 기록은 reviewMode=direct/delegated를 구분하고, 생략한 기존 기록은 방식 미기록으로 읽는다.
Chrome 값은 설치 대상 버전임을 HTML과 JSON에 명시한다.

## 검증과 수정 전후 재현

| 명령·대상 | 실제 결과 |
| --- | --- |
| Node 24.19.0 계약 및 Chromium HTML 검사 | 41 PASS / 0 FAIL / 0 SKIP |
| Node 22.23.4 계약 | 40 PASS / 0 FAIL / 0 SKIP |
| runId 실행 소속 assertion | 최초 base에서 undefined로 FAIL, 구현 후 PASS |
| PNG·모순된 pass·필수 후보 파일 경계 | 보정 전 4 FAIL, 보정 후 PASS |
| 파싱 오류 결과의 시간 초과 표시 | 보정 중 없음 표기를 assertion FAIL로 재현, 미관측 보정 후 PASS |
| pass/fail/error/not-run/incomplete 합성 예제 | 원본 상태 보존, 누락·미완료는 incomplete |
| schemaVersion·후보/도구 SHA·기대값·누락/변조/다른 run·symlink | 계약 검사 통과 |
| PNG 청크·CRC·압축 스캔라인·팔레트·Adam7·128 MiB 제한 | 계약 검사 통과 |
| 공유 제외와 소스에 묶인 direct/delegated 수동 기록 | 계약 검사 통과, 실패 상태 승격 없음 |
| CSP 비의존 DOM/escaping 및 1200px·390px 표시 | 대소문자/혼합 태그·이벤트 실행 없음, 5가지 상태의 상세 접기/펼치기에서 가로 넘침 없음 |
| 같은 실제 원본·CLI로 HTML/JSON 생성 | 바이트 동일, fail/incomplete는 exit 1 |

최초 계약 32개와 이전 재검토 계약 39개는 과거 결과다. 현재는 계약 40개와 HTML 검사 1개로
집계한다. 원인·전후 결과는 [이전 보정](../working/task_m100_7449_stage2.md)과
[가독성·위임 판정 보정](../working/task_m100_7449_stage3.md)에 기록했다.

## 실제 확장 E2E 실행 시도

2026-10-24 12:31 KST에 실제 후보를 공식 실행기로 검사했다. 합성 runner를 사용하지 않았다.

- 후보: 0.8.7 / inventory SHA-256 `8ccccb9e9ee3a802f6436b7fe0615bc1af93ada2d0039c38c58f2d0ffe600295`.
- 원본 실행 도구: `e32f63d2aac839190d2b8085f0ec5346fa4e0479`, dirty=false.
- 원본 SHA-256: `fabdb4c1b313eed53cf752dd583b91ae022dd0dab3cce5e966ea8c6d15d23609`.
- 결과: extension-smoke는 Chrome for Testing 미설치로 launch 단계 FAIL, 뒤 두 검사는 not-run.
  원본 전체 fail / CLI exit 1이다.
- 증적: 로그·진단 JSON의 runId·해시는 확인했다. 브라우저가 뜨지 않아 PNG가 없어 보고서는
  failure-png-missing / incomplete / exit 1이다. 다른 이미지를 실제 화면 대신 넣지 않았다.
- 환경: 지정 프록시에서 storage.googleapis.com 다운로드 HEAD는 403 Forbidden이었다.
  시스템 Chromium은 확장 설치 금지 관리 정책이 있어 대체하지 않았다.
- 후보 WASM은 기존 pkg를 사용했으며 이번에 fresh 빌드하지 않았고 빌드 SHA는 미관측이다.
  원본 environment.chrome은 설치 대상 기록이며 실제 브라우저 실행 관측값이 아니다.

따라서 실제 확장 전체 E2E는 완료하지 못했다. HTML DOM 검사나 합성 예제를 이 증거로 대신하지 않는다.

## 사용자 위임에 따른 수동 판정과 공유 경계

사용자는 가독성의 목적 적합성과 정보 제외의 적절성을 에이전트에게 위임했다. 보정 후 실제 보고서를
데스크톱·모바일에서 직접 판독하고 원본과 공유 HTML/JSON을 대조해 **readability=pass,
sharingExclusions=pass, reviewMode=delegated**로 기록했다. 사용자 본인이 직접 판정했다고 쓰지 않는다.
기록 시각은 `2026-10-24T03:40:22.789Z`이며 위 원본 SHA에 묶여 있다.

판정 근거는 실패 상태·증적 불완전·launch 단계·미실행·PNG 누락을 먼저 읽을 수 있다는 점과,
원본 경로·로그/stack·진단 내용·다운로드 정보가 제외됐다는 점이다. SHA·버전·시각은 결과 추적에
필요한 공유 범위로 판단했다. 위임 통과 뒤에도 자동 상태 fail/incomplete/not-run은 유지했다.

산출물은 ignored `output/pr-review/7449/user-review-20261024/`에 보존했다.
공유 ZIP에는 final/shared/report.html과 report.json만 담고, 비공개 ZIP에는 원본과 private.html을
상대 위치 그대로 담았다. 이 세션에는 attach_artifact 도구가 없어 파일 링크와 화면으로 제시했다.
검토를 통과한 두 공유 파일·ZIP과 보고서 화면은 [공유 증적](assets/issue7449/README.md)에도
포함해 GitHub에서 다운로드할 수 있게 했다. 비공개 원본·인덱스는 저장소에 게시하지 않았다.

- 최종 공유 HTML SHA-256: `0dd238562ea03e7f8b20b08336ed3d30e1ba26bb3759128938c13f8e23aa5a06`.
- 최종 공유 JSON SHA-256: `a7d15ce9c66e48fcc7330093d1e892e25787beaeffc1502c4637fda9fcf65fda`.

## 실행 명령과 원격 검증

```bash
node --test rhwp-chrome/e2e/report.test.mjs rhwp-chrome/e2e/png-integrity.test.mjs \
  rhwp-chrome/e2e/run.test.mjs rhwp-chrome/e2e/tab-monitor.test.mjs \
  rhwp-chrome/e2e/failure-diagnostics.test.mjs rhwp-chrome/e2e/page-budget.test.mjs
RHWP_REPORT_BROWSER=/usr/bin/chromium node --test rhwp-chrome/e2e/report-browser.test.mjs
git diff --check
```

실제 후보 실행·해시 기대값·수동 JSON·재생성 명령은 위 stage3 기록과
[매뉴얼 3.10](../manual/chrome_edge_extension_build_deploy.md#310-선택-실행-증적-보고서-7449)을 따른다.
출력은 매번 새 폴더를 지정한다. private 인덱스를 열려면 비공개 ZIP의 폴더 구조를 유지한다.
공유하는 자료는 검토한 두 요약뿐이다.

code candidate `393a59dc5e3991a1b90ac15589322eab53e90b8b`의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/40060079315/job/120773848041), Adapter inter-diff, Proptest roundtrip과 CodeQL workflow는 성공했다. 원격 체크는 **18 success / 16 skipped / 1 neutral**, 실패·대기 0개다.

[JavaScript CodeQL 분석](https://github.com/edwardkim/rhwp/actions/runs/40060079220/job/120771522804)은 성공했다. [통합 CodeQL 판정](https://github.com/edwardkim/rhwp/runs/120772313762)은 base의 Python·Rust 분석 구성이 이번 실행에 없어 전체 경고 비교를 확정할 수 없다는 이유로 neutral이다. 전체 언어 보안 검사 통과로 해석하지 않는다. 기존 #283 리뷰 스레드는 GitHub Advanced Security 봇이 해결한 상태다.

최신 base와 code candidate의 merge-tree exit 0, merge tree 공백·문서 링크·base 오늘할일 보존을
확인했다. review 문서-only trailing head의 CI와 mergeability는 push 후 별도로 확인한다.

## 로컬 실제 E2E 완료와 최종 검토

2026-10-24 로컬 macOS arm64에서 Chrome for Testing 155.0.8059.79를 사용해 전체 확장 E2E를
완료했다. 검증 source SHA는 `955fa1125fce67b9a5ad8db09ed8780a9eb1caab`이며 fresh release
WASM·wasm-opt와 확장 빌드를 마친 후보를 실제 실행했다. smoke 2.425초, download 15.012초,
lifecycle 34.116초 모두 PASS, 첫 공식 실행 1회·exit 0·재시도/timeout/not-run 0이다.
원본 결과에서 최신 보고서를 생성한 결과도 pass/issues 0/exit 0이다.

[공유 HTML·JSON·ZIP, 1200px·390px 화면과 해시·명령](assets/issue7449/local-e2e-20261024/README.md)에
증거를 모았다. 후보 해시가 검사 전후 동일하고 실행기 dirty=false다. 새 원본 해시에 묶인
delegated 가독성/공유 제외 판정도 pass/pass다. 이전 클라우드 실패 이력은 그대로 보존한다.
[최종 검토](../pr/archives/pr_8178_review.md)는 승인이다. 사용자가 최신 CI 확인 후 merge까지
승인했으며, 이 문서-only 후행 commit의 CI는 push 후 별도로 확인한다.

## 미검증·비해당

Rust·Studio·제품 렌더·워크플로우를 바꾸지 않아 Rust lint·독립 한컴 PDF Visual Sweep은
이번 변경의 게이트로 비해당이다. 실제 E2E용 fresh WASM은 별도로 빌드했다.
print surface의 통과를 문서 출력 품질 검증으로 확대하지 않는다. 스토어 설치/업데이트,
Edge/Firefox, 실제 사용자 프로필, 문서 표시·인쇄 품질은 미검증이다. 운영 배포는 하지 않는다.
