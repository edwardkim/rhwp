---
kind: pr-review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
pr: 6547
issue: 6534
author: edwardkim
---

# PR #6547 review - XLSX 다운로드 HWP 자동 열기 오탐 제거

## 결론 - 승인

[PR #6547](https://github.com/edwardkim/rhwp/pull/6547)은 공공기관 다운로드 handler의 URL·MIME에
HWP 힌트가 남아 있어도 확정 XLSX metadata를 우선하고, 일반 OOXML ZIP을 HWPX로 확정하지 않도록 두
방어 계층을 교정한다. self-review 대상 code candidate는
`108d612810bbd32c4a313de887586351db47835e`, 통합한 base는
`upstream/devel@891e395bb962627333262f110fc79354f44a2dbd`다. 선행 후보 `8a7af6a18`의 base-aware CI가
`src/parser/mod.rs` source unit test 2건 증가를 거부했고, 두 회귀를 새
`tests/cases/issue_6534_download_format_boundary.rs` 원본으로 이동해 로컬 blocker를 해소했다. 수정
code head의 GitHub Full CI·CodeQL·Render Diff·Adapter inter-diff·Proptest가 모두 성공했으므로
code candidate를 승인한다. 이 기록의 review-only trailing head가 trusted fast-pass와 required aggregate를
통과하고 최신 mergeability가 clean인지 확인한 뒤 별도 merge 승인을 받는다.

## 검토 경로와 metadata

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- self PR이므로 reviewer를 지정하지 않는다.
- 작성 시점 참고값: Open, base `devel`, head `task_m100_6534`, `MERGEABLE/CLEAN`.
- 변경 규모: 26 files, +2,179/-45, 9 commits(최신 devel 병합 checkpoint 1개 포함).
- 1,000행 초과 대형 PR이므로 즉시 admin merge하지 않고 code candidate·review-only head 검증과
  별도 merge 판단을 순차로 수행한다.
- 관련 이슈: #6534. PR 본문의 `closes #6534`는 merge 뒤 자동 종료를 의도한다.

## 변경 경계

- 공유 다운로드 판정은 `intercept/defer/ignore` 3상태와 확정 filename·MIME 우선순위를 소유한다.
- Chrome/Firefox adapter는 `onCreated`의 저신뢰 후보를 보류하고 filename 확정 또는 terminal event에서
  재조회한다.
- Studio는 ZIP 시그니처를 `zip` 후보로만 전달하고, Rust core가 HWPX 필수 entry 두 개로 최종 확정한다.
- packaged Chrome download E2E, 브라우저별 unit 계약, Rust·Studio 회귀와 canonical 매뉴얼을 포함한다.
- renderer/layout/paint, PDF·sample·golden·baseline, workflow, Cargo.lock과 공개 API는 바꾸지 않는다.
- generated integration suite·manifest, `dist`, `pkg`, `target`, private corpus와 실제 사이트 payload는
  포함하지 않는다.

## self-review 결과

### 다운로드 판정과 observer

- DEXT5 재요청 불가 handler를 가장 먼저 거부하고, 확정 HWP filename은 generic·잘못된 MIME보다
  우선하는 기존 수용 범위를 유지한다.
- 확정 `.xlsx` filename과 구체 OOXML MIME은 URL·finalUrl·HWP MIME 힌트보다 우선하므로 문제의
  false-positive 경로를 닫는다.
- URL/MIME만 HWP인 `onCreated` 후보는 `defer`되고, filename 확정 또는 terminal recheck에서만 최종
  분류된다. 확정 XLSX가 된 후보는 탭을 열지 않고 정상 다운로드도 취소하지 않는다.
- Chrome의 download id별 processing promise와 양쪽 adapter의 handled/terminal 상태를 유지해 중복 탭과
  #1498 과거 다운로드 재처리 방어를 보존한다.
- extensionless HWP MIME은 terminal에서만 수용해 #198 fallback을 유지한다.

### ZIP/HWPX 형식 경계

- Studio의 `zip`은 transport 후보일 뿐 HWPX 성공 판정이 아니며, 일반 ZIP도 공통 WASM core까지 전달돼
  `Unknown`으로 거부된다.
- Rust `has_required_package_entries()`는 payload를 압축 해제하지 않고 ZIP 중앙 디렉터리의
  `Contents/content.hpf`와 `Contents/header.xml`을 exact lookup한다. 두 entry 중 하나만 있거나 XLSX·손상
  ZIP이면 HWPX parser에 진입하지 않는다.
- 추적된 ZIP HWPX 525/525와 비밀번호 표본을 통과해 관측 코퍼스의 false-negative를 만들지 않았다.
- 전건 회귀에서 드러난 기존 threat-scan 합성 fixture의 필수 entry 누락은 detector를 완화하지 않고
  유효한 HWPX package로 교정했다.
- 병렬 회귀에서만 실패하던 schema 정책 경로 검사는 `include_str!`로 compile-time 연결해 원래 존재·비어
  있지 않음 계약을 결정적으로 보존한다.

### packaged browser 증적

- loopback fixture와 격리 Chrome profile/download directory를 사용해 실제 browser download event,
  저장 파일, `chrome.downloads` id와 viewer tab 수를 교차 확인한다.
- 정상 XLSX와 `.hwp` URL의 XLSX는 각각 저장 1·viewer 0, 확정 HWP와 extensionless HWP는 각각
  저장 1·viewer 1이었다.
- 외부 네트워크·사용자 profile에 의존하지 않으며 browser/server/임시 directory를 종료 시 정리한다.

첫 self-review에서 발견한 차단 finding은 1건이었다. `issue6534_detect_format_does_not_promote_xlsx_zip_to_hwpx`와
`issue6534_detect_format_requires_both_hwpx_package_entries`를 `src/parser/mod.rs`에 새 unit test로 둬
base `4,221` 대비 source test가 `4,223`, 해당 module이 `57` 대비 `59`로 증가했다. CONTRIBUTING의
source-side 증가 금지 계약에 따라 두 회귀는 `tests/cases/issue_6534_download_format_boundary.rs` 원본으로
이동했다. 제품 구현 완화나 기준선 상향은 사용하지 않았으며, 수정 뒤 base-aware source unit 수는
`4,221/4,221`로 복귀하고 focused integration 2/2와 full nextest를 통과했다.

## 렌더·시각 영향 판정

renderer/layout/typeset/paint와 Studio Canvas 출력 경로를 바꾸지 않는다. 변경된 Studio 코드는 원격
payload의 선행 시그니처 분류뿐이고 최종 parser가 동일 문서 IR을 생성하므로 신규 PDF·visual sweep은
필요하지 않다. packaged browser의 탭 생성 여부는 실제 Chrome E2E로 직접 확인했다.

## 로컬 검증

- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`: passed
- native root, wasm32 library, workspace all-target Clippy `-D warnings`: passed
- workspace build: passed
- integration manifest: 1,097 sources / 4,760 attrs / 48 targets
- source unit tier: 4,221 tests / 299 modules, base-aware policy check passed
- moved #6534 focused integration: 2/2 passed
- full release-test nextest: 8,908/8,908 passed, 46 policy skip, 308.125초
- Docker WASM release build와 `wasm-opt`: passed, 6분 09초
- Studio: 1,330 passed / 1 policy skip / 0 failed
- 공유·Chrome·Firefox service-worker: 131/131 passed
- Studio document signature: 17/17 passed
- Studio·Chrome·Firefox production build, frontend dist, packaged smoke: passed
- packaged Chrome download E2E: 4/4 passed
- generated suite·manifest와 build 산출물을 stage하지 않았다.

## 성능과 잔여 위험

- 동일 WSL2 host에서 ZIP HWPX 525개를 번갈아 9회 측정한 중앙값은 기준 934 ms, 후보 977 ms로
  +43 ms(+4.60%), 문서당 약 +0.082 ms였다. process 생성까지 포함한 참고값이며 hard gate가 아니다.
- 실제 문제 기관의 원응답·인증 세션은 확보하지 않아 기관별 metadata 분포를 주장하지 않는다. 보고된
  충돌 shape를 synthetic response로 구성하고 실제 packaged Chrome lifecycle을 검증한 범위다.
- Safari는 downloads API가 없어 별도 fetch 선행 게이트를 쓰며 공유 signature helper가 ZIP 후보를 아직
  `hwpx`로 부르는 차이가 남는다. 공통 core가 일반 ZIP을 최종 거부하지만 선행 명칭 정합화는 별도 범위다.
- 알려진 비-HWP 확장자가 없는 extensionless 파일이 잘못된 HWP MIME/URL을 terminal까지 유지하면 #198
  fallback 대상이 될 수 있다. 실제 HWP 감지율 보존을 위한 명시적 trade-off다.

## 코드 후보 GitHub Actions

선행 code candidate `8a7af6a18`의 CI run
[`33448288095`](https://github.com/edwardkim/rhwp/actions/runs/33448288095)에서 `Lint (fmt, clippy,
WASM check)`가 실패했다. 실패 step은 `Validate Rust test suite manifest` 안의 base-aware unit tier
검사이며 다음 수치를 보고했다.

- source-side 총량: `4,223 > 4,221`
- `src/parser/mod.rs::tests#1`: `59 > 57`

같은 조건을 `node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel`로 로컬 재현했다.
format과 integration manifest 검사는 그 앞에서 성공했고, Clippy 단계는 정책 실패 뒤 skip됐다. 따라서
과거 로컬 Clippy 성공을 현재 GitHub candidate green으로 대체하지 않는다.

수정 code candidate `108d61281`은 동일 PR branch에 push했고 다음 exact-head 검증을 통과했다.

- CI run [`33449946805`](https://github.com/edwardkim/rhwp/actions/runs/33449946805): `Build & Test`,
  `Lint (fmt, clippy, WASM check)`, Frontend package gates와 A/B/C/D archive build·test 모두 성공
- CodeQL run [`33449946802`](https://github.com/edwardkim/rhwp/actions/runs/33449946802): Rust,
  JavaScript/TypeScript, Python 분석 성공
- Render Diff run [`33449946311`](https://github.com/edwardkim/rhwp/actions/runs/33449946311): Canvas visual
  diff 성공
- Adapter inter-diff run
  [`33449946831`](https://github.com/edwardkim/rhwp/actions/runs/33449946831): 성공
- Proptest roundtrip run
  [`33449946839`](https://github.com/edwardkim/rhwp/actions/runs/33449946839): 성공
- preflight가 source·test 변경 범위에 불필요하다고 판정한 WASM Build, Native Skia, Frontend unit gates는
  예상대로 skip됐다. review와 무관한 duration refresh와 중복 CodeQL aggregate의 skip·neutral도 실패가
  아니다.

## 최종 판정

- 판정: 승인
- 근거: test 배치 blocker를 기준선 완화 없이 integration 원본으로 교정했고, 로컬 base-aware tier,
  필수 Rust lint, focused/full 회귀와 exact code head의 GitHub Full CI·required check가 모두 통과했다.
- 병합 조건: 이 review-only 기록을 trailing commit으로 push한 뒤 trusted fast-pass·required aggregate와
  최신 `MERGEABLE/CLEAN`을 확인하고 별도 merge 승인을 받는다.
- 원격 조치: self PR이므로 GitHub review event는 만들지 않는다. issue edit/close와 merge는 아직
  수행하지 않는다.
