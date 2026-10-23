# #8174 1단계 — 게시 의존성과 최소 source 패키지

Issue: [#8174](https://github.com/edwardkim/rhwp/issues/8174)

## 단계 결과

**1단계 사전 분석·source 선별 완료. 실제 Cargo 패키지 검증과 게시 준비 전체는 미완료다.**

- 승인: 2026-10-23 사용자 「이슈로 등록해서 진행하도록 합니다」.
- 코드 기준: `def947c2149df5bc812d801e6e79be19a403b4d1`.
- 계획 커밋: `874d11eb9`.
- 브랜치: `chore/issue8174-crates-preparation-20261023`.
- [수행 계획](../plans/crates_io_preparation_20261023.md), [구현 계획](../plans/task_m100_8174_impl.md).
- [source 선별 정책](../../packaging/crates-io/source-policy.json)은 제안한 게시 이름·버전·순서와
  파일 범위를 선언한다. 제품의 Cargo include·version·publish flag에 아직 적용하지 않았다.

## 등록과 선행 해결 추적

중복 검색에서 열린 crates.io·0.9.0 이슈는 없었다. #5446의 과거 운영 보류 결정을 연결한
새 준비 이슈 #8174를 만들고 담당자 `edwardkim`, milestone `v1.0.0`, labels
`enhancement`·`rust`·`packaging`을 지정했다. API로 UTF-8 본문·metadata를 재확인했다.
기존 #5446이나 다른 작업의 상태는 바꾸지 않았다.

- #4375 → 원 PR #4376 → 통합 #4496 `dd9ecdc4b7d9535c6e0d5b5872d735f458a1694f`:
  원 PR은 직접 merge되지 않고 통합됐다. 당시 installer의 exact source 보정과 로컬 검사,
  crates.io 실제 실행 미완료는 기존 검토에 기록돼 있다.
- #4655의 운영 롤백과 #5446의 보류는 단순 token 누락이 아니라 채널 유지보수·공개 API의
  책임을 이유로 했다. 새 계획은 이를 포함하며 과거 workflow를 무조건 복원하지 않는다.
- #7875/#7876 → #7981의 실제 Rust/WASM 소비 계약을 유지한다. 기존 검토가 확인한 범위는
  git/path 소비자의 feature·start/export 개선이며 registry 게시 완료가 아니다.
- #7911 B08의 별도 보안 backend/decoder와 원본·고지를 유지한다. 보안 patch를 제거하거나
  upstream 원본 버전으로 되돌려 패키지 문제를 우회하지 않는다.
- 새 devel에는 #7488의 opt-in cdylib-only 빌드와 #7474의 standard release 실제 출력·산출물
  동일성 gate가 반영돼 있다. 이번 단계에서 이 코드·workflow·Cargo.lock을 변경하지 않았다.

## 게시 구조 선택

**기존 crate 경계를 유지하는 8개 별도 패키지**를 2단계의 우선 구현안으로 구체화했다.
본체에 흡수하면 `ir_schema`·`ontology`·`provenance`·`ooxml_chart`·`password_crypto`의 기존
공개 re-export와 도구/보안 코드 재사용 경계를 바꾸게 된다. 게시 준비를 위해 이 구조를 다시
작성하지 않는다. PDF export fork만 별도 package 이름으로 연결해 root `[patch]`의 비전파 문제를 해소한다.

| 순서 | 게시 이름 제안 | 버전 제안 | 내부 선행 의존성 |
| --- | --- | --- | --- |
| 1 | rhwp-contracts | 0.1.0 | 없음 |
| 2 | rhwp-ooxml-chart | 0.1.0 | 없음 |
| 3 | rhwp-password-crypto | 0.1.0 | 없음 |
| 4 | rhwp-jpeg-decoder | 0.3.2-rhwp.1 | 없음 |
| 5 | rhwp-zune-jpeg | 0.5.15-rhwp.1 | 없음 |
| 6 | rhwp-svg2pdf | 0.13.0-rhwp.1 | 없음 |
| 7 | rhwp-pdf-import-backend | 0.3.78-rhwp.1 | 두 decoder |
| 8 | rhwp | 0.9.0 | 내부 3개, PDF import backend, PDF export fork |

독립 leaf의 순서는 상호 교환 가능하다. 단순한 dependency 번호 정렬만으로 registry의
실제 등록 여부를 통과로 처리하지 않는다. 새 제안 이름 `rhwp-svg2pdf`는 API 조회 404였으나,
이 결과도 이름이나 게시 권한 확보를 뜻하지 않는다.

## Rust 소비 계약 초안

| 사용자 여정 | 보호할 계약과 2단계의 실행 대상 |
| --- | --- |
| Rust 라이브러리 | `parse_document`, `parse_document_with_password`, `model::Document`, `DocumentCore`, `serialize_document`로 HWP/HWPX 열기·내용 확인·저장 왕복. 기본 feature의 PDF 입력도 유지한다. |
| WASM 앱 링크 | `default-features = false`에서 자체 main/start를 유지한다. `wasm-api` opt-in과 기본 JS facade/export 계약을 기존 #7981 검증과 대조한다. |
| CLI | 설치 대상은 `--bin rhwp`로 명확히 한다. 기본 구성과 `--features native-skia`를 구분해 PNG·직접 PDF·Sixel/Kitty capability를 검증한다. |
| docs.rs·지원 Rust | 최초 검증 기준은 저장소 고정 Rust 1.93.1이다. 아직 더 낮은 MSRV를 보장하지 않고, 실측 후 `rust-version`과 docs.rs feature/target을 선언한다. |
| 0.9.x 호환성 | 문서화한 입력·반환·feature 기본값·기존 re-export의 보존을 검사한다. 전체 `pub` 표면의 장기 안정성이 검증됐다고 선언하지 않고, 구체적인 지원 정책·예제는 2단계에서 확정한다. |

이 표는 지원 계약의 초안이다. 이 단계에서 외부 소비 앱을 새로 실행했거나 API 안정성을
검증 완료했다고 주장하지 않는다.

## 파일 선별과 실제 측정

최신 작업 트리의 Cargo 기본 목록은 **50,373파일·원본 6,566,423,185바이트**다.
앞선 605ccfd92 기준 50,352파일과 구분한다. 1단계 정책만 미커밋인 상태라 최초 실행은
Cargo의 dirty guard에 막혔다. 제품 manifest·lock·소스가 HEAD와 같음을 확인하고 진단에 한해
`--allow-dirty`로 목록을 다시 산출했다. 이 옵션을 실제 게시 절차의 기본값으로 정하지 않는다.

선별한 source 파일은 전부 Git index의 blob과 바이트가 같음을 확인했다. 실제 gzip tar를
만들고 다시 열어 파일 수를 확인했으며 압축 파일 SHA-256과 파일별 원장도 보존했다.

| source 후보 | 파일 수 | 원본 바이트 | gzip tar 바이트 |
| --- | ---: | ---: | ---: |
| rhwp-contracts | 7 | 110,098 | 30,247 |
| rhwp-ooxml-chart | 10 | 424,378 | 102,722 |
| rhwp-password-crypto | 3 | 43,060 | 11,337 |
| rhwp-jpeg-decoder | 24 | 226,384 | 51,602 |
| rhwp-zune-jpeg | 36 | 380,714 | 91,154 |
| rhwp-svg2pdf | 24 | 202,371 | 51,512 |
| rhwp-pdf-import-backend | 409 | 22,625,290 | 6,814,747 |
| rhwp | 1,792 | 39,821,247 | 8,329,278 |

**위 결과는 Cargo 정규화 전의 source archive이며, 검증된 `.crate`가 아니다.**

source archive는 모두 10,000,000바이트 미만이었다. 실제 `.crate`의 manifest·lockfile·파일
집합과 압축 크기는 2단계에서 확인한다. 이 압축값을 crates.io의 최종 크기 검사나
`cargo publish --dry-run`의 성공으로 바꾸어 주장하지 않는다.

실행 시 참조하는 `src/` 밖의 주요 14개 참조에서 MCP 문서·수식 폰트·숫자 메트릭·
`saved/blank2010.hwp`를 확인했다. 마지막 HWP는 `create_blank_document_native`가
실제로 내장하는 빈 문서 템플릿이므로 제외하지 않는다. `src/renderer/svg/tests.rs`와
`src/wasm_api/tests.rs`는 부모의 `#[cfg(test)]` 아래에 있음을 확인했다.
게시 패키지에 저장소의 회귀 suite/corpus 전체를 포함하는 계약으로 확대하지 않는다.

root의 폰트·README 고지와 backend 내장 폰트의 기존 고지를 source 후보에 보존했다.
LICENSE·NOTICE와 원본 출처 정보를 유지한다. 사유 폰트, PDF·문서 corpus, output과
계획·보고서를 후보에 포함하지 않음을 파일 집합 검사로 확인했다.

## 실제 Cargo 패키지 작성의 차단 지점

```bash
CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review \
  cargo package -p rhwp --no-verify --offline --locked --allow-dirty
```

exit 101. `rhwp-contracts`의 version requirement 누락으로 manifest 검증이 중단됐다.
처음의 dirty guard 중단은 이 결함의 재현으로 세지 않고, 진단 조건을 갖춘 재실행으로 확인했다.
이 경로에서는 실제 crate 작성·빌드까지 진행하지 못했다. `publish=false`를 해제하거나 성공으로
표시하지 않고, 2단계에서 version/registry 지정을 보정할 정확한 대상으로 특정했다.

## 검증과 증적

ignored `output/pr-review/issue8174/`에 다음 자료를 보존했다. 중간 JSON·로그·archive는 커밋하지 않는다.

- `baseline-package-list.*`, `baseline-package-size.json`: 최신 목록·크기·실행 명령과 exit.
- `root-package-blocker.*`: manifest 차단 지점. 최초 dirty guard와 재실행을 구분한다.
- `measure-source-packages.py`, `source-package-measurements.json`, `*.files.json`:
  재현 가능한 gzip tar 생성, Git blob·파일 해시·archive digest.
- `*.source.tar.gz`: 실제 crate와 구분한 8개 source 후보.
- `source-policy-audit.json`: 의존 순서·8개 패키지·주요 14개 내장 참조·제외 집합·라이선스 존재 확인.
- `external-include-audit.json`: `src/` 밖의 literal 참조 진단.
  1차 추출의 `after_first_cfg_test`는 완전한 Rust cfg 분석이 아니며 부모 모듈·구현 위치를 별도로 확인한다.

본체 source archive SHA-256:
`903a505b194a5a7aa52d3aa3a6827c361373b78435149c9e65438698351c47ce`.
PDF import backend source archive SHA-256:
`15946d51ded36554a41b6551a7653e60662de81a249856e9550c9c7f9e296dcc`.

Rust production·Cargo.toml·Cargo.lock·workflow는 변경하지 않았다. 문서와 source 선별 정책의
범위이므로 이 단계에서 전체 Rust 회귀·새 WASM 빌드·Visual Sweep을 반복하지 않는다.
패키지 소비·PDF 보안/출력·새 WASM 빌드 계약의 실제 검증은 2단계 필수 항목으로 유지한다.

변경 문서 3개의 상대 링크 검사, source 정책의 JSON 구문 검사, `git diff --check`는 통과했다.
고정 base와 현재 작업 트리의 제품 manifest·lock·Rust/vendor source·배포 workflow·WASM wrapper가
같음을 확인했다. 이 단계는 패키지 구성 제안과 source 실측의 완료이며 실제 배포 게이트 통과가 아니다.

## 다음 단계

먼저 [메인테이너 결정 대장](../plans/task_m100_8174_maintainer_decisions.md)에 선택·조건을
기록해 구현 범위를 확정한다. 8개 패키지는 아직 우선 권고안이다. 확정한 패키지·지원 계약에
맞춰 2단계의 manifest·README·정규화한 crate와 독립 소비 검사를 구체화하고 단계 진행 승인을
기록한다. 1단계 source 후보는 배포 준비 전체 완료가 아니다. 원격 게시 작업은 수행하지 않았다.

## 보완 — 보안 제한 기능 현황 (2026-10-23)

작업지시자의 보안 제한 기능 정리 요청에 따라
[보안 제한과 기능 사용 범위](../manual/security_feature_limits.md)를 추가했다. 1단계의 지원 계약
보완이며 2단계 manifest 변경을 시작하거나 기능을 활성화한 것이 아니다.

- 검토 source: `e5086b4bab13b8267ee5b9d0104afd3c24a9a160`.
  제품 source는 기존 기준 `def947c21`과 같으며 그 뒤 두 commit은 문서·source 정책 변경이다.
- 보안 경계: PDF JBIG2·JPEG 2000·ICC CMM feature 미활성, 제한된 constructor의 손상 xref
  재구성 거부, SVG 글리프의 정적 replay 경계, 링크 URI 제한, Studio 양식 명령 단추와
  보호 문서의 평문 recovery draft를 실제 소비 위치에서 확인했다.
- 조건부 거부: PDF·HWP5·HWPX의 자원·깊이 한도를 별도로 기록했다. 암호 PDF·DRM 등 일반
  지원 경계, `publish=false`, 렌더러 선택, 보고 전용 보안 검사를 보안상 기능 차단과 구분했다.
- 이력 정정: `git log -S`로 PDF 기본 포함의 원 PR #7953·통합 `b7a538803`을 확인했다.
  #7911 백로그의 현재 상태에 남아 있던 “기본 비활성” 문구를 정정했다. 당시 단계 기록은 보존했다.
- 게시 계약: upstream README의 전체 기능 목록을 그대로 보장하지 않으며, 별도 게시될 backend의
  legacy 무제한 API와 rhwp 요청 예산 경로를 구분한다. 게시용 README와 외부 Native/WASM
  소비자에서 거부·실패 후 정상 요청을 검증하는 항목을 기존 계획에 연결했다.

실행 결과와 증적은 ignored `output/pr-review/issue8174/security-feature-limits/`에 있다.

| 명령/확인 | 결과 | 증적 |
| --- | --- | --- |
| `cargo tree --offline --locked -e features -i rhwp-pdf-import-backend` | exit 0. 본체 기본 `pdf-import` 포함, backend 추가 feature 없음 | `pdf-backend-features.{command.json,stdout.txt,stderr.txt}` |
| 같은 명령에 `--target wasm32-unknown-unknown` 지정 | exit 0. 기본 WASM도 같은 feature 경계 | `pdf-backend-features-wasm.*` |
| `node --test rhwp-studio/tests/autosave-manager.test.ts rhwp-studio/tests/static-svg-path-layers.test.ts rhwp-studio/tests/hyperlink-target.test.ts` | exit 0. 22 PASS / 0 FAIL / 0 skip | `studio-security-contracts.*` |
| `gh api repos/edwardkim/rhwp/issues/8174`의 상태·담당자 조회 | open, `edwardkim`. 읽기 전용 조회 | `issue8174.*` |
| 새 매뉴얼과 연결 문서 7개의 상대 링크·언어·변경 범위, `git diff --check` | PASS. 별도 canonical 링크 검사도 6개 문서에서 통과 | `document-check.json` |

이 보완은 문서만 바꾸므로 제품 source·Cargo manifest/lock·workflow·패키지 선별 정책은
그대로다. 1단계 source archive 실측을 변경하지 않는다. 코드/feature 검토와 기존 helper 검사
22건을 PDF codec·브라우저·전체 보안 감사의 새 실행 결과로 확대하지 않는다. 외부 패키지의
실제 소비와 잔여 보안 게이트는 다음 단계의 필수 검증으로 유지한다.

## 보완 — 메인테이너 결정 절차 (2026-10-23)

사용자의 결정 기록·구현 범위 확정 지시에 따라
[결정 문서](../plans/task_m100_8174_maintainer_decisions.md)에 MD01–MD09의 선택지·권고안·
구현 영향을 정리했다. 현재 제한된 기능 F01–F08은 개별 해제 범위를 선택할 수 있게 구분했다.
기존에 확정된 제품 버전 0.9.0·준비 착수·결정 절차와 아직 선택하지 않은 기술 정책을 분리했다.

결정 대장의 선택·조건·발언 근거와 구현 범위 v1은 미확정으로 두었다. 제품 source·manifest·
feature·source 선별 정책은 변경하지 않았으며, 2단계 구현이나 원격 적용도 시작하지 않았다.
변경 문서 5개의 상대 링크 대상, 결정 9개·제한 기능 8개의 식별자, 미결정 상태와
`git diff --check`를 확인했다. 문서만 변경했으므로 기존 제품 검사를 반복하지 않았다.
