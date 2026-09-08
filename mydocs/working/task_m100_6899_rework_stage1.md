# #6899 R2 — 실패 증적 중심 재설계와 실제 재생 검증

- Issue: #6899 / PR #6903. 일자: 2026-09-08.
- 승인: 메인테이너의 “처음부터 다시 설계를 수정해서 구현” 지시. 로컬 재설계·구현·검증 범위.
- 기존 후보 `38bb7bb87` 보존. 수정 설계 commit `b2629cdd1` 뒤 구현했다.
- 상태: 로컬 R2 검증 완료, 수정 push·원격 CI·main 배포 대기.

## 1. 결함 계보

이번 PR의 Archive B는 Rust channel checksum 다운로드 중 `Connection reset by peer (os error 104)`가
발생해 Install Rust toolchain step에서 중단됐다. `Run Archive B`는 skipped이며 회귀 테스트 실패가 아니다.
별개로 CodeQL workflow Analyze job들은 성공했지만 GHAS CodeQL check는 High 경고 1건으로 실패했다.

기존 reporter는 workflow failure만 탐색하고 테스트 panic 위주로 로그를 해석했다. 이 때문에
설치 오류는 exit 1만 표시했고, workflow가 성공한 CodeQL의 GHAS check는 아예 수집하지 않았다.
이는 아직 main에 reporter가 배포되지 않은 문제와 별개다. 새 코드를 그대로 배포해도 보고가 불충분했다.

## 2. 변경 구조

live PR identity → 모든 workflow 실패 job/step 목록 → 동일 head GHAS check/annotation →
제한된 worker 로그 보강 → 안전한 요약 순서다. 초기 실패 로그가 뒤 보안 증적 예산을 먼저 소모하지 않게 했다.

- Actions run은 repository/head/PR/attempt를 검증한다.
- GHAS는 provider `github-advanced-security`, name `CodeQL`, head, 알려진 PR association,
  check ID·시작 시각을 검증하고 detail에서 다시 확인한다. 같은 head의 최신 check를 선택하며
  특정 Actions attempt와의 연결은 주장하지 않는다.
- 보안 check의 title, 최대 6개 annotation의 상대 경로/행·규칙 제목·설명을 escape/제한한다.
  원문 로그·raw_details·API response URL·signed URL·credential은 복제하지 않는다.
- 실패 위치/오류/테스트 미실행/다음 조치와 원본 링크를 표시한다. 자동 재실행·dismiss는 없다.
- `checks: read` 추가, 기존 write 권한·policy/classifier 구현·required check·fast-pass·concurrency 유지.
- 기존 24요청·45초·요청당 5초·6 job·1 MiB/job·2 MiB metadata·16 KiB summary 상한 유지.
  시간 측정은 monotonic clock으로 변경했다.
- audit는 성공/pending·CodeQL workflow 증적 유무와 독립적으로 보안 check를 조회한다. 정상 경로 항상 0요청 주장은 폐기했다.

## 3. 실제 before/after

같은 원격 PR head `38bb7bb874c6cb85e43963b285227ff0c609617e`와 base `e7e978589`에서 읽기 전용으로 실행했다.

| 대상 | 기존 reporter | R2 reporter |
| --- | --- | --- |
| CI run 34227262955, job 102066785624 | 설치 step와 exit 1 | 설치 step, 연결 재설정, 다운로드 실패, 테스트 미실행, 후속 조치, 직접 링크 |
| CodeQL workflow 34227263072 (success) / GHAS check 102065590077 (failure) | 누락 | High 1건, Bad HTML filtering regexp, scripts/tests/ci-impact-report.test.cjs:202, 대문자 SCRIPT 미검사 설명, 직접 링크 |
| 최종 재생 진단 요청 / 관측 시간 | 5회 / 3,425 ms | 8회 / 약 3,877 ms |

공통 입력 준비용 PR/run metadata 조회 3회는 위 reporter 진단 요청 수와 별도다.
첫 재생은 기존 3,667 ms/R2 약 3,646 ms였다. 소수 관측이므로 속도 개선률이나 CI 전체 비용을 일반화하지 않는다.

재현 도구: `node output/6899/r2-replay.cjs`. 토큰은 로컬 gh 인증에서 메모리로만 가져온다.
현재 PR head가 바뀌면 원본 실패와의 동일성을 주장하지 않고 중단한다.
입력 snapshot `output/6899/r2-input.json`, 이전 보고 `output/6899/r2-before.md`,
수정 보고 `output/6899/r2-after.md`는 로컬 증적이며 원문 로그는 저장하지 않는다.
실제 report의 네트워크 오류·테스트 미실행·규칙명·202행·High·두 원본 링크·다음 조치를 assertion으로
검사하고 생성 Markdown도 직접 읽었다. 이와 같은 정규화 실패 형태는 커밋되는 오프라인 테스트에 포함했다.

## 4. 검증 및 정정

- `node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs scripts/tests/ci-impact-report.test.cjs`: 122 PASS.
- `node --test scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs scripts/tests/verify-trusted-postmerge-ci-reuse-squash.test.mjs scripts/tests/verify-trusted-postmerge-review-bridge.test.mjs`: 83 PASS.
- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'`: 227 PASS.
- actionlint v1.7.12: ci.yml, ci-impact-policy.yml PASS. node 구문·diff 공백 검사 PASS.
- 실제 재생과 기존 verdict 동일성 유지 확인. 신규 보안 provider/head/PR/detail identity,
  pagination·최신 check 선택·권한 실패·민감정보·표시 상한 및 독립 실패 병행 검사를 추가했다.
- 테스트 helper 작성 중 닫는 중괄호 누락을 node 구문 검사에서 발견해 정정하고 전체를 재실행했다.
- 기존 CodeQL 지적은 출력 assertion의 태그명 정규식이다. `<`/`>` 전체 부재와 대소문자 입력의
  escape 검증으로 강화했다. CodeQL dismiss·규칙 제외는 하지 않았다. 원격 재분석 통과는 아직 미확인이다.
- Rust/제품/샘플 변경은 없어 Cargo/WASM/시각 검증을 수행하지 않았다. 외부 ShellCheck는 미설치다.

## 5. 운영 완료 조건과 남은 제한

GitHub 공식 [Checks API](https://docs.github.com/en/rest/checks/runs?apiVersion=2022-11-28)는
annotation 조회에 checks read 권한을 요구한다. 이번 변경은 로컬 YAML이며 원격 권한 변경은 하지 않았다.
[check_run 이벤트 제약](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows) 때문에
Actions head의 보안 check 완료마다 자동 진단된다고 가정하지 않는다. 기존 Controller 이벤트 시점의
snapshot이며 늦게 변경된 check는 다음 이벤트 또는 승인된 재실행이 필요하다.

원격 PR 본문은 최초 후보 설명이므로 수정 push 승인 시 R2 권한/비용/검증으로 함께 현행화해야 한다.
PR은 계속 OPEN이며 Draft 전환·comment 게시·재실행·merge·main 배포는 수행하지 않았다.
수정 head CI/CodeQL 성공 → 승인된 self-review/병합 → 별도 승인된 main 적용 →
실제 Controller summary에서 위 정보와 판정 보존 확인이 끝나기 전 #6899를 완료로 닫지 않는다.
