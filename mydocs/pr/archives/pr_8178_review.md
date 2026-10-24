# PR #8178 self-review — 선택 실행 증적 보고서

## 최종 판정

**승인 — 로컬 실제 확장 E2E 통과로 기존 보류 사유를 해소했다.**
검토·실행 소스는 `955fa1125fce67b9a5ad8db09ed8780a9eb1caab`다. 아래 기존 클라우드 실패는
이력으로 보존하며, 최신 성공은 [로컬 공유 증적](../../report/assets/issue7449/local-e2e-20261024/README.md)에 연결한다.
새 actionable code finding은 없다. 같은 head의 계약 40 PASS·Chrome HTML 1 PASS를 재사용했다.
실제 Chrome for Testing 155.0.8059.79의 smoke·download·lifecycle 3개 모두 최초 실행에 PASS,
exit 0·timeout 0·not-run 0이다. 정확한 소스에서 release 최적화·wasm-opt를 포함해 fresh WASM과
확장 후보를 빌드했고 후보 해시가 검사 전후 동일했다. 실행기 dirty=false다.

가독성과 공유 정보 제외는 새 원본 해시에 묶인 delegated pass/pass다. 사용자 본인의 판정이나
자동 인증으로 기록하지 않는다. 1200px·390px 최종 보고서 화면도 직접 확인했다.
[리뷰 코멘트](https://github.com/edwardkim/rhwp/pull/8178#issuecomment-6380534004)를 게시했다.

사용자는 같은 브랜치의 문서·증적 반영, 최신 CI 확인 후 merge를 명시적으로 승인했다.
현재 검증 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/40061085074)는 성공이며,
preflight가 code candidate `393a59dc5e3991a1b90ac15589322eab53e90b8b` 성공을 재사용했다.
이 후행 문서 commit은 제품 코드·테스트를 바꾸지 않는다. 새 head CI·정책·mergeability를
확인한 뒤 일반 squash merge하며 self-APPROVE·admin bypass·운영 배포는 하지 않는다.

## 접수와 범위

- PR: https://github.com/edwardkim/rhwp/pull/8178 · Issue: #7449
- 작성자: postmelee. 본인 self-review이며 현재 보정은 사용자의 명시 요청 범위다.
- 최초 분기 base: devel `def947c2149df5bc812d801e6e79be19a403b4d1`.
- 최신 비교 base: `3536e72819e165dbe959ee42cfd4e4cf8db33ce7`.
- 재검토 전 code head: `e32f63d2aac839190d2b8085f0ec5346fa4e0479`.
- 검증한 최종 code candidate: `393a59dc5e3991a1b90ac15589322eab53e90b8b`.
- base route: collaborator_self_merge. modifiers: intake_and_review, local_validation,
  rework_and_exceptions, review_only_fast_pass.
- loaded documents: AGENTS.md, CLAUDE.md, Codex MEMORY.md, mydocs README/manual 지도,
  문서·Git 절차, PR review 라우터와 위 기본·보조 문서, CONTRIBUTING.md.
- #3512/#7283의 병합 결과를 소비하는 로컬 보고서 도구다. 다른 이슈의 구현과 자동 E2E CI를 추가하지 않았다.

## 보정과 직접 검증

