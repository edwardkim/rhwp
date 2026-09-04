---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_6534_impl.md
issue: 6534
stage: 4
date: 2026-09-01
---

# Task M100 #6534 Stage 4 — packaged 다운로드·전체 회귀 검증 결과

## 1. 결과 요약

격리된 Chrome for Testing에 실제 unpacked `rhwp-chrome` 확장을 적재하고, 브라우저가 발생시킨
download ID·저장 완료·최종 파일명·뷰어 탭 수를 함께 확인하는 독립 E2E를 추가했다.

| 실제 다운로드 | 최종 파일명 | 저장 | rhwp 뷰어 탭 |
| --- | --- | ---: | ---: |
| 정상 XLSX URL | `normal.xlsx` | 1 | 0 |
| `.hwp` URL + XLSX filename/MIME | `public-report.xlsx` | 1 | 0 |
| 확정 HWP | `document.hwp` | 1 | 1 |
| extensionless + HWP MIME/body | `extensionless` | 1 | 1 |

네 건 모두 같은 전체 실행에서 통과했다. 따라서 #6534의 XLSX false positive는 실제 파일 다운로드를
방해하지 않으면서 탭 생성을 0으로 만들었고, 확정 HWP와 기존 #198 extensionless fallback은 각각
정확히 한 탭을 유지했다.

## 2. packaged Chrome E2E 경계

`rhwp-chrome/e2e/download-interceptor.test.mjs`는 다음 조건을 고정한다.

- 임시 Chrome profile과 download 폴더를 매 실행마다 새로 만든다.
- loopback fixture server 외 요청은 차단 proxy로 보내 외부 네트워크에 의존하지 않는다.
- `Browser.downloadWillBegin`과 `Browser.downloadProgress`로 실제 download GUID와 완료를 확인한다.
- 저장된 파일이 존재하고 비어 있지 않은지 확인한다.
- 각 case 전 fixture tab을 다시 활성화해 앞선 viewer tab이 다음 사용자 클릭을 가로막지 않게 한다.
- 진단 시 `RHWP_EXTENSION_DOWNLOAD_CASE`로 한 case만 재현할 수 있다.
- 성공·실패와 무관하게 browser, server, 임시 profile/download를 정리한다.

초기 실행에서는 `puppeteer`가 설치되지 않아 빌드 뒤 E2E 진입 전에 멈췄다. `rhwp-chrome`의 lockfile로
`npm ci`를 수행한 뒤 재실행했다. 첫 연속 실행에서 앞선 HWP viewer tab이 활성화되어 마지막 fixture
click이 대기한 것은 extensionless case 단독 실행이 통과하는 것으로 제품 결함과 분리했고,
`fixturePage.bringToFront()`와 click timeout을 넣어 전체 네 건을 결정적으로 만들었다. 중단한 두 진단
실행의 임시 폴더도 제거했으며 최종 확인에서 잔여 `rhwp-download-e2e-*` 폴더는 0개다.

## 3. 전체 회귀가 드러낸 두 기존 test 문제

### 3.1 threat-scan 합성 HWPX

첫 full release-test는 8,881건 중 8,875건 통과, 6건 실패였다. 그중 5건은 기존 threat-scan helper가
HWPX라고 만든 ZIP에 `Contents/header.xml`을 넣지 않았고 일부에는 `Contents/content.hpf`도 없어서,
Stage 3의 엄격한 구조 판정이 올바르게 `Unknown`으로 거부한 사례였다.

메인테이너가 승인한 계획 변경에 따라 제품 detector를 느슨하게 하지 않고 두 helper가 누락된 필수
entry만 최소 XML로 보완했다. 제공된 entry는 중복하지 않으며 위협 payload·finding cap·출력 계약과
테스트 수는 그대로다. focused 결과는 threat-scan 9/9, CLI threat-scan 5/5 PASS다.

### 3.2 schema 정책 문서 연결

fixture 정정 뒤 두 번째 full release-test는 8,880/8,881건이 통과했다. 남은
`policy_path_points_to_existing_document`는 전체 병렬 실행에서만 runtime `Path::exists()`가 실패했고,
같은 release binary의 단독 실행과 `rhwp-contracts` 전체 15건은 반복 통과했다. 대상 파일은 Git 추적
상태와 inode·mtime이 변하지 않았고 #6534 제품 변경과의 인과도 없었다.

두 번째 승인된 계획 변경에 따라 registry의 `policy`를 canonical 상대경로와 exact 비교하고, 내부
비배포 crate에서 같은 파일을 `include_str!`로 compile-time 연결했다. 문서가 없거나 이동하면 컴파일이
실패하고 registry 문자열만 drift하면 assertion이 실패하므로 원래 불변식을 런타임 파일 조회 없이 더
결정적으로 유지한다.

