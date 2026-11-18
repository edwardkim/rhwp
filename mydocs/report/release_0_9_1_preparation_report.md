# v0.9.1 릴리즈 준비 보고서

상태: 로컬 준비·필수 검증을 완료했다. 원격 push·PR·workflow dispatch·main 승격·tag·registry 게시·스토어 제출은 아직 수행하지 않았다.

## 고정 기준과 실제 실행 소스

- 사용자 승인으로 기능 마감 기준을 devel `197ce19e99438c2cba452c654ae1af504dab85ec`에 고정했다. 이후 새 기능 PR은 다음 릴리즈로 넘긴다.
- 작업 브랜치: `release/0.9.1-preparation-20261118`.
- Rust 전체 검증·Linux CLI·Cargo 소비 소스: `c0f9ea46f4a5693323e51c6be2cd9a9368f3311d`.
- 최종 Studio 보정·fresh WASM·브라우저·보호 검증 소스: `5167f63a26b7ad3d9111f08e3e82307be6f309e6`.
- 두 소스 사이 Rust·Cargo·toolchain·asset 입력 차이는 0개다. 실제 실행 SHA를 영수증에 그대로 보존했다.
- main `7213ed1f96266b95e22cd5645a07851c9c18298e`, v0.9.0 `c9095c5f67632e6721e800ccc7fb43ea49d26682`.
- [수행 계획](../plans/release_0_9_1_20261118.md), [국문 노트](../release/release_notes_v0.9.1.md), [영문 노트](../release/release_notes_v0.9.1_EN.md), [검증 원장](../tech/investigations/release-v0.9.1/README.md).

기능 마감 전에 #9168·#9170·#9157·#9171을 반영했다. #9157 때문에 Rust 입력이 달라진 뒤 전체 필수 검사를 다시 실행했다. #9157의 미완료 보호 검증도 이번 준비에서 실제로 완료했으며 이전 병합 지시를 릴리즈 예외로 사용하지 않았다.

## 준비한 변경

제품·npm·VS Code·Studio·Chrome·Firefox·Safari manifest와 관련 lock root를 0.9.1로 동기화했다. README·PDF 개발자 안내·Native 안내·Chrome/Edge/Firefox 제출 초안과 게시 가이드를 갱신했다. 국·영문 CHANGELOG의 v0.9.0 이하 부분은 바이트 단위로 보존했다.

deprecated Atomic 호출 13곳의 승인된 보정을 원 이력 그대로 포함했다. MSRV 1.93.1, CAS 메모리 순서·자원 상한·ID 고갈 계약을 유지한다. runner 종료 exit 143의 원인을 deprecated 경고로 설명하지 않는다. 새 제품 조판 알고리즘은 이번 릴리즈 준비에서 추가하지 않았다.

검증 중 실제로 발견한 두 초기화 경계는 원인을 보정했다.

- Chrome 설정 응답 채널이 닫히면 callback 안에서 `runtime.lastError`를 확인하고 선택적 배지·미리보기를 임의로 켜지 않는다. 실제 production content script의 오류/정상/비활성 대조와 기존 worker 종료 E2E를 통과했다.
- Studio 준비 직후 iframe에서 파일을 열 때 startup 빈 문서가 요청을 취소하던 경합을 공통 `DocumentOpenSession`으로 보정했다. 이전 요청 취소·첫 화면 완료·실패 후 복원은 보존했다. 기존 E2E 수정 전 FAIL, 수정 후 재시도 없이 3회 PASS와 최종 SDK·PDF·자동 복구 검증을 확인했다.

## 기여자와 패키지

v0.9.0 이후 고정 기능 기준까지 1,238개 commit의 Git 저자·공동 저자·원 PR·통합 출처를 감사했다. 사람 19명과 Dependabot을 국·영문 변경 이력과 노트에 반영했다. 미분류 저자·번호는 0건이다. 원 cherry-pick 참조 341개 중 340개 객체를 확보했고 미확보 객체의 통합 Git 저자는 보존했다. 원시 이메일은 ignored output에만 두었다.

| 새 게시 후보 | 버전 |
| --- | --- |
| rhwp | 0.9.1 |
| rhwp-codec-resource | 0.1.1 |
| rhwp-ooxml-chart | 0.1.1 |
| rhwp-hayro-ccitt | 0.3.0-rhwp.2 |
| rhwp-hayro-jbig2 | 0.3.0-rhwp.2 |
| rhwp-hayro-jpeg2000 | 0.4.0-rhwp.2 |
| rhwp-qcms | 0.3.0-rhwp.2 |
| rhwp-pdf-import-backend | 0.3.78-rhwp.2 |

