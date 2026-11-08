# PR #8757 self-review — v0.9.0 배포 준비와 PDF 온보딩

## 최종 판정

승인 — 제출 head `d778ab7bfd28c858c638d7f2c38238bea5a58412`의 필수 CI와 보안 검사가 모두 성공했다. Native의 인프라 시간 초과는 사용자 승인에 따라 같은 head의 해당 작업·종속 집계만 1회 재실행해 해소했다. 코드·workflow·검증 기준을 바꾸지 않았다.

## 범위와 근거

- head: `d778ab7bfd28c858c638d7f2c38238bea5a58412`
- 준비 기능 기준: `5077b216cbc51d2046da07350de562f6f521a7c1`
- 제출 시 live base: `7f76cee60e38fceb63feacaff706cf984b19c427`; #8754 포함.
- 제출 시 base 병합 tree: `aa6ac5a1003289383a1953f45ab2ced5a8e64ad2`, 충돌 0.
- 병합 직전 실제 devel: `ccc2453261f777a1c12662d8d5ff6dafda174071` (#8269 포함). 병합 tree `19cbc624cd03078d6bb3a566c8c6871a49a47a08`, 충돌 0. Rust·Studio 제품 소스, Cargo manifest/lock, Studio package.json은 최신 devel과 동일하다. #8269 원 검토·CI 근거도 대조했다.
- 사용자 승인: push·devel Open PR·해당 head CI·self-review·merge commit. main 승격·태그·실제 게시 제외.
- 기본 경로: collaborator_self_merge. 보조: intake_and_review, local_validation, rework_and_exceptions, post_merge.
- workflow 실행 계약을 수정하므로 옵션 2: 원 PR에 문서-only trailing push를 하지 않는다.

## 코드 검토

- 제품 조판 원칙: 비해당. 이번 diff는 Rust 제품 로직·Studio 구현·Cargo 의존성·기존 fixture/golden을 변경하지 않는다. 합성 merge tree의 제품 소스·자산은 새 devel과 byte 동일하며 #8754·#8269의 제품 소스를 그대로 보존한다.
- 기여 감사: 원 저자·공동 저자·원 PR·cherry-pick SHA를 추적하고 33명 사람과 Dependabot을 분리했다. NOT_FOUND 일부 응답만 수용하며 인증·서버 오류는 실패를 유지한다. 새 devel의 jangster77·pblsketch·postmelee도 명단에 있으며 최종 승격 범위 원장은 게시 전 갱신한다.
- 포장: 기존 고지 원문을 경로와 함께 복사하며 외부 symlink를 거부한다. 실제 npm/Native package의 원문 동일성과 editor LICENSE, VSIX의 runtime dist/media 유지·미사용 사본 제외를 확인했다.
- CI 정책: mutable PR 입력을 privileged shell 명령으로 실행하는 경로·새 권한·보안 규칙 비활성화를 추가하지 않았다. 신규 계약 실행은 read-only이며 promotion 증거는 direct/adapter/verify-only/명시적인 completed verdict로 구분한다. incomplete/skip/build failure를 성공 증거로 인정하지 않는다.
- Layout advisory: 실제 ANOMALY는 advisory로 유지하며 누락·중단·parser 오류·잘못된 집계는 completed로 승격하지 않는다.
- 브라우저 E2E: 기존 #1328/#1454 글꼴 동의 화면의 실제 버튼을 선택한다. 동의 저장소 사전 입력·제품 UI 우회·검사 상한 완화 없음. Chrome/Firefox 실제 종료 결과를 확인했다.
- 안내: CLI PNG 디렉터리/PDF 파일 구분, Rust/WASM 실제 API, 같은 버전의 Worker/Studio, 암호/한도/폰트·OCR 비지원·registry 미게시를 구분했다.
- 제품 빌드와 시각 수치의 한계: 로컬 artifact는 #8754 이전 기능 기준으로 검증했다. 새 base의 전체 독립 Print 재검증이나 source-only 검사를 Cargo registry publish 검증으로 표현하지 않는다.

## 로컬 증거

준비 보고서와 공개 local-validation.json의 실제 빌드 SHA·범위·결과·package 해시를 따른다. 제품 코드가 변하지 않아 동일 광범위 회귀/빌드를 이번 원격 단계에서 반복하지 않는다. 최초 실패 E2E와 수정 후 통과를 함께 보존했다.

## CI와 병합 조건

- [CI attempt 2](https://github.com/edwardkim/rhwp/actions/runs/42420774418): 전체 회귀 A–E 12,777건, Lint, Frontend, Native, Build & Test 성공. Frontend의 실제 Studio 검사는 2,769건 통과·2건 건너뜀이다.
- [Native 재실행](https://github.com/edwardkim/rhwp/actions/runs/42420774418/job/128556171504): checkout 3분 16초, 전체 약 23분으로 완료. 원 실행은 checkout 15분 10초 후 30분 상한에 도달했으며 실제 assertion·컴파일 오류는 없었다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42420774441), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42420774469), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42420774468) 성공. 최종 `CI Impact Policy`도 success다.
- 최신 check 31개 success·5개 skipped. Render Diff는 이번 변경 경로에 해당하지 않아 워크플로 path filter에 따라 실행 대상이 아니다. 이를 실행 성공으로 집계하지 않았다.
- GitHub 제품 CI의 합성 merge는 `7f76cee60e`를 기준으로 실행했다. 뒤에 병합된 #8269까지 포함하는 새 전체 CI를 실행한 것으로 주장하지 않는다. 최종 devel과 충돌 없는 병합 및 제품 소스 동일성으로 기능 보존을 확인했다.
- 기존 npm 개발 도구 High 2건·문서 metadata 기존 오류 16건·#8745 보호 브랜치 전체 분석은 별도 잔여 항목으로 공개했다. release 승인·tag·배포·crates.io 게시 완료로 표현하지 않는다.
- 사용자가 승인한 merge commit으로 main ancestry를 보존한다. CI 정의 변경 PR의 옵션 2에 따라 확정된 검토·CI·merge 기록은 별도 문서 PR로 준비한다.

[로컬 준비 보고서](https://github.com/edwardkim/rhwp/blob/d778ab7bfd28c858c638d7f2c38238bea5a58412/mydocs/report/release_0_9_0_preparation_report.md) · [실제 로컬 검증 결과](https://github.com/edwardkim/rhwp/blob/d778ab7bfd28c858c638d7f2c38238bea5a58412/mydocs/tech/investigations/release-v0.9.0/local-validation.json)

## 병합 후 확정

- 2026-11-08 08:28:10 KST에 [PR #8757](https://github.com/edwardkim/rhwp/pull/8757)을 merge commit `0e6cfe91df7aabb886319346f78af568def25a64`으로 병합했다. 관리자 우회는 사용하지 않았다.
- 실제 parents는 `ccc2453261f777a1c12662d8d5ff6dafda174071`과 검증한 `d778ab7bfd28c858c638d7f2c38238bea5a58412`다. 실제 tree `19cbc624cd03078d6bb3a566c8c6871a49a47a08`은 직전 병합 시뮬레이션과 같다. main `1a76570e83`과 원 후보 ancestry를 모두 확인했다.
- [공개 self-review](https://github.com/edwardkim/rhwp/pull/8757#pullrequestreview-5815557037)와 [정식 CI·병합 증거](../assets/pr8757/ci-results.json)를 남겼다. Native CI 실제 선택 검사는 237건 통과이며 로컬 공식 전체 Native 검사 수와 구분한다.
- 로컬 devel을 같은 merge SHA로 fast-forward했다. 다른 작업의 `pr-8296-seal-pua` worktree는 유지했다.
- [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/42425380606)은 성공했다. B–E의 실제 attempt 1 측정을 확인하고 메타데이터 commit `767b5c82509bf83339f744ae66403a95b7295118`로 게시했다. 검증 CI를 병합 후 다시 실행하지 않았다.
- CI 정의 변경의 옵션 2에 따라 이 검토·오늘할일·CI 증거를 별도 [PR #8760](https://github.com/edwardkim/rhwp/pull/8760)에 포함했다. 0.9.0 사용자 문서와 crates.io 초회 배포 범위 보완도 같은 PR에 반영했다. #8760 제출 head의 Full CI는 성공했으며 해당 PR의 [self-review](pr_8760_review.md)와 trailing head 검증·머지 후 종료 정리를 진행한다.
- #8745는 보호 브랜치 전체 보안 분석이 남아 OPEN이다. crates.io 초회 배포는 2026-11-08 결정에 따라 0.9.0 릴리즈 범위에 포함됐다. main 승격·수동 검증 workflow·태그·실제 게시 실행은 최종 릴리즈 절차로 진행한다.
- 후속 기록이 진행 중이므로 임시 `output/`과 원 PR branch는 아직 종료 정리하지 않았다. 기록 반영 완료 후 규정에 따라 정리한다.
