# v0.9.0 릴리즈 실행 — 2026-11-08

## 승인과 기준

작업지시자는 문서 PR #8760 완료 보고에 이어 최종 배포 검증, main 승격·태그와 crates.io 실제 게시의 진행을 승인했다. 해당 승인에 따라 필요한 workflow 실행·릴리즈 PR·게시·결과 확인을 진행한다. 토큰 값·secret·owner·branch protection을 변경하지 않는다. 별도 실패 작업의 취소·재실행이나 검증 예외는 자동 승인으로 확대하지 않는다.

- 준비 PR #8757, 문서·후속 기록 PR #8760은 이미 devel에 병합됐다.
- 현재 원격 main: `1a76570e833917d15817415a53c09ad61ab3203f` (v0.8.7).
- 최종 기능·문서 기준 devel: `41e08ae19e360a32179d851ca2534c1195a8dced`; main ancestry 확인.
- 마감 문서 branch: `release/0.9.0-finalization-20261108`. 이 변경도 devel PR로 통합한 뒤 정확한 승격 SHA를 고정한다.
- 기여자 재감사: 4,896개 커밋, 사람 33명·Dependabot 1개, 미해결 0건. 기존 명단과 추가·제거 0명이다. 원시 이메일·GitHub 응답은 ignored output에만 둔다.
- 공개 crates.io 조회: 14개 이름 모두 HTTP 404. 미등록 관측이며 예약·소유권 확인을 뜻하지 않는다. Cargo 인증 파일의 존재만 확인했으며 값은 읽지 않았다.

## 순서와 완료 조건

1. 마감 문서·기여 원장을 devel에 통합한다. release-facing “배포 준비” 표기를 제거하고 게시 날짜·각 채널 실제 상태는 GitHub Release와 registry에 연결한다. 기존 0.8.7 이력은 보존한다.
2. 승인된 exact devel에서 promotion inventory의 22개 실행 표면을 13개 고유 workflow caller로 검증한다. 실제 caller 수는 정책·inventory를 다시 대조한다. CI·CodeQL·5플랫폼 CLI·WASM·VSIX·Pages verify-only·native host·renderer·fuzz·advisory의 필수 job/artifact를 확인한다. 외부 publish가 실행되지 않았음을 확인한다.
3. 실제 14개 `.crate`를 새 파일로 포장하고 VCS·파일·라이선스·정규화 manifest·gzip 무결성·checksum을 검사한다. 읽기 전용 loopback registry에서 path/patch 없는 Rust·CLI·WASM 소비를 실행한다. 원본과 다른 SHA의 산출물을 최종 게시 증거로 승격하지 않는다.
4. 필수 증거가 모두 성공하면 같은 저장소 `devel → main` PR을 만들고 promotion gate·Build & Test·self-review 후 일반 merge한다. 정확한 main merge SHA에 v0.9.0 태그를 게시한다.
5. Release Binary의 다섯 archive와 SHA256SUMS, same-commit package 호출 및 네 공개 채널의 completed verdict를 확인한다. GitHub Release에 최종 국·영문 안내·기여자를 반영한다.
6. 최종 source에서 14개 Cargo 패키지를 의존 순서대로 dry-run·검증·게시한다. 공개 index·checksum·owner·Rust/CLI/WASM 소비·docs.rs를 확인한다. 게시 후 다른 digest를 같은 버전으로 덮어쓰지 않는다.
7. Chrome/Edge/Firefox의 최종 ZIP·Firefox source ZIP과 심사 안내를 준비하고 사용 가능한 인증된 스토어 경로로 제출 상태를 확인한다. 접근 가능한 경로가 없으면 필요한 수동 제출 정보만 구체적으로 보고한다. 실제 업로드 전에는 완료로 기록하지 않는다.
8. 승인 범위의 devel 동기화·필수 증거 보존·임시 branch/output/debug 종료 정리를 완료한다. #8745는 보호 브랜치 전체 보안 분석 결과를 실제 대조한 뒤 판단한다.

## 현재 검증

- source/publish preflight의 기존 계약 36건 통과.
- contributor audit·0.9.0 릴리즈 준비 계약 18건과 변경 Markdown 13개 링크 검사 통과. 과거 0.8.x 변경 이력은 수정하지 않았다.
- 공개 GitHub의 현재 commit→PR 연결을 다시 조회해 원장의 provenance가 갱신됐다. 기존 33명 모두 유지했으며 이전 감사 근거도 Git 이력에 보존돼 있다.
- 마감 문서는 collaborator self-merge 경로로 처리한다. 보조 절차는 intake_and_review, local_validation, review_only_fast_pass, post_merge이며 제품·workflow·버전 변경은 없다. PR 번호 기반 검토 기록은 해당 head CI 성공 뒤 추가한다.
- 최종 `.crate`·원격 promotion·공개 배포는 아직 실행 전이다.

## 실제 포장에서 확인한 예약 파일명 보정

`22e56b1349`의 실제 `cargo package`는 네 번째 패키지 `rhwp-jpeg-encoder`에서
`invalid inclusion of reserved file name Cargo.toml.orig in package source`로 실패했다.
첫 세 패키지는 포장됐으며 업로드는 수행하지 않았다. #8360의 초기 manifest는 이 파일을
include하지 않았으나 #8389에서 include가 추가됐고, #8757의 `cd71fa9c33`은 중복 후보를
제거하면서 source 원장에 해당 경로를 유지했다. source 후보 검사만으로 Cargo의 예약 이름
충돌을 발견하지 못했다.

