---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_6534_impl.md
issue: 6534
stage: 3
date: 2026-08-31
---

# Task M100 #6534 Stage 3 — ZIP 후보·HWPX 구조 확정 결과

## 1. 구현 결과

ZIP 매직을 HWPX로 단정하던 두 경계를 후보와 최종 확정으로 분리했다.

- Studio `detectDocumentByteKind()`는 ZIP 매직을 `hwpx`가 아니라 `zip` 후보로 반환한다.
- transport gate는 ZIP 후보를 Rust로 전달하되 HWPX라는 의미를 부여하지 않는다.
- Rust `detect_format()`은 `PK\x03\x04` 뒤 중앙 디렉터리에서
  `Contents/content.hpf`와 `Contents/header.xml`을 exact lookup해 두 엔트리가 모두 있을 때만
  `FileFormat::Hwpx`를 반환한다.
- XLSX·일반 ZIP·필수 엔트리가 하나뿐인 ZIP·손상 ZIP은 `FileFormat::Unknown`이다.

새 `FileFormat` variant와 브라우저 ZIP parser는 추가하지 않았다. 새 helper는 엔트리 payload를
압축 해제하지 않고 중앙 디렉터리의 이름만 읽는다. XML·암호·압축 해제 상한은 기존 HWPX parser가
계속 소유한다.

## 2. focused·보호계약 검증

| 검사 | 결과 |
| --- | ---: |
| Studio document signature | 17/17 PASS |
| Rust `issue6534_detect_format*` | 2/2 PASS |
| 기존 정상 합성 HWPX 감지 | 1/1 PASS |
| HWPX reader unit test | 9/9 PASS |
| 실제 비밀번호 HWPX와 plain counterpart integration | 1/1 PASS |
| Rust unit tier 정책 | PASS — 4,223 tests / 299 modules |
| `cargo fmt --all -- --check` | PASS |

Stage 1의 Studio 2건과 Rust 2건 red가 모두 green으로 전환됐다. 실제 ODF 암호 HWPX 표본도 필수
엔트리 구조 게이트를 통과한 뒤 기존 비밀번호 계약을 유지했다.

integration 검증은 먼저 독립 target 이름을 추측해 실행해 등록되지 않은 target 오류가 한 번
발생했다. 제품 실패로 취급하지 않았고, 정본 `--prepare` 뒤 생성된 `regression_suite_025`에서 해당
case를 선택해 통과시켰다. 파생 suite·manifest는 stage 대상에 포함하지 않는다.

## 3. 추적 HWPX 감사

`git ls-files -z '*.hwpx'`의 534개를 새 detector로 전수 확인했다. 파일명이나 문서 내용은 보고서에
수록하지 않았다.

| 입력 분류 | 수 | 새 detector 결과 |
| --- | ---: | --- |
| 실제 ZIP HWPX | 525 | 525 `Hwpx`, 실패 0 |
| CFB 매직의 `.hwpx` 경로 | 6 | 6 `Hwp` 유지 |
| placeholder·기타 | 3 | 3 `Unknown` 유지 |

전체 534개에서 기대 분류와 실제 분류의 불일치는 0개였다. 기존 정상 문서를 일반 ZIP으로 거부하거나
확장자만 보고 HWPX로 수용하는 회귀는 관측되지 않았다.

## 4. release warm 성능 기초값

기준판은 별도 detached worktree의 정확한 `upstream/devel@9edf3d82ba47c539f4a5e0aa7fe3716671619bba`,
후보판은 같은 의존성·release profile에서 현재 Stage 3 source를 독립 재빌드했다. 두 바이너리의 SHA-256이
서로 다름을 확인하고, 실제 ZIP HWPX 525개에 `rhwp-agent magic`을 한 번씩 실행하는 전체 통과를 각각
예열한 뒤 순서를 번갈아 9회 측정했다.

| 지표 | 기준판 | 후보판 | 차이 |
| --- | ---: | ---: | ---: |
| median, 525건 전체 | 934 ms | 977 ms | +43 ms (+4.60%) |
| p95, 525건 전체 | 943 ms | 990 ms | +47 ms (+4.98%) |
| median 환산, 문서당 | 1.779 ms | 1.861 ms | 약 +0.082 ms |

표본이 9회라 nearest-rank p95는 각 집합의 최댓값이다. 또한 수치는 CLI process 생성 비용까지 포함한
현재 WSL2 호스트의 end-to-end 기초값이며, CI hard gate나 일반화된 성능 보장을 뜻하지 않는다.
증가분은 ZIP 중앙 디렉터리 parse와 두 exact lookup의 비용으로 해석할 수 있지만, 별도 profiler 없이
세부 원인 비율을 단정하지 않는다.

첫 후보 build가 공유 target의 mtime 때문에 기준판 artifact를 재사용한 사실을 해시 확인 전에 발견했다.
해당 후보는 측정하지 않고 폐기했으며, `cargo clean -p rhwp --release` 뒤 5분 35초 동안 다시 만든 서로
다른 해시의 후보만 위 측정에 사용했다. 측정 뒤 agent가 만든 detached worktree, 전용 release target
563.8 MiB와 복사 바이너리를 모두 제거했다.

## 5. 변경 파일과 비변경 불변식

### 제품 변경

- `src/parser/hwpx/reader.rs`
- `src/parser/mod.rs`
- `rhwp-studio/src/core/document-signature.ts`

### 유지한 경계

- HWP/HWP3/HML과 HTML/XML transport 판정
- 실제 HWPX XML·암호·압축 해제 처리와 크기 상한
- CLI·service의 `FileFormat` 열거형과 비지원 형식 오류 경로
- 다운로드 observer·adapter, settings와 browser manifest
- Safari 별도 signature gate
- generated integration suite·manifest 비제출

## 6. Stage 3 판정과 다음 게이트

Stage 3는 **qualified-hwpx-structure**다.

- Studio는 ZIP을 후보로만 부르고 Rust가 HWPX 최종 확정을 소유한다.
- 일반 OOXML ZIP과 불완전 ZIP은 HWPX parser 전에 `Unknown`으로 끝난다.
- 추적된 실제 ZIP HWPX 525개와 비밀번호 보호 표본의 수용은 유지된다.
- 구조 확인의 end-to-end 비용은 이 환경에서 525건 중앙값 기준 +43 ms로 계측됐다.

다음 단계는 Stage 4다. 격리 packaged Chrome E2E를 추가해 실제 다운로드 유지·자동 탭 수를 검증하고,
Studio/확장 build, dist 계약, Rust 필수 lint 묶음과 Docker WASM을 수행한다. Stage 3 checkpoint 뒤 별도
승인을 받기 전에는 Stage 4 파일을 수정하지 않는다.
