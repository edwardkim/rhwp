---
kind: report
status: active
last_verified: 2026-08-12
---

# PR #4647 검토 — 문서 열기 압축 해제 상한

## 라우팅

```text
base route: CONTRIBUTING.md의 contributor Fork & PR 경로
modifiers: Draft 재작업, 1,000줄 초과 대형 변경
loaded documents: AGENTS.md, CONTRIBUTING.md, pr_review_workflow.md,
  pr_review/README.md, pr_review/local_validation.md,
  pr_review/rework_and_exceptions.md
previous reviewed code head: 8ea821936cababd70f92ef5e297ada97157f089f
current code candidate: 8e1ddbb83be4ef8f0d4d6e7e7b9f13818e53b157
```

## Metadata

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR | [#4647](https://github.com/edwardkim/rhwp/pull/4647) |
| 작성자 | `humdrum00001010` |
| reviewer | `edwardkim` 요청은 작성자 권한 부족으로 실패해 repository 측 지정 대기 |
| base / head | `devel` / `humdrum00001010:fix/bound-open-decompression-m34` |
| 관련 보고 | 비공개 보고 |
| 규모 | 수정 중 (source·E2E 회귀·경계 문서) |
| 상태 | Open, Draft; code candidate의 Gestell 및 로컬 게이트 통과, 최신 GitHub required checks 대기 |

## 변경 범위

완전 문서 열기 진입점인 `parse_hwp*`, `parse_document*`, `hwp3::parse_hwp3*`가 HWP5
DocInfo·본문 섹션 및 HWP3 압축 본문에 적용할 byte 예산을 선택한다. HWP5 strict·lenient,
일반·비밀번호·배포용 분기는 그 선택된 같은 누적 문서 예산을 소비한다. 상한 초과는 빈 레코드나
빈 섹션으로 대체하지 않고 명시적 오류로 반환한다.

HWP5 비압축 DocInfo·본문도 같은 길이 검사를 받으므로 이 변경은 압축 해제 출력뿐 아니라 핵심
스트림의 일반 크기 정책이기도 하다. 256/512 MiB는 HWP 규격의 유효성 상한이 아니라 rhwp의
결정적 자원 예산이다. `BinData`, preview와 기타 보조 스트림은 이 예산에 포함하지 않는다.

CFB와 crypto 계층의 일반 API는 이 정책을 선택하거나 `MAX_*` 기본값을 import하지 않는다.
호출자가 명시적으로 준 `max_bytes`를 기계적으로 적용하는 `_limited(..., max_bytes)` API만
예산을 강제한다. 이미 공개된 `crypto::MAX_PASSWORD_DECOMPRESSED_STREAM_BYTES`는 source
compatibility를 위해 deprecated 값으로만 남기며, 어느 crypto helper도 이를 자동으로 선택하지
않는다. 암호 BinData materialize는 문서 열기 누적 예산과 별도의 parser 소비 경계이며, 해당
호출부가 별도 상한을 명시적으로 전달한다.

문서 열기 정책과 포맷 해석의 소유는 `src/parser/`에 둔다. 독립적으로 공격자 제어 CFB 스트림을
여는 raw-record consumer도 자신의 명시적 상한을 선택한다. 이 보정은 `dump-records` CLI와 공통
`diagnostics` 모듈 및 그 모듈을 쓰는 등록 진단 명령 9개를 그 경계로 배선한다. renderer, layout,
WASM API와 rhwp-studio 출력은 바꾸지 않으므로 시각·fixture 증적 경로는 적용하지 않는다.

## Gestell correction finding

초기 검토의 “blocking code finding은 없다”는 결론은 철회한다. Gestell 검토는 다음을 blocking
abstraction defect로 확인했다.

- CFB와 crypto의 public 일반 API가 문서 열기 전용 `MAX_*` 정책을 직접 선택해, 다른 consumer도
  보이지 않는 제품 정책을 강제로 적용받았다.
- 초기 회귀는 low-level `_limited` 메커니즘을 직접 호출했으므로 공개 문서 열기 경로가 정책을 선택하고
  누적 예산을 전달한다는 계약을 증명하지 못했다.

수정 후의 책임은 다음과 같다.

- 단일 스트림은 HWPX XML 엔트리와 HWP3 기존 문서 열기 계약에 맞춘 256 MiB, HWP5 DocInfo와
  본문 전체는 512 MiB 누적 예산을 사용한다.
- `src/parser/mod.rs`의 완전 HWP5 열기 경계가 strict·lenient·일반·비밀번호·배포용 경로 모두에
  남은 바이트 상한을 전달한다. `hwp3::parse_hwp3*`도 완전 HWP3 열기 경계에서 본문 상한을 선택한다.
- CFB/crypto의 일반 API는 기존의 일반 decode 의미를 유지하고, `_limited(..., max_bytes)`만 호출자
  제공 상한을 적용한다.
- `dump-records`와 등록 raw-record diagnostics는 문서 열기 정책을 재사용하지 않고 각 consumer의
  이름 붙은 상한을 limited CFB/crypto 호출에 직접 전달한다.
- test-only scoped policy seam으로 큰 fixture 없이 공개 `parse_hwp`, `parse_document`,
  `parse_document_with_password`를 작은 예산으로 실행한다. 이는 release build의 API나 정책을 바꾸지
  않는다.

## 10k 문서 호환성 실험

2026-08-12 `~/hwpdocs_10k`의 HWP/HWPX 10,000건을 이전 검토 code head
`8ea821936cababd70f92ef5e297ada97157f089f`로 전건 열었다. 파일명과 문서 내용은 기록하지 않고
포맷·오류·크기만 집계했다. 이번 보정은 같은 256/512 MiB 수치를 유지한 채 정책 선택 위치만
완전 문서 열기 경계로 옮긴 것이므로 이 표는 수치 선택의 호환성 근거로 남긴다. 다만 이 표는
현재 correction commit을 다시 전수 실행한 결과가 아니며, 그 commit의 전수 증거로 주장하지 않는다.

| 항목 | 결과 |
| --- | ---: |
| 전체 입력 | 10,000 |
| 열기 성공 | 9,948 |
| decompression limit 초과 | 0 |
| 비밀번호 필요 | 5 |
| 지원하지 않는 입력 | 47 (`DRM` 8, empty 24, unknown 15) |
| HWP5 | 6,491 (6,490 성공, 비밀번호 필요 1) |
| HWP3 | 38 (38 성공) |
| HWPX | 3,424 (3,420 성공, 비밀번호 필요 4) |

HWP5 성공 문서의 decompressed/raw 핵심 스트림 분포는 다음과 같다. 파싱된 HWP5 6,490건에서
누락된 `raw_stream`은 0개였다.

| 측정 | bytes | MiB |
| --- | ---: | ---: |
| 최대 `DocInfo` | 190,940 | 0.18 |
| 최대 단일 `BodyText` 섹션 | 21,606,061 | 20.61 |
| 단일 스트림 p99 | 1,175,313 | 1.12 |
| `DocInfo` + 모든 본문 누적 최대 | 21,678,881 | 20.67 |
| 누적 p99 | 1,621,500 | 1.55 |

관측 최대는 단일 256 MiB 상한보다 12.4배, 누적 512 MiB 상한보다 24.8배 작았다. 이 결과는
현재 10k 코퍼스에서 정상 열기를 막지 않는다는 근거다. 다만 비공개 코퍼스의 경험적 결과이므로
모든 정상 문서가 상한 안이라는 규격 근거나 전역 증명으로 일반화하지 않는다.

## 검증

- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/phihu/Desktop/rhwp_core/target/pr4647-e2e cargo test
  --profile release-test --lib open_decompression`: 10/10 통과. low-level mechanism 3건과 공개
  문서 열기 E2E 7건(HWP5 strict DocInfo, strict distribution의 ViewText-only 계약, 실제
  strict→lenient fallback DocInfo 및 distribution, HWP5 누적 예산, HWP5 비밀번호 DocInfo,
  HWP3 자동 감지 본문)을 실행했다.
- `cargo fmt --all --check`: 통과
- `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/phihu/Desktop/rhwp_core/target/pr4647-e2e cargo check
  --profile release-test --bin rhwp`: 통과. `dump-records` consumer 배선을 포함해 바이너리 크레이트를
  컴파일했다.
- CI 실패 원인은 `src/parser/mod.rs`의 test-only 정책 조회에 있던 `return`을
  `clippy::needless_return`이 거부한 것이었다. `8e1ddbb83be4ef8f0d4d6e7e7b9f13818e53b157`에서
  같은 표현을 cfg 블록의 tail expression으로 바꿨으며, test·release cfg의 선택 결과는 변하지 않는다.
- exact code candidate `8e1ddbb83be4ef8f0d4d6e7e7b9f13818e53b157`에 대한 독립 Gestell 재검토:
  PASS. 완전 문서 열기 정책 소유, CFB/crypto의 explicit-limit mechanism 경계, strict·lenient·password·
  distribution·HWP3 경로 및 test-only seam에서 blocking abstraction defect가 없음을 확인했다.
- `cargo fmt --all -- --check`: 통과.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/phihu/Desktop/rhwp_core/target/pr4647-contributing cargo test
  --profile release-test --tests`: 통과(exit 0).
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/phihu/Desktop/rhwp_core/target/pr4647-contributing cargo clippy
  -- -D warnings`: 통과.
- GitHub 실패 job과 같은 범위의 `cargo build --workspace` 및
  `cargo clippy --workspace --all-targets -- -D warnings`: 통과.
- `git diff --check`: 통과.
- 이전 code head의 10k open/stream-size audit: 10,000건 완료, limit 초과 0, panic 0.

## 현재 권고

로컬 correction은 게시 가능한 상태다. PR은 Draft 상태를 유지하며, correction을 push한 최신 PR head의
required checks와 repository 권한 보유자의 검토 및 작업지시자 승인이 확인된 뒤 merge 후보로 판단한다.
