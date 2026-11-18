# v0.9.1 릴리즈 준비 보고서

상태: 로컬 검증 마무리 중. AMO source ZIP의 실제 재컴파일·Firefox 재빌드 결과를 기다린다.
원격 push·PR·workflow dispatch·main 승격·tag·registry 게시·스토어 제출은 아직 수행하지 않았다.

## 기준과 범위

- 작업 브랜치: `release/0.9.1-preparation-20261118`.
- 검증·로컬 포장 source: `8a01b6529c98c0f7fc53750dbe22ab86467e1350`.
- 기능 기준 devel: `1695089d21fb5ff116eeba263e4935f04b4ac494`.
- main: `7213ed1f96266b95e22cd5645a07851c9c18298e`.
- 이전 tag `v0.9.0`: `c9095c5f67632e6721e800ccc7fb43ea49d26682`.
- [수행 계획](../plans/release_0_9_1_20261118.md), [국문 노트](../release/release_notes_v0.9.1.md),
  [영문 노트](../release/release_notes_v0.9.1_EN.md).

준비 도중 devel에 병합된 #9168 한 건은 충돌 없이 반영했다. 해당 Studio 소스는 devel과 같으며,
실제 E2E에서 공급자 없음·빈 공급자는 쪽 트리 재조회 0회, 글꼴 공급자는 입력 20회에 편집 쪽만
20회 조회하고 조합·붙여넣기·Undo/Redo와 3쪽 보존을 확인했다.

최초 Rust 검증 source `830d17a308839794237bfbeed3ad6aff42e6124c`부터 위 검증 source까지
Rust 파일·Cargo manifest/lock·toolchain 입력은 같다. 중간 변경은 Python 게시 사전 검사와
문서, 이미 병합된 Studio 기능이다. 각 영수증의 실제 실행 SHA를 보존하며 다른 SHA에서
실행했다고 바꾸지 않는다. Linux CLI는 `57ad9ac277c95c7612021a8e359c8907ad5fe5f7`에서
컴파일하고 위 source에서 문서·고지와 함께 포장했다. 최종 보고서·가이드 commit은 이 검증
source 뒤의 문서 기록이다. 실제 배포물은 승인된 최종 main/tag에서 다시 만든다.

## 준비한 변경

제품·npm·VS Code·Studio·Chrome·Firefox·Safari manifest와 관련 lock root를 0.9.1로
동기화했다. 이전 버전의 변경 이력·실행 결과·기여자 기록은 보존했다. 새 릴리즈의 README,
PDF 개발자 안내·CLI 시작 안내와 Chrome·Edge·Firefox 제출 초안을 갱신했다.

승인된 deprecated Atomic 보정 13곳을 원 이력 그대로 포함했다. 공통 CAS helper를 사용하며
MSRV 1.93.1, 자원 상한·ID 고갈·메모리 순서 계약을 유지한다. 이번 준비 diff에서 조판·렌더러
알고리즘은 새로 수정하지 않았다. 누적 조판 개선의 원 PR은 릴리즈 노트에서 연결한다.

공개 archive의 실제 소스·checksum·VCS 정보와 현재 후보를 대조해 아래 8개 새 버전을 준비했다.

| 패키지 | 새 버전 |
| --- | --- |
| rhwp | 0.9.1 |
| rhwp-codec-resource | 0.1.1 |
| rhwp-ooxml-chart | 0.1.1 |
| rhwp-hayro-ccitt | 0.3.0-rhwp.2 |
| rhwp-hayro-jbig2 | 0.3.0-rhwp.2 |
| rhwp-hayro-jpeg2000 | 0.4.0-rhwp.2 |
| rhwp-qcms | 0.3.0-rhwp.2 |
| rhwp-pdf-import-backend | 0.3.78-rhwp.2 |

변경 없는 contracts·password-crypto·JPEG encoder/decoder·zune-jpeg·svg2pdf의 기존 6개는
공개 archive 그대로 재사용한다. source-policy에 원 source SHA와 checksum을 고정했다.
게시 사전 검사는 재사용 버전이 공개 registry에서 사라졌을 때 새 업로드로 전환하지 않는다.
새 버전 8개와 재사용 6개의 소유자·공개 상태·checksum 검사가 통과했으며 실제 업로드는 없다.

v0.9.0 이후 1,225개 commit의 Git 저자·공동 저자·원 PR·통합 출처를 감사했다. 사람 19명과
Dependabot을 국·영문 CHANGELOG·노트에 반영했고 미분류 저자·번호는 0건이다. 원 cherry-pick
source 341개 중 객체 340개를 확보했다. 미확보 객체의 통합 Git 저자는 보존했고, 과거 issue
참조 2개와 identity 판정 근거는 [공개 원장](../tech/investigations/release-v0.9.1/contributor-ledger.json)
및 overrides에 남겼다. 원시 이메일은 Git 제외 output에만 보관한다.

## 로컬 검증

Ubuntu 24.04.5/GLIBC 2.39, Rust 1.93.1, nightly-2026-11-17, Node 24.15.0을 사용했다.
공용 target은 기본 작업공간 `target/pr-review`이며 Cargo 명령은 순차 실행했다. 16 logical CPU·
31 GiB RAM 환경에서 build jobs 8·nextest threads 12를 사용했다. Rust 회귀는 공개 최소 글꼴
환경에서 실행했다.

