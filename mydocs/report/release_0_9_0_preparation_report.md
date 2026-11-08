# v0.9.0 릴리즈 준비 결과 — 2026-11-08

## 후보와 범위

`release/0.9.0-preparation-20261108`에서 devel 대상 준비를 완료했다. 기능 기준은
`5077b216cbc51d2046da07350de562f6f521a7c1`이며, 이전 배포는 v0.8.7
`1a76570e833917d15817415a53c09ad61ab3203f`다. 준비 중 추가된 #8741도 반영했다.
Rust 제품 구현·Cargo 의존성·Studio/shared 구현은 이 devel과 동일하다. 변경은 안내·기여 감사·
배포 포장·승격 검증 계약이다. 로컬 산출물을 정식 게시 완료로 간주하지 않는다.

## 제공할 안내

- [한국어 릴리즈 노트](../release/release_notes_v0.9.0.md)와
  [영문 릴리즈 노트](../release/release_notes_v0.9.0_EN.md), 두 CHANGELOG를 갱신했다.
- [PDF 개발자 시작 안내](../manual/pdf_developer_onboarding.md)와
  [영문 안내](../manual/pdf_developer_onboarding_EN.md)에 CLI·Rust·WASM·Worker·Studio/iframe
  진입점, 암호, 선택 쪽, 진행률, 한도, 폰트와 지원 경계를 연결했다.
- CLI 압축 파일에도 `PDF_QUICKSTART.md`와 영문판을 넣는다. PNG 출력 인자는 디렉터리이며
  PDF 출력 인자는 파일인 실제 차이를 명시했다.
- npm core/editor, VS Code, Chrome/Edge 제출 초안과 기존 PDF 매뉴얼의 낡은 opt-in 설명을 정리했다.
  2026-11-08 추가 지시에 따라 crates.io 초회 배포도 0.9.0 릴리즈에 포함한다. 기존 MD01의 별도 일정 결정을 갱신했다.

## 기여자 감사

v0.8.7 이후 기능 기준까지 4,852개 커밋의 저자·공동 저자·원 PR·cherry-pick 출처와
GitHub 계정을 대조했다. 사람 **33명**, 별도 bot **1개(Dependabot)**이며 미해결 identity/번호는 0건이다.
한국어·영어 CHANGELOG 및 릴리즈 노트 네 곳의 명단·기준 SHA·커밋 수를 계약 검사로 확인한다.

[공개 원장](../tech/investigations/release-v0.9.0/contributor-ledger.json),
[별칭과 판정 근거](../tech/investigations/release-v0.9.0/contributor-overrides.json)에 증거를 남겼다.
공개 원장에는 이메일을 넣지 않았다. 조회할 수 없는 cherry-pick 원본 9개는 통합 커밋의 저자를
유지하며 한계를 명시했다. 최종 승격에 다른 기여가 추가되면 원장을 다시 산출한다.

## 배포 보정

1. 본체·Studio·npm·확장 manifest/lock의 0.9.0을 확인했다. 내부 0.1 패키지와 fork 버전은 유지한다.
2. `@rhwp/editor`의 실제 npm 포장에서 빠져 있던 MIT LICENSE를 추가했다. Native CLI·npm core에는
   기존 라이선스·저작권·font metric 고지 84개를 원문 그대로 동봉한다. 글꼴 프로그램은 추가하지 않았다.
3. crates.io source 후보의 JPEG encoder 중복을 제거했다. 이전 #8360 정리가 #8389에서 중복된
   이력을 확인하고 두 include 목록의 합집합과 의존성을 보존했다. 고유 후보는 14개다.
   #8565에서 Cargo include에 추가된 Native 글꼴 원장이 source 후보 대장에서 빠진 것도 발견했다.
   runtime 계약으로 누락 실패를 재현한 뒤 후보에 포함해 다시 통과시켰다.
4. main과 devel 사이의 끊어진 ancestry를 파일 변경 없는 merge로 연결했다. 이 준비 PR은
   **merge commit**으로 통합해야 하며 squash/rebase merge로 대신하지 않는다.
5. 승격 inventory의 누락된 실행 표면 6개를 실제 caller·direct·contracts-only·verify-only 증거에
   연결했다. Layout advisory는 skip/build 실패/timeout을 성공 증거로 인정하지 않게 보강했다.
   검사·보안 정책을 비활성화하지 않았다. 실제 원격 실행은 아직 남아 있다.
6. 실제 VSIX에서 사용하지 않는 `media/rhwp.js`·`media/rhwp_bg.wasm`이 발견되어 제외했다.
   실행 경로는 `dist/media`이며 현재 WASM만 남긴 실제 VSIX를 다시 검사했다.