변경 없는 contracts·password-crypto·JPEG encoder/decoder·zune-jpeg·svg2pdf 6개는 기존 공개 archive 그대로 재사용한다. 원 source SHA·checksum을 정책에 고정하고, 재사용 버전이 사라져도 새 업로드로 바꾸지 않는다. 공개 상태·소유자·checksum 사전 검사는 통과했으며 실제 업로드는 없다.

## 실제 로컬 검증

Ubuntu 24.04.5/GLIBC 2.39, Rust 1.93.1, nightly-2026-11-17, Node 24.15.0을 사용했다. 공용 `target/pr-review`의 Cargo 명령은 순차 실행했다. Rust 회귀는 실제 27개 공개 프로그램만 노출한 최소 글꼴 환경에서 실행했다. 정본 보호 시각 검증의 로컬 원본 글꼴 공급은 이 회귀 환경과 구분한다.

| 항목 | 결과 |
| --- | --- |
| fmt·Native/WASM Clippy·workspace build/Clippy | PASS |
| 전체 회귀 | 13,724 PASS, 실패 0, 기존 skip 45 |
| Native Skia 공식 3종 | 4,073 PASS |
| Native 글리프 복구 집중 검사 | 24 PASS |
| doctest | 8 PASS, 기존 ignored 3 |
| nightly workspace/all-targets | PASS, deprecated 진단 0 |
| release/crates Python 계약 | 81 PASS |
| Studio 전체 | 2,811 PASS, 실패·skip 0 |
| editor 단독/선언 계약 | 36/3 PASS |
| TypeScript·production build | PASS |
| 표준 release fresh WASM | PASS, 혼합 crate·manifest LTO·실제 wasm-opt |
| Canvas/direct PDF·iframe·대화창·PDF 대기·호스트 글꼴·첫 화면 E2E | PASS |
| Chrome·Firefox 실제 확장 열기·편집·저장 | PASS |
| npm tgz 별도 소비·VSIX·확장 ZIP payload | PASS |
| Linux CLI archive version·capability·PNG 생성 | PASS, 최대 GLIBC 2.39 |
| Cargo 정규화 archive·격리·소비 | 14개, 6개 별도 소비 프로젝트 PASS |
| Cargo 게시 사전 검사 | 8개 pending approval·6개 already-present, 업로드 없음 |
| AMO 소스 ZIP | 추출 소스의 실제 냉 재컴파일·wasm-opt·Firefox 포장·재빌드 확장 실행 PASS |
| 정본 보호 전쪽 | Native 1,771쪽 + fresh WASM 1,771쪽, 하락·누락 0행 |
| 편람 HWPX 정식 `--handbook-gate` | Native 383쪽 PASS |

정본 보호의 정밀 raw 최저는 Native **98.86043%**, fresh WASM **98.86472%**다. 공식 두 자리 TSV와 정밀 metrics, 입력·독립 PDF·DPI·compat·브라우저·실제 공급 글꼴의 해시를 보존했다. 과거 공급 목록 1,002개와 환경 설정 7개를 재구성했고, 누락됐던 중첩 표 설정은 과거 SHA-256과 정확히 같은 파일로 복원했다. 비공개 글꼴 프로그램은 Git이나 배포물에 넣지 않았다. 90% 판정·공식 하락 금지·기존 보호값에 예외를 사용하지 않았다.

Cargo 소비는 업로드 API 없는 loopback sparse registry와 별도 프로젝트에서 실행했다. 저장소 path/patch 의존성을 쓰지 않으며 14개 archive의 manifest/lock에 로컬 registry·머신 경로가 유출되지 않았다. 최소/기본/PDF 구성·JBIG2/JPX/ICC/xref 예제·한도 오류/재시도·HWPX 저장 재열기·코덱 제외·guide를 확인했다. 미게시 0.9.1의 공개 소비 완료를 뜻하지 않는다. 실제 게시 후 공개 registry·Docker 검증을 별도로 수행한다.

이번 AMO 재빌드는 원 작업공간과 JS·WASM 및 Firefox 파일의 해시가 모두 같았다. 소스 무결성·실제 컴파일·재빌드 확장의 동작도 검증했다. 다른 호스트·도구 버전까지의 일반적인 비트 단위 재현을 주장하지 않는다. 최종 기록 commit은 실행 소스 뒤의 문서·증거 기록이다. 실제 배포물은 승인된 최종 main/tag에서 다시 만든다.

## 진단 복구와 남는 한계

