# crates.io 배포 사전 준비 계획

- 작성일: 2026-10-23
- 상태: 이슈 등록과 1단계 source 선별·실측 완료. 2단계 구현 및 실제 registry 게시는 다음 게이트다.
- 요청: 다음 릴리즈는 0.9.0이며, crates.io 배포를 미리 준비한다.
- 착수 기준 SHA: `upstream/devel` `def947c2149df5bc812d801e6e79be19a403b4d1`.
- 사전 조사 기준 SHA: `605ccfd92f2f12f18e14182548d729f9929fda00`.
- 현재 제품 버전: 0.8.7. 이번 계획 작성에서 버전을 변경하지 않는다.
- Issue: [#8174](https://github.com/edwardkim/rhwp/issues/8174).
- 작업 브랜치: `chore/issue8174-crates-preparation-20261023`.
- 담당자 `edwardkim`, milestone `v1.0.0`, labels `packaging`·`rust`·`enhancement`.
- 사용자 승인: 2026-10-23 「이슈로 등록해서 진행하도록 합니다」.
- [구현 계획](task_m100_8174_impl.md). 단계별 변경을 커밋하고 다음 단계로 전환한다.
- [1단계 결과](../working/task_m100_8174_stage1.md): 8개 source 후보의 목록·의존 순서·고지·실측을 기록했다.
  본체 8,329,278바이트, PDF backend 6,814,747바이트의 gzip tar이며 정규화·검증된 `.crate`는 아니다.
- 2026-10-23 추가 요청에 따라 [보안 제한과 기능 사용 범위](../manual/security_feature_limits.md)를
  정리했다. 기능 활성화와 2단계 manifest 변경은 이 현황 정리에 포함하지 않는다.
- 권위 절차: [문서·Git workflow](../manual/codex/docs_and_git_workflow.md),
  [배포 가이드](../manual/publish_guide.md), [GitHub 운영](../manual/github_operations.md).

## 목표와 승인 경계

0.9.0의 기존 기능과 보안 수정을 유지하면서, 저장소 밖 Rust 프로젝트와 CLI 사용자가
registry 패키지만으로 rhwp를 빌드·사용할 수 있게 준비한다. 실제 게시·이름 예약·태그·토큰
설정은 이번 사전 계획의 실행 항목으로 간주하지 않는다.

2026-10-23의 이슈 등록·진행 지시로 사전 준비 계획과 1단계 착수를 승인받았다.
공개 API의 지원 범위·의존성 전략·단계별 완료 조건을 구체화하고 단계 보고 절차를 적용한다.
remote push·PR 생성·workflow 실행·실제 registry 게시의 승인도 각각 구분한다.

## 선행 결정과 현재 기준선

1. [#4375](https://github.com/edwardkim/rhwp/issues/4375) → 원 PR #4376 → 통합
   [#4496](https://github.com/edwardkim/rhwp/pull/4496)에 crates.io 게이트가 있었으나,
   [#4655](https://github.com/edwardkim/rhwp/issues/4655)의 채널 롤백 이후 현재 활성 경로가 아니다.
   [당시 검토](../pr/archives/pr_4376_review.md)와
   [후속 배포 실측](../working/archives/task_m100_4601_stage4.md)은 게시 완료의 근거가 아니다.
2. [#5446의 메인터너 결정](https://github.com/edwardkim/rhwp/issues/5446#issuecomment-5327810241)은
   v1.0.0까지 신규 배포 채널을 보류하고 공개 API, MSRV, 소비자 검사, 게시·업데이트·yank 운영을
   먼저 확정하도록 했다. 이번 0.9.0 준비는 이 결정의 재검토이며, 과거 이슈가 closed라고 해서
   crates.io 배포가 완료됐거나 승인됐다고 해석하지 않는다. 기능 없는 이름 예약 게시도 채택하지 않는다.
3. [#7875](https://github.com/edwardkim/rhwp/issues/7875) → 원 PR #7876 → 통합
   [#7981](https://github.com/edwardkim/rhwp/pull/7981)은 git/path 소비자의 `wasm-api` 선택 계약이다.
   [검토](../pr/archives/pr_7876_review.md)의 실제 소비 앱 검증을 재사용하되,
   공식 registry 게시와 Rust 공개 API 안정성까지 입증한 것으로 확대하지 않는다.
4. PDF dependency 배포 준비는 [#7911 B08](task_m100_7911_backlog.md)과
   [기존 구현 결정](task_m100_7911_impl.md)에 연결한다. 보안 수정본의 별도 내부 이름과
   원본·라이선스·NOTICE 보존을 유지한다. 중복 구현이나 보안 수정 철회로 게시를 쉽게 만들지 않는다.

읽기 전용 점검 증적은 ignored `output/release/v0.9.0-readiness-20261023/`에 있다.

- `cargo package -p rhwp --list --offline --locked`: exit 0. 50,352개 파일,
  실제 존재하는 파일의 원본 크기 합계 6,565,804,834바이트. 압축 `.crate` 크기는 아직 미측정이다.
- 내부 3개 crate는 버전 없는 path 의존성과 `publish = false`다.
- PDF backend와 보정 JPEG decoder 2개도 `publish = false`다.
- 로컬 `svg2pdf` 패치는 게시 manifest에서 제거되는 `[patch]`에 의존한다.
- 현재 release workflow·완료 집계에 crates.io 채널은 없다.
- 조회 시점에 `rhwp`와 기존 보조 패키지명 6개는 registry API에 등록 항목이 없었다.
  이것을 이름·게시 권한 확보의 증거로 보지 않는다.

## 구현 전에 확정할 사항

| 결정 | 검토할 내용 |
| --- | --- |
| Rust 지원 계약 | HWP/HWPX 파싱·Document IR·저장/내보내기·PDF 입력 중 지원할 사용자 여정, 공개 API와 0.9.x 호환성 정책을 명시한다. 현재 `pub` 표면 전체의 안정성이 확정됐다고 가정하지 않는다. |
| 의존성 게시 방식 | 내부 3개와 보안 fork를 별도로 게시하는 전략을 우선 비교한다. 기존 모듈을 본체에 흡수하는 대안은 경계·검증 비용을 함께 평가한다. |
| PDF export fork | upstream 반영 또는 별도 이름의 게시 의존성으로 수정본을 유지한다. `svg2pdf`의 결정화·그러데이션·텍스트 중복 방지 수정을 보호한다. |
| 버전·소유 | 본체 0.9.0과 보조 crate의 버전 정책, owner와 보안 업데이트 책임을 정한다. 내부 0.1.0 또는 upstream 기반 보정 버전을 일괄 0.9.0으로 바꾸지 않는다. |
| MSRV·feature | 현재 고정 툴체인 1.93.1을 기준으로 검증하고 실제 지원할 최소 버전을 정한다. 기본, default 해제, `wasm-api`, `pdf-import`, `native-skia`의 지원 조합을 명시한다. |
| 보안상 기능 제한 | PDF 기본 포함과 추가 codec/CMM 미활성, 자원 초과 거부, 전체 할당 증명 미완료를 README와 소비 계약에 명시한다. [현황 목록](../manual/security_feature_limits.md)의 각 제한과 일반 미지원을 구분하며, feature를 켜거나 `publish=false`를 푸는 것으로 보안 게이트를 대신하지 않는다. |
| 배포 채널 | 초기 게시 순서, 실제 게시 시점, 이후 OIDC 전환과 부분 실패 복구·yank 절차를 정한다. 기존 네 채널을 포함한 전체 자동화를 한 번에 바꾸는 것으로 가정하지 않는다. |

## 단계와 완료 조건

### 1단계 — 게시 graph와 최소 패키지

- 제품 코드를 바꾸기 전에 각 crate의 실제 dependency와 지원 API 소비 경로를 확정한다.
- 별도 게시 전략이면 본체, 내부 3개, PDF backend, JPEG decoder 2개, PDF export fork의
  약 8개 패키지 graph와 순서를 만든다. 이는 현재 권고안이며 확정된 패키지 분해가 아니다.
- Cargo include 목록에 Rust 소스, 컴파일 시 참조하는 문서·템플릿·폰트/메트릭 자산,
  필요한 고지와 README만 선별한다. 일괄 폴더 제외로 `include_str!`/`include_bytes!`를 깨뜨리지 않는다.
- 패키지별 실제 목록·압축 크기·해시를 측정한다. 전체 PDF·샘플 corpus·개인 폰트·작업 증적을
  배포물에 넣지 않는다. 원본 저장소의 자료를 삭제하거나 ignore 정책으로 숨기지 않는다.
- 완료 조건: 지원 계약 초안과 dependency graph, 필요한 파일·고지 및 패키징 실패 원인이 설명된다.

### 2단계 — 저장소 밖 소비 검증

- 승인된 graph에 맞춰 manifest의 이름·버전·path와 registry 위치를 정리한다.
- 아직 게시되지 않은 상호 의존 crate는 별도 임시 registry 등으로 **실제 포장한 source**의
  의존성 해석을 검증한다. 부모 저장소 path나 사용자 Cargo 캐시의 직접 수정으로 대체하지 않는다.
  임시 registry의 통과와 실제 crates.io 등록 상태는 구분한다.
- 저장소 밖 소비 앱에서 HWP/HWPX 열기·내용 확인·저장 왕복, 지원하는 PDF 입력,
  WASM 초기화/feature 경계와 Native CLI 설치·이미지/터미널 출력을 실제 실행한다.
- 현재의 git/path 소비 사례와 같은 입력·출력을 비교한다. 의존성 이름 변경도 보안 patch와
  실제 PDF 출력·자원 제한의 보존을 검증하며, 출력이 바뀌면 적용 시각 게이트를 수행한다.
- 지원 입력의 성공뿐 아니라 미지원 codec·암호 PDF·손상 xref·자원 초과의 거부와 실패 후
  정상 요청을 Native/WASM 소비자에서 확인한다. PDF 기본 포함은 전체 할당 증명이나 비신뢰
  입력 배포 준비의 완료가 아니므로 #7911의 잔여 보안 게이트 처리 방침도 게시 전에 확정한다.
- docs.rs와 README 예제, `cargo install`의 대상 bin·`native-skia` 옵션을 검증한다.
- 완료 조건: root checkout 없이 추출 패키지로 지원 사용자 여정이 실행되고 필요한
  Rust/WASM/소비자 검증 및 영향 범위의 출력 비교가 충족된다.

### 3단계 — 게시 운영 준비

- 실제 업로드가 없는 preflight와 명시적인 publish 경로를 분리해 같은 source SHA를 검사한다.
- 패키지별 `cargo package`/`cargo publish --dry-run` 결과, dependency 순서, 압축 크기,
  source·버전·고지·digest를 완료 증적으로 연결한다.
- 미게시 dependency 때문에 실제 crates.io 대상 부모 dry-run이 불가능한 초기 상태는 명시한다.
  임시 registry 검증 뒤 실제 게시 승인 단계에서 자식의 등록을 확인하며 부모 dry-run을 순차 완료한다.
- 초회 수동 게시 후 Trusted Publishing을 설정하는 순서를 준비한다. 토큰을 코드·문서에
  기록하지 않고 채널별 이미 게시됨·신규 게시·실패 상태와 부분 복구 절차를 정의한다.
- 기존 release aggregate를 확장한다면 publish 전후·verify-only·부분 실패 계약을 검사한다.
  실제 게시를 수행하지 않은 상태를 채널 배포 완료로 보고하지 않는다.
- 완료 조건: 게시자가 그대로 검토·실행할 수 있는 운영 절차와 명령, 실패 복구 및
  필요한 권한 목록이 있고 원격 적용 전 필수 검증이 완료된다.

## 실제 0.9.0 릴리즈와의 연결

제품 전체 버전 동기화·CHANGELOG·main 이력 연결·5플랫폼 release smoke·promotion preflight는
별도 0.9.0 릴리즈 준비 작업이다. 이 문서의 package 목록 검사가 이를 대체하지 않는다.
미병합 PR #8116과 취소한 메인터너 보정을 준비 범위에 다시 포함하지 않는다.

준비 완료 후에도 실제 crate 게시·토큰/Trusted Publisher 설정·workflow dispatch·태그와
정식 릴리즈는 대상·source·결과를 제시한 뒤 승인된 범위에서 수행한다.

## 이슈 등록 결과

열린 crates.io·0.9.0 이슈의 중복 검색 결과는 없었다. #5446의 운영 결정 재검토,
#4375/#4496의 비활성화 이력, #7911 B08과 #7875/#7981을 본문에 연결해 #8174를 등록했다.
API로 담당자·milestone·labels와 UTF-8 본문의 바이트 일치를 확인했다.
기존 이슈의 재개·close나 다른 PR 변경은 수행하지 않았다.