7. Chrome/Firefox E2E의 첫 실패는 #1328 → #1454의 글꼴 동의창에서 사용자 선택을 기다린 것이었다.
   저장소의 기존 동의 계약을 유지하며 실제 대체 글꼴 버튼을 누르고 문서 준비 상태를 기다리도록
   하네스를 보정했다. 최초 실패와 수정 후 성공 모두 보존했다. 제품 UI·검사 상한은 바꾸지 않았다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 릴리즈 manifest·명단·포장 계약 | 72건 통과; 버전 0.9.0, 33명 명단·기준 SHA·커밋 수 일치 |
| 승격 Python / CI impact Node 계약 | 각각 105건 / 187건 통과; inventory 22개 실행 표면의 미등록 0건 |
| source archive·릴리즈 계약 | 24건 통과; 14개 source 후보·manifest blocker 0건 |
| Studio / editor | 2,754건 통과·기존 skip 2건 / 36건 통과; TypeScript 검사 통과 |
| Native release | locked·native-skia 빌드 성공; 실제 압축 파일의 버전·PNG 출력 확인 |
| CLI PDF | 15개 예상 결과 일치: 두 배치 모드·쪽 선택·R2–R6 암호·오류 시 출력 보존·저장 재열기·PNG/PDF |
| Rust 시작 예제 | 실제 라이브러리에 링크해 컴파일·PDF → HWPX 실행 성공 |
| 최적화 WASM | standard release rlib+cdylib·실제 wasm-opt 확인; 26,678,316바이트 |
| WASM PDF / 실제 Worker | 271개 assertion·17개 실제 Worker·12쪽 저장/렌더 통과; 기능 제외 빌드는 별도 미실행 |
| WASM 브라우저 소비 | 실제 응답 SHA 일치; Canvas 3건·direct PDF 호환성 3건 통과 |
| 보고 전용 PDF 비교 | 4쪽·경고 4건·오류 0건. 기본 96/72 DPI 크기 차이 포함; 독립 Print 일치율 판정으로 사용하지 않음 |
| 실제 npm core 소비 | tgz에서 @rhwp/core import·0.9.0·PDF 가져오기·HWPX 저장/재열기 본문 보존 |
| Chrome / Firefox 확장 | 전체 3개 suite 통과 / 다운로드 3개·단일 뷰어·편집 저장 보존 |
| 배포 파일 | 8개 구성·버전·해시 검사 통과; core/VSIX/Chrome/Firefox의 RHWP WASM 동일, Chrome=Edge ZIP |
| Firefox 심사용 소스 ZIP | 소스만 추출·locked WASM 최적화·Firefox bundle 성공; 모든 링크 대상 포함 |

[공개 검증 요약·artifact 해시](../tech/investigations/release-v0.9.0/local-validation.json)에
빌드 SHA와 실제 검사 범위를 기록했다. WASM 빌드 소스는 `5064570b3f`다. 이후 변경은 VSIX의
미사용 사본 제외, E2E 글꼴 선택 처리와 source archive 대장 보정이며 컴파일되는 제품 입력은 같다.
소스 ZIP 재빌드 시작 후 세 파일의 대장/E2E 변경도 실제 소스와 파일별로 대조했다.
정식 태그의 배포 파일은 원격에서 최종 승격 SHA로 다시 생성·검증한다.

검증 원시 로그와 로컬 배포 파일은 `output/release-v0.9.0/`에 있다. 큰 빌드 파일·PDF·임시 로그는
커밋하지 않는다. Rust 안내 예제는 실제 Native 빌드 라이브러리로 컴파일·실행했다. 별도 최소 feature
Cargo 소비 빌드를 이번에 다시 완료했다는 뜻은 아니다. crates.io 검사는 source archive 검사이며
실제 registry 게시·Cargo registry 소비 검증과 구분한다.

## 원격 처리와 남은 절차