| 항목 | 결과 |
| --- | --- |
| fmt·Native/WASM Clippy·workspace build/Clippy | PASS |
| 전체 회귀 | 13,723 PASS, 실패 0, 기존 skip 45 |
| Native Skia 공식 3종 | 4,073 PASS |
| doctest | 8 PASS, 실패 0, 기존 ignored 3 |
| nightly workspace/all-targets | PASS, deprecated 진단 0 |
| release/crates Python 계약 | 81 PASS |
| Studio 전체 | 2,808 PASS, 실패·skip 0 |
| editor 단독/선언 계약 | 36/3 PASS |
| TypeScript·Studio production build | PASS |
| 표준 release fresh WASM | PASS, rlib+cdylib·manifest LTO·실제 wasm-opt |
| 배포 WASM Canvas/direct PDF | PASS, direct PDF 3쪽 실패 0 |
| About·공통 창·PDF 대기·iframe·호스트 글꼴 입력 E2E | PASS |
| Chrome 확장 E2E | smoke·download·lifecycle 3묶음 PASS |
| Firefox 확장 E2E | viewer 1개·다운로드/저장 3회·편집 보존 PASS |
| npm tgz 실제 소비 | core 초기화·PDF 지원 조회·HWPX 재열기·editor 통신 PASS |
| VSIX·확장 ZIP payload | 0.9.1·LICENSE·동일 JS/WASM·불필요 파일 검사 PASS |
| Linux CLI archive | version·PNG capability·실제 PNG 생성 PASS, 최대 GLIBC 2.39 |
| Cargo source 사전 검사 | 14개 PASS, manifest blocker 0 |
| Cargo 정규화 archive·의존성·소비 | 14개 포장, 최소/기본/PDF 구성·PDF 예제·코덱 제외·guide PASS |
| Cargo 게시 사전 검사 | PASS, 8개 pending approval·6개 already-present, 업로드 없음 |
| AMO source ZIP 실제 재컴파일·Firefox 포장 | 진행 중 |

Studio 첫 실행의 Node 전용 WASM 미공급 skip 2개는 같은 release raw WASM에서 Node 바인딩을
생성해 전체 suite를 다시 실행했고 최종 skip 0을 확인했다. 추가 소비 하네스의 Puppeteer
모듈 경로와 PDF 예제의 consumer feature 지정 오류도 보정 후 재실행해 통과했다. 제품 코드나
검사 기대값은 이 하네스 복구로 바꾸지 않았다. 최초 실패와 완료 영수증을 구분해 보존한다.

Cargo 소비는 업로드 API 없는 loopback sparse registry와 별도 프로젝트에서 실행했다.
저장소 path/patch 의존성을 쓰지 않으며 14개 archive의 정규화 manifest/lock에 localhost
registry 설정·머신 의존 경로가 없음을 검사했다. PDF 예제는 JBIG2·JPEG 2000·ICC·xref 복구,
정책 한도 오류와 정상 재시도, HWPX 저장·재열기를 확인했다. 이는 미게시 0.9.1의 공개 registry
소비 완료를 뜻하지 않는다. 게시 후 별도 공개 소비·Docker 검증이 남는다.

PDF browser/compatibility 보고용 4쪽에는 경고가 남는다. 기존 #9168의
[실제 CI 증거](https://github.com/edwardkim/rhwp/actions/runs/44215150230)와 대조해 PDF raster
이미지 7개는 같고, direct gate의 3개 비율도 0.00757801·0.00269187·0.01882513으로 같다.
CI·로컬 대조에서 KTX의 Canvas 참조 이미지는 0.81634% 차이가 남는다. 이 보고용 수치를 독립 한컴 Print 전쪽
시각 일치율로 사용하지 않는다.

Chrome·Firefox·Safari manifest의 버전 외 필드는 v0.9.0과 같다. 웹 production 소스의 추가된
줄에서 새 HTTP literal·네트워크 호출은 0건이며 Chrome E2E의 요청/프록시 guard도 통과했다.
모든 사용자 흐름의 네트워크 동작을 전수 실행했다는 주장은 하지 않는다. 새 production 글꼴
프로그램은 없으며 테스트용 글꼴 fixture만 범위에 추가됐다. 실제 package에는 기존 라이선스
고지를 포함했다.

## 원격 단계와 종료 조건

1. 준비 PR을 devel에 제출해 해당 head CI·self-review·병합을 완료한다.
2. 최종 exact devel에서 main 대비 workflow inventory를 다시 생성하고 정책이 지정한 승격
   검증을 실행한다. 현재 inventory는 executable surface 15개·고유 evidence workflow 12개,
   정책 위반 0이다. 이 조사 결과는 workflow 실행 성공 영수증을 대신하지 않는다.
3. main PR의 CI·promotion gate와 요구 승인 리뷰를 완료한다. 기존 0.9.0의 CI/리뷰 예외는
   적용하지 않는다. #9154는 main 적용·실제 실행 관찰 뒤 종료 여부를 판단한다.
4. 승인된 main/tag에서 실제 CLI 5플랫폼·Pages·npm 2종·VS Code/Open VSX·crates 8개 새 버전을
   배포한다. local preview archive를 다른 source의 게시물로 사용하지 않는다.
5. 각 채널의 공개 exact version·checksum·provenance·Rust/CLI/WASM 소비를 확인하고
   Chrome·Edge·Firefox 제출·공개 상태를 구분해 기록한다. CI 변경을 포함한 main 승격 PR의
   후속 기록은 옵션 2로 처리한다.
6. 필요한 증거를 정식 경로에 보존하고 devel 동기화·소유한 branch/worktree·output/debug
   종료 정리를 수행한다. 현재는 검증·원격 단계에 사용할 산출물을 유지한다.