첫 fixture 수정 직후 파생 suite mapping을 다시 준비하지 않은 focused 명령이 0건을 선택한 진단 오류도
있었다. 이 결과는 증적으로 사용하지 않았고 정본 `--prepare` 뒤 실제 14건을 통과시켰다. 파생 suite와
manifest는 제출 대상에 포함하지 않는다.

## 4. 최종 검증

### Rust

| 검사 | 결과 |
| --- | ---: |
| `cargo fmt --all -- --check` | PASS |
| native Clippy `-D warnings` | PASS |
| WASM lib Clippy `-D warnings` | PASS |
| workspace build | PASS |
| workspace all-target Clippy `-D warnings` | PASS |
| integration manifest | PASS — 1,083 sources / 4,731 attrs / 48 targets |
| source unit tier | PASS — 4,223 tests / 299 modules |
| 세 번째 full release-test | **8,881/8,881 PASS**, 46 skipped, 309.178 s |

전건 진행은 진단과 교정을 숨기지 않기 위해 다음과 같이 보존한다.

| 실행 | 통과 | 실패 | 판단 |
| --- | ---: | ---: | --- |
| 1차 | 8,875 | 6 | 합성 HWPX 5건 + 비인과 schema 1건 발견 |
| 2차 | 8,880 | 1 | 합성 HWPX 회복, schema runtime lookup만 잔존 |
| 3차 | 8,881 | 0 | 두 승인 교정 뒤 전체 병렬 green |

로컬 nextest는 저장소 권장 0.9.140보다 낮은 0.9.137이라
`profile.ci-duration-observation.junit.report-skipped`를 모른다는 경고를 냈다. 테스트 선택·실행은 정상이며,
최종 summary가 8,881건 전체를 확인했다.

### WASM·Studio·확장

| 검사 | 결과 |
| --- | ---: |
| Docker WASM release build + `wasm-opt` | PASS — 6m 10s |
| 공유·Chrome·Firefox service-worker | 131/131 PASS |
| Studio document signature | 17/17 PASS |
| Studio 전체 test | 1,328 PASS, 1 policy skip, 실패 0 |
| Studio production build | PASS |
| Chrome production build | PASS |
| Firefox production build | PASS |
| frontend-extension dist 계약 | PASS |
| packaged Chrome smoke | PASS |
| packaged Chrome 실제 download E2E | 4/4 PASS |

모든 프런트엔드와 확장 검증은 Stage 3 Rust detector가 포함된 최신 Docker WASM `pkg`를 만든 뒤
수행했다. 빌드 중 Vite의 기존 native config·chunk size 경고는 있었지만 오류는 없었다.

## 5. 변경 범위와 보호 불변식

Stage 4가 추가·정정한 source는 다음으로 한정된다.

- `rhwp-chrome/e2e/download-interceptor.test.mjs`
- `rhwp-chrome/package.json`
- `tests/threat_scan_contract.rs`
- `tests/cases/threat_scan_cli_contract.rs`
- `crates/rhwp-contracts/src/schema_registry.rs`

다음 경계는 유지했다.

- 다운로드한 XLSX 파일은 취소·삭제·변형하지 않는다.
- HWP filename의 즉시 열기와 extensionless HWP terminal fallback을 보존한다.
- 사이트별 URL blacklist와 browser manifest 권한을 추가하지 않는다.
- 사용자 Chrome profile, 외부 공공기관 사이트와 private corpus를 사용하지 않는다.
- dist, `pkg`, generated integration suite·manifest를 stage하지 않는다.
- 제품 detector를 test fixture에 맞춰 완화하지 않는다.

## 6. Stage 4 판정과 다음 게이트

Stage 4는 **qualified-packaged-download-boundary**다.

- 단위 adapter뿐 아니라 실제 packaged Chrome 다운로드와 탭 생성 결과가 기대와 일치한다.
- Rust 전체 8,881건과 네이티브/WASM lint, 최신 Docker WASM, Studio와 양쪽 확장 검증이 모두 green이다.
- full 회귀가 발견한 기존 test 문제는 원인을 분리하고 승인된 최소 교정으로 해결했으며 우회하지 않았다.

다음 단계는 Stage 5 문서·결산이다. 브라우저 확장 개발 문서에 metadata 우선순위, defer/terminal 규칙,
ZIP 후보와 Rust HWPX 확정 경계를 반영하고 최종 보고·PR 제출 범위를 정리한다. 별도 승인 전에는 Stage 5
문서나 원격을 변경하지 않는다.