Native sandbox의 WSL DNS 연결 파일 누락은 해당 파일만 읽기 전용으로 공급해 복구했다. GitHub 공식 Skia archive의 전체 다운로드·digest도 대조했다. 최소 글꼴 노출은 유지했다. 자동 suite 생성물이 오래돼 focused filter가 0건을 선택한 경우는 생성물을 다시 만들고 대상 검사를 실제 실행했다. 전체 회귀에 사용한 모듈 2,384개의 내용·집합은 동일함을 검증했다. loopback의 이전 미게시 preview lock과 브라우저 실행 경로 누락도 하네스를 보정해 실제 재실행했다. 최초 실패를 PASS로 덮거나 제품 기대값을 낮추지 않았다.

보고용 PDF compatibility 4쪽의 기존 경고는 남는다. #9168의 실제 CI와 PDF raster 7개·direct gate 3개 비율을 대조한 근거를 별도 보존했다. 이 보고용 수치는 독립 한컴 Print 전쪽 검증을 대신하지 않는다. Chrome·Firefox·Safari manifest의 버전 외 필드는 v0.9.0과 같고, 실제 production 경로의 추가된 HTTP literal·네트워크 호출은 0건이다. 모든 사용자 흐름의 전수 네트워크 검증을 주장하지 않는다.

## 준비 PR의 배포 문서 보정

[#9173의 첫 lint](https://github.com/edwardkim/rhwp/actions/runs/44259474754/job/134649184729)에서 라이선스 고지의 프로젝트 버전과 VS Code 변경 이력의 제목 형식이 기존 릴리스 계약에 맞지 않아 실패했다. 최초 로컬 실행의 검사 목록에 이 계약 모듈이 빠져 있었다. 두 문서의 표기를 맞추고 라이선스 고지에 실린 내부 코덱 버전을 Cargo의 실제 버전과 일치시켰다. 동일한 CI 단계의 workflow 계약 643건, 고지 파일 84개 생성, 두 문서의 링크 검사가 통과했다. 제품 코드·빌드 입력 의존성·검사 기준은 그대로이며 앞선 영수증은 실제 실행 소스를 보존한다.

## devel 통합과 main 계보 동기화

[#9173](https://github.com/edwardkim/rhwp/pull/9173)은 리뷰와 해당 SHA의 필수 CI 확인 후 devel `d50dacc348d9533661378f40339b62381f0dd354`에 병합됐다. main에 별도로 병합된 0.9.0 릴리즈와 [#8884 보안 백포트](https://github.com/edwardkim/rhwp/pull/8884)의 계보는 병합 전에 devel에 합쳐지지 않았다. 승격 게이트는 main이 후보 devel의 조상이어야 하므로 승격 실행 전에 양쪽 계보를 합친다.

main 보안 코드 `7bbe651832`와 devel의 선행 보정 `913713dbfd`는 stable patch ID가 같다. 네 충돌에서는 최신 devel의 qcms NEON 테스트 조건·PDF ToUnicode 패치 설명·최종 보안 검토 기록을 보존했다. main에만 존재하던 기존 검증 문서 여섯 개를 가져왔으며 `mydocs/` 밖의 파일은 #9173 병합본과 바이트가 같다. 이후 12개 승격 검증의 대상은 이 동기화가 통합된 devel SHA다.

## 다음 원격 단계

1. 구체적인 후보와 PR 본문·원격 실행안에 대해 승인받은 뒤 devel Open PR·해당 head CI·self-review·merge를 진행한다.
2. 최종 소스의 main 대비 승격 inventory를 고정하고 executable surface 15개를 다루는 고유 evidence workflow 12개의 실제 검증을 완료한다. inventory 조사만으로 실행 성공을 대신하지 않는다.
3. main PR의 CI·promotion gate·요구 승인 리뷰를 완료한다. v0.9.0의 CI/리뷰 예외를 이월하지 않는다. #9154는 main 적용·실제 실행 관찰 뒤 종료를 판단한다.
4. 별도 승인된 main/tag에서 CLI 5플랫폼·Pages·npm 2종·VS Code/Open VSX·새 crates 8개를 배포하고 공개 exact version·checksum·provenance·Rust/CLI/WASM 소비를 검증한다.
5. Chrome·Edge·Firefox의 제출·심사·공개 상태를 구분해 기록한다. CI 변경을 포함한 승격 PR의 후속 기록은 옵션 2로 처리한다.
6. 종료 시 정식 증거를 보존하고 devel 동기화·소유한 branch/worktree·output/debug 정리를 수행한다. 현재는 다음 단계에 필요한 산출물을 유지한다.
