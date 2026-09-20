# Issue #3515 수행·구현 계획

- Parent: #3512, 선행 구현: #3514 및 #3513 (`b4c6b0e58`)
- 사용자 승인: 2026-09-20, #3513 다음 CI 연결 진행.
- 운영 등급 O3: 실제 browser job, O2 영향 분류·cache·artifact 포함.

## 구현

기존 trusted preflight 입력을 재사용하는 독립 Chrome 영향 분류기를 추가한다. 기존 Rust/render/CodeQL
판정 정책을 바꾸지 않는다. Chrome E2E가 필요하면 frontend package lane도 활성화해 같은 run의 fresh
WASM과 production dist를 만든다. 이 dist를 짧게 보존하는 run/attempt별 artifact로 browser job에 넘긴다.
브라우저 job은 5분 timeout이며 Chrome 자체·Puppeteer·manifest 버전과 실행 시간을 출력한다.
빌드 준비 시간은 browser job의 90초 warm 목표와 구분한다.

PR은 잠긴 Chrome cache를 복원만 한다. branch push CI가 없으므로 캐시 최초 저장은 별도의
수동 browser-cache 준비 workflow에서 신뢰된 기본 branch 코드로 수행한다. 검증 CI를 병합 후 다시
실행하지 않으며, 캐시가 없으면 해당 PR job에서 Chrome을 다운로드하되 저장하지 않는다.
일반 frontend 의존성 설치는 Puppeteer 자동 다운로드를 끈다.

분류 실패·불완전 목록·rename 정보 누락·tag/manual은 실행한다. extension source, shared/sw,
Studio production source·필수 정적 surface·WASM·빌드 입력이 영향을 준다. Firefox/Safari/VSCode/npm
editor 전용 코드와 문서·Studio tests/e2e만 바뀌면 건너뛴다. 불명확한 production 경로는 실행한다.

필수 Build & Test 집계는 expected run의 success와 expected skip의 skipped를 각각 검증한다.
실패 시 console·worker·다운로드·단계·extension URL·screenshot만 artifact로 남기며 profile과
fixture 원본은 제외한다. 성공 시 작은 결과 summary만 남기고 진단 artifact는 올리지 않는다.

## 검증과 완료 경계

- 영향 경로·rename·목록 잘림·tag/manual/fallback의 Node 계약.
- YAML/actionlint·생산자/소비자·required 집계의 실행/skip/failure/cancelled 계약.
- 실제 production dist에서 smoke/download/lifecycle 전체 실행, #3513 10회 연속 결과 연결.
- 실패를 의도적으로 발생시켜 제한된 진단 파일과 오류 exit code 확인.
- GitHub Actions retry 없는 3회와 cache 로그는 원격 게시 승인 후 실제 run으로 확인한다.
- Firefox Phase 2는 Chrome CI의 Linux/CfT 안정화 결과를 확인할 때까지 구현 보류 근거를 Epic에 남긴다.
- #3512 최종 통합 완료 후에만 증적 보고서 연동 후속 이슈를 만든다.
