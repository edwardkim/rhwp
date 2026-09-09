---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_6534_impl.md
issue: 6534
stage: 1
date: 2026-08-31
---

# Task M100 #6534 Stage 1 — red 계약 결과

## 1. 목적

제품 코드를 고치기 전에 다음 두 원인이 현행 구현에서 실제 실패하는 계약으로 고정되는지 확인했다.

1. 확정 XLSX filename/MIME보다 URL·finalUrl·HWP MIME의 양성 신호가 우선하는 다운로드 오탐
2. 일반 OOXML ZIP과 불완전 ZIP을 HWPX로 확정하는 Studio/Rust 포맷 오분류

이번 단계의 변경은 test source뿐이다. 다운로드 interceptor, 공유 판정 구현, Studio 제품 코드,
Rust parser 제품 로직은 수정하지 않았다.

## 2. 다운로드 계층 red 결과

| 표면 | 전체 | 기존·보호 green | 신규 red | 확인된 원인 |
| --- | ---: | ---: | ---: | --- |
| 공유 판정 | 32 | 26 | 6 | boolean OR 판정, 3상태 API 부재 |
| Chrome adapter | 20 | 17 | 3 | XLSX도 즉시 탭 생성, URL/MIME-only 후보를 `onCreated`에서 처리 |
| Firefox adapter | 14 | 11 | 3 | Chrome과 같은 공유 판정·즉시 처리 |

새 계약은 다음을 고정한다.

- `.xlsx` filename + `.hwp` URL: 자동 열기 거부
- `.xlsx` filename + HWP MIME: 자동 열기 거부
- generic filename + XLSX MIME + HWP redirect: 자동 열기 거부
- URL/MIME-only HWP 근거: `onCreated`에서는 보류
- extensionless HWP MIME: terminal 재조회 뒤 수용
- 확정 `.hwp` filename과 DEXT5 차단 우선순위 유지

실패 출력에서 Chrome은 XLSX filename을 포함한
`chrome-extension://rhwp/viewer.html?...filename=...public-report.xlsx` 탭을 실제로 1개 만들었고,
Firefox도 동일하게 `moz-extension://` 탭을 만들었다. 이는 #6534의 사용자 관찰과 같은 호출 경로다.

기존 보호계약 중 DEXT5, 과거 다운로드 차단, event-page/service-worker restart, download id당 1회,
동시 terminal 처리, settings sync fail-closed, Chrome `file://` HWP 억제는 계속 통과했다.

## 3. ZIP/HWPX 계층 red 결과

| 표면 | 전체/선택 | green | red | 확인된 원인 |
| --- | ---: | ---: | ---: | --- |
| Studio signature | 17 | 15 | 2 | HWPX·OOXML ZIP 모두 `hwpx`; 기대 `zip` 후보 |
| Rust `issue6534_detect_format*` | 2 | 0 | 2 | XLSX와 필수 엔트리 하나뿐인 ZIP 모두 `FileFormat::Hwpx` |
| Rust 정상 합성 HWPX | 1 | 1 | 0 | 두 필수 엔트리가 있는 ZIP의 기존 수용 확인 |

Rust 회귀는 payload를 해제하지 않는 합성 ZIP을 test 안에서 만들었다.

- 정상 HWPX 후보: `Contents/content.hpf` + `Contents/header.xml` → 현행도 `Hwpx`
- XLSX 후보: `[Content_Types].xml` + `xl/workbook.xml` → 현행은 잘못 `Hwpx`
- 불완전 후보: HWPX 필수 엔트리 하나만 존재 → 현행은 잘못 `Hwpx`

XLSX의 기대 경로는 `FileFormat::Unknown`과 `UNSUPPORTED_FILE_FORMAT`이다. 현행은 첫
`assert_eq!`에서 `Hwpx != Unknown`으로 실패해 HWPX parser 진입 전 거부 계약이 없음을 확인했다.

## 4. 비-red 보호 검사

| 검사 | 결과 |
| --- | --- |
| `node --test rhwp-shared/sw/download-observer-state.test.js` | 14/14 PASS |
| `cargo fmt --all -- --check` | PASS |
| `node scripts/rust-unit-test-tiers.mjs --check` | PASS — 4,223 tests / 299 modules |
| `git diff --check` | PASS |

첫 Rust 실행에서 `cargo test`에 test filter 두 개를 위치 인자로 동시에 넘겨 Cargo 사용법 오류가 한 번
발생했다. 제품·테스트 실패 증적으로 사용하지 않았고, 올바른 단일 filter 명령으로 다시 실행해 위 결과를
확정했다.

## 5. 변경 파일

- `rhwp-shared/sw/download-interceptor-common.test.js`
- `rhwp-chrome/sw/download-interceptor.test.mjs`
- `rhwp-firefox/sw/download-interceptor.test.mjs`
- `rhwp-studio/tests/document-signature.test.ts`
- `src/parser/mod.rs`의 `#[cfg(test)]` 블록

새 integration source, binary fixture, generated suite·manifest, 제품 구현 변경은 없다.

## 6. Stage 1 판정과 다음 게이트

Stage 1은 **qualified-red-contract**다.

- #6534 오탐이 공유 순수 판정과 두 브라우저 adapter에서 결정론적으로 재현된다.
- ZIP 매직만으로 HWPX를 확정하는 두 번째 원인도 Studio와 Rust에서 분리 재현된다.
- 기존 보호 불변식은 새 red 이외에 깨지지 않았다.

다음 단계는 Stage 2 공유 `intercept/defer/ignore` 분류와 Chrome/Firefox adapter 연결이다. Stage 1
checkpoint를 고정한 뒤 메인테이너의 다음 단계 승인을 받기 전에는 제품 코드를 수정하지 않는다.
