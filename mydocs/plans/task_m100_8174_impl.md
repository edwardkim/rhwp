# #8174 crates.io 배포 준비 구현 계획

Issue: [#8174](https://github.com/edwardkim/rhwp/issues/8174)

- [수행 계획](crates_io_preparation_20261023.md)
- 기준 source: `def947c2149df5bc812d801e6e79be19a403b4d1`
- 현재 승인 범위: 이슈 등록·공통 선행 작업과 A01–A07의 로컬 기술 보완 범위 v1. 원격 게시·push·PR 생성·실제 릴리즈는 포함하지 않는다.
- 2026-10-23 「PR 준비 하세요」 지시에 따라 제출 전 필수 로컬 검증·입력 보존·커밋과 PR 본문·생성 명령 초안을 준비했다. 실제 결과는 [2단계 보완 기록](../working/task_m100_8174_stage2_corrections.md#pr-제출-전-로컬-검증--2026-10-23)에 있으며 원격 push·PR 생성은 별도 승인 단계다.
- 추가 문서 작업: [메인테이너 결정 대장](task_m100_8174_maintainer_decisions.md)을 작성하고
  선택·조건·근거를 기록한다. 현재 MD02–MD04의 기술 보완 범위는 부분 확정이며 다른 정책은 미결정이다.
- 2026-10-23 추가 착수 승인: 정책 선택과 무관한 C01–C04의 로컬 사전 검사 도구·실행 증적을
  먼저 구현한다. 제품 manifest·feature·실제 게시 설정은 이 절편에서 변경하지 않는다.
- 2026-10-23 구조 재조사 뒤 수정 지점·우선순위 착수 승인으로
  [로컬 보완 범위 v1](task_m100_8174_maintainer_decisions.md#결정-뒤-구현-범위-확정-방법)을 확정했다.
  [A01–A07 보완 실행](../working/task_m100_8174_stage2_corrections.md)을 기존 기본값과 보안 경계를
  유지하며 진행한다. 아래 일반 후보 목록은 범위 v1에서 승인한 부분부터 적용한다.

## 1단계 — 실제 source에서 계약과 패키지 범위 확정

1. Cargo manifest와 `src/lib.rs`의 공개 소비 경로에서 graph·feature·버전·소유 경계를 기록한다.
   내부 crate를 본체로 흡수하는 대안과 별도 게시를 비교하고, 기존 재사용·보안 경계를 보존하는
   별도 게시를 우선안으로 구체화한다. 이 단계에서는 제품 manifest·기능을 바꾸지 않는다.
2. 현재 Cargo 기본 패키지 목록을 재산출하고, 별도 게시 후보 8개 각각의 source 파일 목록을 선별한다.
   원본 경로·Git blob과 compile-time include를 대조한다. 실행용 문서·템플릿·메트릭·폰트와
   고지는 보존하며 대형 검증 corpus와 작업 기록은 제외한다.
3. 선별 source를 실제 gzip tar로 포장해 파일 수·원본/압축 크기·SHA-256을 측정한다.
   이 archive는 **정규화 전 source 후보**로 명시한다. unpublished dependency와 manifest
   전환을 마치지 않은 source archive를 Cargo 검증 완료 `.crate`로 부르지 않는다.
4. 현재 manifest의 `cargo package` 차단점을 확인한다. 임의로 버전·publish flag를 풀거나
   보안 patch를 제거하지 않고, 2단계에서 바꿀 정확한 파일과 이유를 남긴다.
5. [1단계 기록](../working/task_m100_8174_stage1.md)에 graph·사용자 여정·실측·미완료 게이트를
   기록한다. 이번 단계에서 실제 실행하지 않은 소비자 빌드를 PASS로 표시하지 않는다.
6. 추가 요청의 [보안 제한 현황](../manual/security_feature_limits.md)을 코드·feature tree와
   대조한다. 제한·자원 한도·일반 미지원을 구분하고 현황 정리 중에 feature를 활성화하지 않는다.

제품 소스·Cargo.lock·배포 workflow가 그대로이면 해당 변경의 전체 Rust/WASM 재빌드는
이 단계 문서·source 목록 검사의 완료 조건이 아니다. 이후 실제 package 소비는 생략하지 않는다.

## 2단계 — 게시용 manifest와 독립 소비

### 공통 선행 절편 — C01–C04

1단계 임시 측정을 정식 재실행 가능한 source 사전 검사로 옮긴다. Git source, 후보 graph,
필수 내장 파일·라이선스/고지, 제외 자료, archive 목록·크기·해시와 manifest 차단점을 확인한다.
지원 정책을 선택하지 않고도 도구와 진단을 준비하며, 실제 Cargo package·외부 소비 검증과
구분한다. 로컬 실패 경계 검사와 실제 저장소 실행을 완료하고 2단계 기록에 결과를 남긴다.
이 절편의 완료 결과와 남은 경계는 [2단계 공통 기록](../working/task_m100_8174_stage2.md)을 따른다.

### 정책 선택 뒤의 구현

아래 항목은 1단계 권고안에 따른 후보 목록이다. 관련 MD 결정과 범위 버전·단계 진행 승인을
기록한 뒤 선택된 항목을 구현한다. 미결정 항목에 의존하는 변경을 먼저 시작하지 않는다.

- 본체의 3개 path 의존성에 registry 버전을 병기하고 별도 PDF export fork의 package 이름을 연결한다.
  내부 crate 0.1.0, 보안 backend/decoder의 upstream 기반 보정 버전과 본체 0.9.0은 구분한다.
- 승인된 패키지별 include·README·metadata·라이선스 경로를 반영한다. publish flag 해제는
  패키지 검증과 승인된 범위를 확인하고 수행한다. 이 단계도 실제 업로드를 하지 않는다.
- 미게시 source는 저장소 밖 임시 registry에 실제 포장해 dependency를 검증한다.
  repository path fallback이나 사용자 Cargo cache 수정으로 registry 소비를 대신하지 않는다.
- core HWP/HWPX 왕복, 기본 PDF 입력, CLI 기본/Native 기능, WASM `wasm-api` on/off 소비,
  docs.rs와 feature 조합을 고정 source에서 실행한다. MSRV는 실제 검사한 버전만 선언한다.
- 게시용 README에 보안 제한을 반영하고 PDF 미지원 codec·암호·손상 xref·자원 초과의 거부,
  실패 후 정상 요청을 실제 소비자에서 검사한다. backend upstream 기능 목록을 그대로 지원표로 쓰지 않는다.
- 전체 소스·의존성을 바꾸는 후보는 local_validation 4.3의 해당 lint·회귀·fresh WASM 게이트를
  완료한다. PDF 보안 한도와 기존 수정 출력도 소비자 쪽에서 대조한다.
- 새 devel의 #7488 opt-in cdylib-only와 #7474 standard release 산출물 동일성 계약을 유지한다.
  package 검증용 임시 manifest가 root 개발 manifest를 덮어쓰지 않도록 한다.

## 3단계 — 게시 운영과 릴리즈 인계

MD01·MD08·MD09와 확정한 범위에 따라 수동 게시 준비 또는 자동화 구현을 선택한다.

- exact SHA/version/digest preflight, 자식→부모 게시·이미 게시된 항목 조회·부분 실패 복구를 마련한다.
- 실제 registry 대상 부모 dry-run은 자식이 아직 없으면 완료할 수 없음을 기록한다.
  사전 임시 registry 검증과 실제 게시 승인 뒤의 최종 dry-run을 구별한다.
- 초회 수동 게시와 이후 OIDC 연결, owner·권한·yank 절차를 준비한다. 실제 인증 설정과 게시를
  단계 검증의 이름으로 실행하지 않는다.
- workflow·운영 script 변경은 github_operations의 비례 검증을 적용한다. 기존 네 채널을
  성공으로 오판하거나 새 채널을 태그 생성만으로 활성화하지 않도록 계약을 확인한다.

PR 준비·실제 게시 완료·제품 릴리즈 완료는 각각 해당 검증과 승인 이후에 판정한다.