[결과보고](../../report/task_m100_7449_report.md),
[가독성·위임 판정 보정 기록](../../working/task_m100_7449_stage3.md),
[사용 매뉴얼](../../manual/chrome_edge_extension_build_deploy.md#310-선택-실행-증적-보고서-7449)에
명령·증적·공유 범위·미검증을 연결했다.

- 긴 메타데이터 JSON이 결과를 가리는 문제를 확인하고 상태 → 문제 → 검사별 결과 순서로 바꿨다.
  SHA·환경은 상세로 접고, 오류 코드에 한국어 설명을 붙였다. 모바일에서는 검사 표를 세로로 읽는다.
- 파싱 실패의 미관측 시간 초과를 없음으로 표시하던 보정 중 경계는 assertion FAIL로 재현하고 수정했다.
- Node 24.19.0 계약 40개 및 Chromium 151 HTML DOM 검사 1개: **41 PASS / 0 FAIL / 0 SKIP**.
- Node 22.23.4 계약: **40 PASS / 0 FAIL / 0 SKIP**.
- 5가지 합성 상태, schemaVersion·후보 inventory·도구 SHA·기대 해시, 로그/JSON/PNG의 누락·변조·혼입,
  잘린 PNG와 CRC/압축 오류, symlink·불완전 결과, 공유 정보 제외·소스에 묶인 수동 기록을 확인했다.
- 대소문자·혼합 태그·이벤트 입력에서 CSP 없이도 능동 DOM과 실행이 없었다. 1200px·390px에서
  5가지 상태의 상세를 접고 펼쳐도 가로 넘침이 없고, 데스크톱 첫 화면에 상태와 검사 표 시작이 보였다.
- 실제 실행 시도의 동일 원본·동일 CLI로 HTML/JSON 바이트 재현을 확인했다. 위임 수동 pass/pass 뒤에도
  원본 fail / 보고서 incomplete / 후속 not-run 및 CLI exit 1을 유지했다.
- 보정 전 PNG·모순된 pass·필수 후보 파일·CodeQL #283의 이력은
  [이전 보정 기록](../../working/task_m100_7449_stage2.md)에 보존했다.
- 다른 세션에서 e32f63d2a의 계약 39 PASS와 Chrome 155 HTML 검사 1 PASS가 전달됐다.
  이는 이전 head의 참고 결과이며 본 세션의 최신 코드 직접 검증과 구분했다.

## 이전 클라우드 실제 확장 E2E 시도와 환경 제한 (이력)

- 실제 후보: 버전 0.8.7, inventory SHA-256
  `8ccccb9e9ee3a802f6436b7fe0615bc1af93ada2d0039c38c58f2d0ffe600295`.
- 원본 실행 도구: `e32f63d2aac839190d2b8085f0ec5346fa4e0479`, dirty=false.
- 원본 result.json SHA-256: `fabdb4c1b313eed53cf752dd583b91ae022dd0dab3cce5e966ea8c6d15d23609`.
- 2026-10-24 12:31 KST 공식 실행은 exit 1이었다. extension-smoke는 Chrome for Testing 미설치로 launch
  단계에서 실패했고, download-interceptor와 extension-lifecycle은 not-run이다.
- 브라우저가 뜨지 않아 PNG가 없다. 보고서는 failure-png-missing / incomplete / exit 1이며,
  로그·진단 JSON만 같은 runId와 해시를 확인했다. 실제 확장 동작 성공으로 판정하지 않는다.
- 지정된 환경 프록시에서 storage.googleapis.com 다운로드 HEAD는 403 Forbidden이었다.
  시스템 Chromium은 관리 정책으로 확장 설치가 금지돼 있어 대체하지 않았다.
- 후보 WASM은 준비된 pkg이며 이번에 fresh 빌드하지 않았고 그 빌드 SHA는 미관측이다.
  원본의 Chrome 값은 설치 대상 기록이다. HTML과 JSON 모두 실제 브라우저 실행 증거가 아님을 표시한다.

## 위임 수동 판정과 산출물 경계

- 사용자가 두 판정의 판단을 에이전트에게 위임한 뒤, 실제 원본에 연결된 보정 화면과 공유 JSON을 검토했다.
- **가독성 pass**: 원본 실패·증적 불완전·launch 단계·미실행·PNG 누락 사유가 먼저 보이며,
  상세의 추적값을 펼쳐 읽을 수 있다. 데스크톱·모바일 화면도 직접 판독했다.
- **공유 정보 제외 pass**: 원본과 대조해 후보/브라우저 절대 경로·원문 로그/stack·진단 내용·다운로드
  정보가 공유 HTML/JSON에 없음을 확인했다. SHA·버전·시각은 이 도구의 추적 목적상 적절한 공유 범위로 판단했다.
- 수동 기록의 checkedAt은 `2026-10-24T03:40:22.789Z`, reviewMode는 delegated다.
  기록은 위 원본 SHA에 묶여 있다. 사용자 본인의 직접 판정이나 자동 인증을 주장하지 않는다.
- 최종 공유 HTML SHA-256: `0dd238562ea03e7f8b20b08336ed3d30e1ba26bb3759128938c13f8e23aa5a06`.
- 최종 공유 JSON SHA-256: `a7d15ce9c66e48fcc7330093d1e892e25787beaeffc1502c4637fda9fcf65fda`.
- ignored output의 공유 ZIP은 두 요약만 담는다. 비공개 ZIP은 원본 결과·로그·진단과 private.html을
  상대 위치 그대로 담으며 공개 대상에서 제외했다. 검토한 공유본과 보고서 화면만
  [공유 증적](../../report/assets/issue7449/README.md)에도 포함해 GitHub에서 다운로드할 수 있게 했다.
  비공개 원본은 작업 세션에만 보존한다.

## 원격 검사와 병합 시뮬레이션

code candidate `393a59dc5e3991a1b90ac15589322eab53e90b8b`의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/40060079315/job/120773848041), Adapter inter-diff, Proptest roundtrip과 CodeQL workflow는 성공했다. 원격 체크는 **18 success / 16 skipped / 1 neutral**, 실패·대기 0개다.