1. **devel 준비 PR 완료:** [#8757](https://github.com/edwardkim/rhwp/pull/8757)의 head
   `d778ab7bfd28c858c638d7f2c38238bea5a58412`에서 필수 CI와 self-review를 완료하고
   `0e6cfe91df7aabb886319346f78af568def25a64` merge commit으로 통합했다.
   Native의 checkout 시간 초과는 사용자 승인에 따라 해당 job·종속 집계만 1회 재실행해 해소했다.
   전체 회귀 12,777건·CI 선택 Native 237건·Studio 2,769건이 통과했다. 최신 #8269까지의 제품 소스를
   그대로 보존한 병합과 main ancestry를 확인했다. [최종 검토](../pr/archives/pr_8757_review.md),
   [CI 증거](../pr/assets/pr8757/ci-results.json)를 별도 기록 후보로 준비했다.
   CI 정의 변경의 [옵션 2](../manual/pr_review/post_merge.md)에 따른 이 기록 PR의 원격 반영은 남았다.
2. 통합된 devel의 exact SHA로 [승격 preflight](../manual/github_operations.md#76-workflow-promotion-preflight)를
   실행한다. 원격 full CI/CodeQL, direct/adapter 실행과 5플랫폼 Release Binary·WASM·VSIX verify-only
   결과를 확인한다. `#8745`의 보호 브랜치 전체 보안 분석도 여기서 실제 결과를 확인해야 한다.
3. 성공 후 devel → main 릴리즈 PR을 진행한다. 원격 SHA가 달라지면 기여자·변경 범위·실행 증거를 갱신한다.
4. 승인된 main SHA에 `v0.9.0` 태그를 게시하고 GitHub Release 및 npm core/editor·VS Code·Open VSX의
   실제 게시 결과를 확인한다. **crates.io 초회 게시도 0.9.0의 배포 단계에서 수행**한다.
   최종 source의 14개 `.crate`·의존 순서·checksum을 확인하고 수동 게시 뒤 공개 registry의
   Rust·CLI·WASM 소비와 docs.rs 결과를 확인한다. Chrome/Edge/Firefox 스토어 제출과 Safari 검증은
   해당 채널의 환경·절차를 따른다.

조회 시 npm 두 패키지의 latest는 0.8.7이고, npm·VS Code Marketplace·Open VSX의 0.9.0은 미게시다.
게시 직전에 다시 확인한다. 토큰 값·권한·보호 규칙은 변경하지 않았다.

## 공개 전 확인할 제한

- 제품 조판 코드를 바꾸지 않아 이번 준비에서 전체 조판 회귀와 편람을 중복 실행하지 않았다.
  기능 기준을 포함한 정식 PR CI 및 승격 full 검증은 원격 단계에서 완료한다.
- Studio npm audit에는 기존 개발 도구 경고 13건(High 2)이 남는다. High는 Chromium BiDi의 zod,
  Babel/core-js 도구 경로의 browserslist다. `--omit=dev` 조회는 0건이지만 이를 모든 번들·개발 환경의
  무결점 보안 보증으로 해석하지 않는다. lockfile은 임의로 올리지 않았다.
- 문서 metadata의 기존 오류 16건은 기준 devel과 동일하고 신규 오류는 0건이다.
- Linux 외 공식 Native 플랫폼, 실제 스토어 설치·심사, 신규 릴리즈 registry 소비는 로컬 성공으로
  대신하지 않는다. 게시 날짜와 채널별 완료 표시는 실제 성공 뒤 확정한다.


## 2026-11-08 문서 보완과 crates.io 결정 반영

#8757 병합 뒤 메인테이너는 배포 문서의 0.9.0 현행화를 지적하고,
crates.io 배포도 0.9.0부터 시작하도록 결정했다. 사용자 배포 문서는 main 승격 전에 devel에서
완성하며, 기존 MD01의 별도 일정 결정을 이번 지시로 대체했다.

- 국·영문 README·CHANGELOG·릴리즈 노트, PDF 온보딩, crates.io 소비·게시 가이드와
  패키지 README에 초회 게시·공개 registry 소비 확인을 0.9.0의 범위로 반영했다.
- Chrome·Firefox README의 최신 항목을 0.9.0으로 보완하고 0.8.7 변경 이력은 보존했다.
  [Firefox AMO 0.9.0 심사 안내](../feedback/firefox-0.9.0_amo_notes.md)를 신설했으며
  [게시 가이드](../manual/publish_guide.md)에 네 스토어 제출 문서와 반영 순서를 연결했다.
- 과거 What's New 제안은 제출용 안내와 구분했다. 예전 0.8.7 제출 파일은 바꾸지 않았다.
- 준비 중 병합된 #8754·#8269의 기능을 국·영문 변경 목록에 추가했다. 해당 저자들은 기존
  33명 명단에 포함되어 있다. 최종 승격 SHA의 기여 원장 재감사는 게시 전 수행한다.
- 문서 19개 링크 검사, 기존 버전·기여자·배포 계약 검사 6건, 공백 검사를 통과했다.
  Chrome·Firefox manifest의 권한·CSP·Firefox 최소 버전은 v0.8.7과 같음을 실제 대조했다.
- 제품 코드·feature·workflow·버전·토큰을 변경하지 않았다. 본 문서 후보의 원격 PR·CI·머지와
  최종 14개 `.crate` 검증·실제 게시·공개 소비 확인은 아직 남아 있다.
