# 선택 실행 증적 보고서 조사·계획

Issue: #7449

- 기준: 최신 upstream/devel `def947c2149df5bc812d801e6e79be19a403b4d1`. 시작 시 REST API와 fetch로 일치 확인. #7449 댓글 0개, 관련 열린 PR 없음. 작성자는 실행 계정 postmelee와 동일하며 담당자를 지정했다.
- #3512 종료와 #7283의 자동 CI 철회 합의를 확인했다. 기존 결과를 소비하는 독립 로컬 도구만 추가한다.
- 원인: schemaVersion 1 결과는 사람이 JSON·로그·PNG를 각각 열어야 하며 파일별 소속/해시가 없다. 기존 pass를 증적 완전성의 pass로 해석하면 누락·혼입을 놓친다.
- 구현: 실행기에 runId·suite별 증적 SHA-256 목록을 추가한다. 보고서 생성기는 기존 v1 상태를 보존하되 소속 증거가 없는 과거 결과는 incomplete로 표시한다. 후보 inventory 해시와 도구 SHA 형식을 확인하며 후보 빌드 SHA와 혼동하지 않는다.
- 저장·공개: 로컬 새 폴더에 공유용 allowlist 요약 HTML/JSON과 비공개 원본 링크 인덱스를 분리한다. 원문/프로필은 수집하지 않고 원본 로그·JSON·PNG도 공유 요약에 복사하지 않는다. 외부 업로드 없음.
- 검증: pass/fail/error/not-run/running, 잘못된 schema/hash/SHA, 누락·변조·혼입·symlink, 정보 제외, 같은 CLI 재현 및 기존 실행기 계약. 렌더·WASM·제품 변경은 없다.
- 승인: 사용자 지시에 구현·테스트·기록·push·PR 생성 포함. merge/release/deploy/issue close 제외. 보고서 읽기 편의성과 공유용 제외 결과는 사용자 수동 확인으로 남긴다.

## 구현·검증 결과

- 실행기: 원자적 result.json 갱신, 실행 ID와 suite별 증적 해시, 로그 실행/suite 머리말, 실패 진단 runId. 기존 suite 실행·중단 정책은 그대로 소비했다.
- 보고서: 공유용 HTML/JSON과 비공개 원본 상대 링크 인덱스 분리. v1 legacy·스키마·후보 inventory·도구 SHA·누락·혼입·변조를 검사하고 원본 상태와 incomplete를 따로 보존했다. 독립 기대 SHA 대조와 source 해시로 묶은 사용자 수동 기록을 지원한다.
- `npm --prefix rhwp-chrome ci --no-audit`: 44개 설치. 처음 기존 page-budget 검사는 Puppeteer 누락으로 실패했으며 설치 뒤 정식 전체 재실행에서 통과했다.
- Node 계약 32 PASS / 0 FAIL / 0 SKIP. 첫 새 CLI 테스트 실패는 테스트가 환경 fixture 객체를 공유해 오염한 원인으로 수정했다. 실패를 skip하거나 성공으로 기록하지 않았다.
- 수정 전 base 실행기에 같은 새 run.test assertion 적용: runId undefined로 1 FAIL. 수정 후 같은 assertion을 포함한 32개 PASS. 환경 실패와 구분했다.
- 합성 예제 생성: pass→pass, fail→fail(뒤 2 suite not-run), error→error(모두 not-run), running→incomplete, 원본 pass의 로그 누락→incomplete. 실제 브라우저 실행 결과가 아니다.
- 설치된 Chromium 151에서 같은 HTML 내용을 로드해 데스크톱과 390px 화면의 표·미확인·정보 제외를 확인했다. 가로 overflow 없음. 로컬 file URL은 관리 정책으로 차단되어 setContent를 사용했다. Chrome for Testing 다운로드는 storage.googleapis.com EAI_AGAIN으로 실패했다. 실제 확장 전체 E2E·스토어·Edge/Firefox·문서 품질 검사는 실행하지 않았다.
- 변경 경로는 E2E 증적과 보고서·관련 매뉴얼/기록뿐이다. Rust/Studio/렌더러/후보 빌드/CI 정책을 변경하지 않아 Cargo·WASM·Visual Sweep 게이트 비해당. HWP/HWPX/PDF 입력을 사용하지 않았다.
- 보고서의 읽기 편의성과 공유 전 정보 제외에 대한 사용자 직접 판정은 미확인으로 유지했다.