[JavaScript CodeQL 분석](https://github.com/edwardkim/rhwp/actions/runs/40060079220/job/120771522804)은 성공했다. [통합 CodeQL 판정](https://github.com/edwardkim/rhwp/runs/120772313762)은 base의 Python·Rust 분석 구성이 이번 실행에 없어 전체 경고 비교를 확정할 수 없다는 이유로 neutral이다. 전체 언어 보안 검사 통과로 해석하지 않는다. 기존 #283 리뷰 스레드는 GitHub Advanced Security 봇이 해결한 상태다.

code candidate와 최신 비교 base의 merge-tree는 exit 0이며 tree는
`60d386a029d8c5d8bae398cff69cbc6f532f8672`다. merge tree의 공백·변경 Markdown 링크와
최신 base 오늘할일 바이트 보존을 확인했다. 문서 trailing head는 push 전 같은 검사를 다시 수행하고
최신 CI·mergeability를 별도로 확인한다. 이 문서가 아직 실행 전인 trailing head CI를 보증하지 않는다.

## 공통 적용 여부와 남은 범위

- 조판 원칙: 비해당. 보고서 표시·수동 기록·관련 테스트만 바꾸며 제품 렌더·조판·baseline을 바꾸지 않았다.
- 검증 입력 커밋 확인: 실제 smoke 서버가 읽은 기존 samples/hwp3-pagedef-1915.hwp는 HEAD와 동일하다.
  SHA-256은 `b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0`다.
  이전 클라우드 시도는 브라우저 실행 전 중단됐다. 로컬에서는 두 실제 fixture와 실행을 확인했다.
  합성 계약은 원문 입력을 사용하지 않으며, 문서의 독립 한컴 PDF 시각 일치는 이번 검사의 범위가 아니다.
- Cargo·fresh WASM·Native/Visual Sweep: 이번 소스 변경의 게이트로는 비해당. 실제 후보 WASM의 출처
  한계는 과거 클라우드 이력이다. 로컬 fresh WASM 빌드와 전체 확장 E2E는 완료했으며
  제품 조판 변경이 없어 독립 PDF Visual Sweep은 비해당이다.
- 스토어·Edge/Firefox·실제 사용자 환경·문서 표시/인쇄 품질·배포 판단은 미검증이다.

## Merge 후 contributor PR comment 계획

병합 완료 후 실제 merge SHA와 최종 head CI, fast-pass의 candidate SHA 및 heavy job skip을
구분해 기록한다. 위 로컬 공유 증적을 merge SHA로 고정해 연결하고 report-desktop.png를
Markdown 이미지로 표시한다. 실제 E2E 3 PASS·보고서 pass/issues 0·수동 delegated pass/pass와
스토어/다른 브라우저/문서 출력 품질 미검증을 명시한다. 이슈 상태를 조회하고 별도 승인 없는
수동 종료는 하지 않는다. duration 갱신의 성공 또는 자료 부족에 따른 보류를 확인한다.
문서·증적은 이 PR에 포함하므로 병합 뒤 추가 문서 commit은 만들지 않는다.
검토 worktree는 clean 확인 후 archive한다. 클라우드 세션 소유 원격 브랜치와 공유 target은
보존한다. 기본 checkout의 output은 다른 작업 자료의 소유·필요 여부가 불명확하므로 보존하고
output 정리는 미완료로 보고한다.
