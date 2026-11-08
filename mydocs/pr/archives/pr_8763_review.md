# PR #8763 self-review — 0.9.0 안내 마감과 Cargo 포장 보정

## 판정과 승인

승인 — 제출 head `d30e12e0d09c3b1afb500bb8f5b0e781c058d35e`의 필수 CI와 최종 정책 감사가
모두 성공했다. 실제 포장 실패의 원인을 수정하고 14개 archive를 확인했다. 제품 Rust source·
feature·version·workflow 변경은 없다. 이 기록 뒤의 문서-only trailing head는 실제 required
check·병합 가능 상태를 별도로 확인한다.

- PR: https://github.com/edwardkim/rhwp/pull/8763
- base: `41e08ae19e360a32179d851ca2534c1195a8dced`.
- 경로: collaborator_self_merge; 보조 절차 intake_and_review, local_validation, review_only_fast_pass, post_merge.
- 작업지시자는 #8760 완료 뒤 0.9.0 최종 검증·승격·게시 진행을 승인했다. 이번 PR은 그에 필요한 devel 마감 보정이다. 실패 workflow 재실행·보안 분류·권한 변경 승인을 확대하지 않는다.

## 검토 결과

1. 국·영문 CHANGELOG·릴리즈 노트·PDF 시작 안내·패키지/확장 README의 준비 표기를 마감했다.
   실제 배포 날짜를 추정하지 않고 채널별 게시 결과를 GitHub Release와 registry에 연결했다.
   기존 0.8.x 이력과 아직 게시되지 않은 채널의 상태를 보존했다.
2. 최신 devel까지 4,896개 커밋·원 PR·공동 저자·cherry-pick 출처를 대조했다. 사람 33명과
   Dependabot 1개이며 기존 명단의 추가·누락은 0명, 미해결 identity도 0건이다.
   공개 API의 출처 연결 변화와 조회 불가 원본 9개를 원장에 명시하고 이전 원장은 Git 이력에 보존했다.
3. 실제 `cargo package`에서 JPEG encoder의 루트 `Cargo.toml.orig`가 Cargo 예약 이름과
   충돌했다. #8360 → #8389 → #8757의 manifest/include 이력을 추적했다. 원본을
   `upstream/Cargo.toml.orig`로 옮겼으며 100% 동일한 바이트·SHA-256·고지를 보존한다.
   Cargo가 생성하는 rhwp의 정규화 전 manifest와 upstream 원본을 혼동하지 않게 했다.
4. 공통 source 사전 검사는 직접·wildcard·extra mapping으로 Cargo 생성 루트 파일을
   덮는 후보를 거부한다. 정상 nested upstream 원본은 허용한다. 실제 Cargo 실패의 재발을
   막는 검사이며 제품 API나 의존 그래프를 바꾸지 않는다.
5. 조판·렌더 입력·Rust 동작 변경이 없어 새 Visual Sweep·편람 gate·독립 Print 첨부는 비해당이다.
   제품 CI 성공과 실제 14개 포장을 확인했으며, 별도로 진행 중인 최종 릴리즈 소비·게시 검증을
   이 PR의 완료 결과로 과장하지 않았다.

## 완료한 검증

- source/publish preflight·기여자 감사·릴리즈 준비 계약 57건 통과.
  예약 이름은 보정 전 5개 subcase 실패 → 보정 후 통과다.
- 깨끗한 동일 head에서 14개 실제 `.crate` 포장·VCS SHA·파일·정규화 manifest·라이선스·
  원본 provenance·gzip CRC/잔여 바이트·checksum 확인과 읽기 전용 공개 registry 점검 통과.
- path/patch 없는 독립 registry 소비: 기본/최소 Rust, 오류 계약, README 예제,
  기본 PDF 4개와 codec 제외 구성의 명시적 거부/정상 입력 복구 통과.
- 기본/native-skia CLI를 `cargo install --locked`로 설치해 버전을 확인했다.
  설치된 기본 CLI의 PDF → HWPX 격리 실행과 Native CLI의 실제 PNG 출력을 확인했다.
  no-PDF CLI·최적화 WASM/브라우저와 최종 main source·공개 registry 소비는 릴리즈 실행에서 계속한다.
- 변경 문서 링크와 `git diff --check` 통과. 최신 devel 병합 simulation의 tree는
  `d689887a5e7991b7d476f5d24a183203de721c2d`이며 GitHub 합성 merge tree와 같다.
- [포장·소비 범위](../assets/pr8763/package-validation.json), [실행 기록](../../working/release_0_9_0_execution_20261108.md).

## GitHub CI

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/42431522165) attempt 1 성공:
  Archive A–E **12,803건**, 선택 Native **237건**, Studio **2,777건 통과·2건 건너뜀**.
  Lint·Frontend 후속 검사와 Build & Test 집계도 성공했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42431522206),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/42431522172),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/42431522178),
  [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42433951394) 성공.
- 수동 재실행·취소·경고 분류 변경은 없었다. 보정 전 head의 실행은 기존 stale-run 자동화가 취소했다.
  Render Diff는 이번 diff의 path filter상 비대상이며 실행 성공으로 집계하지 않는다.
- 수치는 이번 실제 job 로그·작은 shard count artifact에서 산출했다. [CI 증거](../assets/pr8763/ci-results.json).

## 후속 조건

이 기록과 오늘할일의 문서-only trailing head를 push한 뒤 실제 fast-pass·required check·
최신 base 병합을 확인해 일반 merge한다. 이후 정확한 devel SHA의 승격 workflow와 main PR을
진행한다. 이 PR 자체가 main 승격·태그·crates.io 게시를 완료한 것은 아니다.
릴리즈 소비 검증이 사용하는 registry·output·target은 사용 종료 뒤 정리하고 다른 작업의
worktree·branch는 보존한다.