원본 manifest를 `upstream/Cargo.toml.orig`로 옮겨 바이트·원 archive provenance를
보존하고 실제 include와 source 정책을 일치시킨다. 제품 Rust source·feature·version은
변경하지 않는다. 공통 사전 검사는 Cargo 생성 루트 파일의 직접·wildcard·extra mapping
선택을 거부하며 하위 upstream 경로는 허용한다. 신규 계약은 수정 전 5개 subcase 실패를
재현했다. 수정 후 계약·실제 14개 포장과 독립 소비 결과는 다음 검증에서 확정한다.

## PR #8763 제출 head 검증 완료

보정한 `d30e12e0d09c3b1afb500bb8f5b0e781c058d35e`에서 source/publish·기여 감사·
릴리즈 계약 57건과 14개 실제 `.crate` 포장/무결성 점검이 통과했다. 업로드는 하지 않았다.
path/patch 없는 Rust 기본·최소·PDF/codec 제외 소비와 README 예제·실패 후 복구를 확인했다.
기본/native-skia CLI 설치와 실제 PDF → HWPX·PNG 출력도 통과했다. no-PDF CLI와
최적화 WASM/브라우저 검증은 이어서 실행한다.

[Full CI](https://github.com/edwardkim/rhwp/actions/runs/42431522165)의 Archive 12,803건,
선택 Native 237건, Studio 2,777건이 통과했고 기존 skip은 2건이다. CodeQL·Adapter·Proptest와
최종 정책 감사도 성공했다. [self-review](../pr/archives/pr_8763_review.md)와 실제 증거를
CI 성공 뒤 기록했다. 이 문서-only trailing head의 검사·devel 병합이 다음 단계이며,
최종 main source와 공개 registry 소비·태그·배포가 완료됐다는 뜻은 아니다.

## devel 병합과 승격 검증에서 발견한 Oracle 입력 누락

#8763은 2026-11-08 10:27:07 KST에 `78615c52a7c7d7a6b25fad57c3bf1041fb2dfcf2`로
병합됐다. trailing head `927be2076f`는 실제 fast-pass로 source CI를 재사용했고 required
check와 최종 정책 감사도 성공했다. 병합 후 duration 갱신 `42434310949`도 성공했다.
review·오늘할일은 병합 head에 이미 포함됐고 전용 local/upstream branch는 삭제했다.
실행 중인 릴리즈 registry·WASM이 사용하는 output/target과 다른 작업은 보존한다.

위 exact devel에서 승격 inventory의 22개 실행 표면을 13개 고유 workflow로 시작했다.
모든 run의 head SHA가 같음을 확인했다. 실제 게시·태그는 아직 하지 않았다.

[Oracle 실행](https://github.com/edwardkim/rhwp/actions/runs/42434354283)은 job 자체는
success지만 `oracle-public-advisory-verdict/verdict.json`이 `build-failed`,
`promotionEligible=false`다. 따라서 승격 완료가 아니다. 컴파일 오류는 다음 세 경로가
sparse checkout에서 제외된 데서 발생했다.

- `assets/fonts/enclosed-pua-metrics.json`: `enclosed_pua_geometry.rs`의 include.
- `assets/fonts/LatinModernMath-VariableOutline-subset.otf`: `equation/variable_outline.rs`의 include.
- `assets/font-rules/font_rule_registry_v2.json`: `native_webfonts.rs`의 include.

기존 #6689는 외부 include 입력을, #7605의 `c687bfd1fa`는 vendored Cargo patch를
보강했다. 이후 `75fadf3152`·`08e4192d15`·#8565의 `5551091085`에 해당하는 현재
조판 코드와 입력을 대조했다. 제품 Rust는 정상이며 기존 Oracle 목록의 `assets` 누락이 원인이다.

보정 후보 `release/0.9.0-promotion-fix-20261108`은 Oracle sparse 목록에 `assets`
한 줄을 추가한다. 실제 source-relative include 경로를 대조하는 기존 workflow 계약을 보강해
수정 전 실패를 재현했고, 보정 뒤 관련 계약 65건이 통과했다. 실제 cone-mode checkout에서
세 입력의 바이트 동일성도 확인했다. 제품 코드·검사 명령·advisory 판정·권한·배포 설정은 바꾸지 않는다.

실제 sparse compile 검사를 완료한 뒤 CI 실행 정의 변경의 diff·검증·원상복구 범위를 제시하고
원격 적용 승인을 받는다. 같은 SHA의 재실행으로는 입력 누락이 해결되지 않는다.
승인 후 devel 보정 PR·Full CI·self-review·병합을 먼저 완료하고 새 devel SHA에서 승격
증거를 다시 확보한다. CI 변경 PR의 운영 기록은 옵션 2로 남긴다. 현재 다른 workflow는
취소하지 않고 결과를 확인한다.

보정 목록으로 새로 만든 실제 Git cone-mode sparse checkout에서
`cargo check --locked --profile release-test --bin rhwp`가 118초에 통과했다.
원격과 같은 파일 선택 조건의 include 오류가 해결됐음을 확인한 것이며, 아직 원격 Oracle
비교와 verdict가 성공했다고 주장하지 않는다. 관련 계약 65건·공백·문서 링크를 확인한다.
원격 적용 후 문제가 있으면 보정 commit을 별도 PR에서 revert하며 검사 기준을 낮추지 않는다.

병행한 registry 소비 검증도 완료됐다. 기본/최소/PDF 구성의 Rust, 기본/native-skia/no-PDF
CLI 설치와 실제 사용 7건, 두 최적화 WASM의 CDP 19222 브라우저 검사 10건이 통과했다.
Worker의 입력 한도·시간 초과·기존 문서 보존·정상 재시도를 확인했다. 이 검증의 archive
source는 `d30e12e0d0`이며 이후 차이는 운영 문서·Oracle workflow/계약뿐이다.
읽기 전용 loopback registry는 모든 소비가 끝난 뒤 종료했다. 공개 registry 검증과 구분한다.
