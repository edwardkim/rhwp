---
kind: snapshot
status: active
canonical: mydocs/manual/publish_guide.md
last_verified: 2026-11-09
---

# v0.9.0 배포 결과와 잔여 후속 처리

공개 배포와 Rust·CLI·WASM 소비 검증을 완료했다. 이 문서는 최종 결과를 정식 경로에
반영해 임시 `output/` 삭제 뒤에도 확인할 수 있게 한다. 태그 CI의 실패와 Chrome·Edge의
공개 전환 확인은 별도 잔여 항목이며, 릴리즈 전체의 모든 검사가 성공했다는 뜻은 아니다.
이 보고서와 증적은 [최종 기록 PR #8793](https://github.com/edwardkim/rhwp/pull/8793)의 제출 자료다.
문서 검토와 종료 범위는 [self-review](../pr/archives/pr_8793_review.md)에 기록했다.
개별 배포 검증의 관측 시각은 유지하며,
2026-11-09 후속 확인에서도 Release Binary 성공·태그 CI 실패 상태는 같았다.

## 확정 소스와 예외

- 정식 `v0.9.0` 태그·main: `c9095c5f67632e6721e800ccc7fb43ea49d26682`.
- main 승격 [PR #8783](https://github.com/edwardkim/rhwp/pull/8783)은 동일 소스의 성공한
  Render Diff 재사용과 승인 리뷰 수 요건 예외를 사용자에게 명시 승인받아 병합했다.
  실패 check를 성공으로 변경하지 않았다.
- [PR #8784](https://github.com/edwardkim/rhwp/pull/8784)는 crates.io에서 실제 거부된
  JPEG decoder 설명 누락을 보정했다. code head `4a765822988575099104c30081516b2bb7c4ddb5`,
  검토 head `9a9c6256ec926179e4745a8b780f5a780ae407fa`, devel merge
  `4aa8f5fc2b550145a724155cdb7cdd0b0a2576f3`다. [검토 기록](../pr/archives/pr_8784_review.md)에
  해당 CI와 self-review가 포함돼 있다.
- 최초 `rhwp-codec-resource`·`rhwp-contracts`는 정식 태그 소스, 나머지 12개는 승인된
  메타데이터 보정 소스에서 게시했다. 태그를 옮기지 않았으며 제품 Rust 코드는 같다.
  패키지별 source·버전·SHA-256은 [최종 증적](../pr/assets/release-0.9.0-execution/delivery-evidence.json)의
  `crates_packages`에 기록했다.
- 기여자 사람 33명과 원 기여 이력을 유지했다. 최종 태그까지 새 사람 identity가 없음을 확인했다.
  원시 이메일 목록은 이 보고서에 복사하지 않는다.

## crates.io 패키지별 게시 소스

소스 A는 정식 태그 `c9095c5f67632e6721e800ccc7fb43ea49d26682`,
소스 B는 메타데이터 보정 `4a765822988575099104c30081516b2bb7c4ddb5`다.
아래 버전의 공개 archive SHA-256·크기는 `crates_packages.packages`, docs.rs의 실제 빌드 결과는
[최종 증적](../pr/assets/release-0.9.0-execution/delivery-evidence.json)의 `docs_rs`에서 확인한다.

| 패키지 | 게시 버전 | 소스 |
| --- | --- | --- |
| [rhwp-codec-resource](https://crates.io/crates/rhwp-codec-resource/0.1.0) | `0.1.0` | A |
| [rhwp-contracts](https://crates.io/crates/rhwp-contracts/0.1.0) | `0.1.0` | A |
| [rhwp-jpeg-decoder](https://crates.io/crates/rhwp-jpeg-decoder/0.3.2-rhwp.1) | `0.3.2-rhwp.1` | B |
| [rhwp-jpeg-encoder](https://crates.io/crates/rhwp-jpeg-encoder/0.7.1-rhwp.1) | `0.7.1-rhwp.1` | B |
| [rhwp-ooxml-chart](https://crates.io/crates/rhwp-ooxml-chart/0.1.0) | `0.1.0` | B |
| [rhwp-password-crypto](https://crates.io/crates/rhwp-password-crypto/0.1.0) | `0.1.0` | B |
| [rhwp-svg2pdf](https://crates.io/crates/rhwp-svg2pdf/0.13.0-rhwp.1) | `0.13.0-rhwp.1` | B |
| [rhwp-zune-jpeg](https://crates.io/crates/rhwp-zune-jpeg/0.5.15-rhwp.1) | `0.5.15-rhwp.1` | B |
| [rhwp-hayro-ccitt](https://crates.io/crates/rhwp-hayro-ccitt/0.3.0-rhwp.1) | `0.3.0-rhwp.1` | B |
| [rhwp-hayro-jpeg2000](https://crates.io/crates/rhwp-hayro-jpeg2000/0.4.0-rhwp.1) | `0.4.0-rhwp.1` | B |
| [rhwp-qcms](https://crates.io/crates/rhwp-qcms/0.3.0-rhwp.1) | `0.3.0-rhwp.1` | B |
| [rhwp-hayro-jbig2](https://crates.io/crates/rhwp-hayro-jbig2/0.3.0-rhwp.1) | `0.3.0-rhwp.1` | B |
| [rhwp-pdf-import-backend](https://crates.io/crates/rhwp-pdf-import-backend/0.3.78-rhwp.1) | `0.3.78-rhwp.1` | B |
| [rhwp](https://crates.io/crates/rhwp/0.9.0) | `0.9.0` | B |

이 소스 분리는 0.9.0 초회 게시의 부분 실패를 복구한 명시적 예외다. 이후 릴리즈에서
서로 다른 소스를 임의로 섞는 절차로 사용하지 않으며, 게시 전 메타데이터·소비 검증은
[crates.io 가이드](../manual/crates_io_guide.md)를 따른다.

## 채널별 확인 결과

아래는 배포 중 수행한 실제 공개 조회·다운로드·실행 결과다. 관측 시각·출처·해시는
최종 증적의 각 항목에 있다. 종료 정리 시 모든 웹 채널을 다시 검사했다는 뜻은 아니다.

| 채널 | 확인 결과 |
| --- | --- |
| GitHub Release | 다섯 플랫폼 CLI·체크섬과 브라우저 제출 자료 공개. 플랫폼별 실제 PNG smoke와 Linux 공개 archive 실행 확인 |
| GitHub Pages | 공개 JS/WASM과 검증 artifact의 바이트 일치, 브라우저 열기·HWPX 저장·재열기 통과 |
| npm core/editor | 0.9.0 공개 설치, 서명·provenance와 실제 브라우저 열기·저장·재열기 통과 |
| VS Code Marketplace·Open VSX | 공개 0.9.0 VSIX와 CI artifact의 전체 바이트 일치 |
| crates.io·docs.rs | 14/14 게시·공개 index·archive checksum 확인, 지정 버전 docs.rs 빌드 14개 성공 |
| Firefox | 메인테이너 제출 완료. 공개 0.9.0 XPI의 코드·WASM·자원과 검증 배포물의 동일성 확인; Mozilla 서명 추가는 구분 |
| Chrome·Edge | 메인테이너 제출 완료. 마지막 조회에서 Chrome은 0.8.7, Edge HTML은 공개 버전을 확정하지 못함. 승인 후 공개 전환 확인 필요 |

Pages 첫 모듈 초기화에서 일회 오류를 관측했다. 이후 같은 환경과 새 context는 통과했으나
최초 응답 바이트를 확보하지 못해 원인을 확정하지 않았다. 이를 지속 재현 결함이나 완전한
미발생 증거로 확대하지 않는다.

## 공개 소비 검증

- 호스트 공개 registry pipeline은 `2026-11-08T14:09:02Z`에
  `all-public-consumers-passed`로 완료됐다. Rust 기본·최소·PDF 구성, CLI 3종,
  최적화 WASM 2종과 실제 브라우저 검사를 통과했다. loopback 소비와 구분한다.
- Docker 공개 소비는 `2026-11-08T14:53:20Z`에 32개 명령을 통과했다.
  Rust 1.93.1·Debian 12·Linux x86_64에서 새 Cargo home과 공개 패키지를 사용했다.
  제품 소스·호스트 인증 파일은 마운트하지 않고 공용 `target/pr-review`만 캐시로 공유했다.
- Docker 기본·Native CLI에서 메모리·CPU 제한과 파일시스템·네트워크 격리 적용을 확인했다.
  PDF 제외 구성은 `PDF_IMPORT_DISABLED`를 반환하며 기존 출력 파일을 보존했다.
- Noto Sans KR 명시·자동 탐색의 794×1122 PNG는 픽셀과 SHA-256이 같다.
  제목·본문·표 한글을 직접 판독했다. 이 검사는 독립 한컴 출력과의 조판 정확도 판정을 대신하지 않는다.
- 사용자의 지속 보존 지시에 따라 실행 환경·입력·결과·에이전트 매뉴얼은 Git에서 제외된
  `local/crates-io-consumption/`에 있다. 재실행 명령은
  `./local/crates-io-consumption/run.sh --version 0.9.0`이며, 다른 릴리즈는 버전을 바꾼다.
  로컬 원본 결과는 `local/crates-io-consumption/VALIDATION-0.9.0.md`에 있다.

## CI와 인증 복구

[Release Binary](https://github.com/edwardkim/rhwp/actions/runs/42470324565) attempt 4는 success다.
반면 [태그 CI](https://github.com/edwardkim/rhwp/actions/runs/42470324547) attempt 2는 failure다.
WASM의 Binaryen 다운로드 실패, Native 30분·Archive B 60분 컴파일 시간 초과가 남았다.
테스트 assertion·컴파일 오류와 구분하지만 미완료 검사를 통과로 표시하지 않는다.
추가 재실행이나 예외 판정은 수행하지 않았다.

npm 자동 배포는 `release-binary.yml`이 reusable `npm-publish.yml`을 호출한다.
실제 자동 배포가 성공한 Trusted Publisher의 workflow filename은 `release-binary.yml`,
Environment name은 `NPM_TOKEN`이다. Environment 이름과 장기 토큰을 혼동하지 않는다.
GitHub Secret 갱신만으로 caller 등록을 대신할 수 없었고, 사용자가 npm의 publisher를
등록한 뒤 실패 작업을 승인 범위에서 재실행해 게시했다. 토큰 값은 읽지 않았다.

[배포 가이드](../manual/publish_guide.md#npm-trusted-publisher-등록과-확인)에 자동 배포 caller와
독립 수동 배포의 publisher를 구분하고, 패키지별 등록값·environment·허용 동작과
`ENEEDAUTH` 확인 절차를 반영했다. 실제 workflow나 인증 설정은 이번 문서 PR에서 변경하지 않는다.

## 이슈와 종료 정리

- #8239: 반영·배포 고지를 재확인한 뒤 사용자 승인으로
  [완료 댓글](https://github.com/edwardkim/rhwp/issues/8239#issuecomment-6722154696)을 게시하고 close했다.
- #8745: 보호 devel의 실제 전체 [CodeQL 실행](https://github.com/edwardkim/rhwp/actions/runs/42458236082),
  analysis `2120844363`과 flow를 대조했다. 목표 RHWP logging은 0건이며 별도 vendor 36개·140개
  flow는 기존 원장과 같다. 사용자 승인으로 [완료 댓글](https://github.com/edwardkim/rhwp/issues/8745#issuecomment-6722224756)을 게시하고 close했다.
- #8784의 병합 후 [duration 갱신](https://github.com/edwardkim/rhwp/actions/runs/42477955945)은 success다.
- devel 동기화·전용 branch·output/debug 삭제 범위와 실제 확보 공간은
  [정리 기록](../working/release_0_9_0_cleanup.md)에 남긴다. 별도 #8296 작업공간과
  사용자가 보존을 요청한 로컬 소비 검증 환경은 유지한다.

이 후속 PR은 최종 기록·검증 증적과 npm 게시 안내 보완을 함께 제출한다.
별도 잔여 항목은 태그 CI 실패 처리 결정과 Chrome·Edge 공개 전환 확인이다.
문서 제출이나 임시 파일 정리를 이 두 항목의 완료로 간주하지 않는다.
