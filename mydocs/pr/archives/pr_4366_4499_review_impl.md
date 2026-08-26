---
kind: pr_review_impl
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4366, #4499 누적 검토와 메인터너 보정 기록

## 기준과 적용 순서

| 순서 | 적용 내용 | 누적 검토 브랜치 commit |
| --- | --- | --- |
| 0 | `upstream/devel` `8dbe982e89e780fe0612a1bc66aa417bbd6356b2` | 기준점 |
| 1 | #4366 source `71f6071`, `604eaf2`, `5c95b46`, `288cd1a`, `51b5fef`, `463f449`, `eeca016`, `311cbc2` | `e00d1a2ba` |
| 2 | #4499 source `df05c8c`, `55d58a5`, `65ec22a`, `9bea1ed`, `d836359`, `df05405`, `20bf33e`, `966892b`, `83977ca`, `f6b50b5`, `e34e6d8` | `1b0b5005b` |
| 3 | HWPX 실제 검정 음영 보존 메인터너 보정 | `7e37e5b08` |

두 원 PR head는 기준 `devel`의 조상이 아니었다. merge commit을 누적하지 않고 각 source의
기능·테스트·문서 commit을 시간순으로 적용했다. 자동 3-way 병합은 #4366의
`src/parser/hml/reader.rs`, `src/renderer/svg.rs`와 #4499의
`src/document_core/converters/hwpx_to_hwp.rs`, `src/wasm_api.rs`에서 발생했으며 수동 충돌은 없었다.

## 메인터너 보정 단계

1. #4366의 `hwp3_char_shade_color(0, 100) == Some(0)` 경계와 HWPX 라이터를 대조했다.
2. 라이터가 실제 검정 `0`을 `none`으로 바꿔 HWPX 왕복에서 정보를 잃는 것을 확인했다.
3. `0xFFFFFFFF`만 sentinel으로 취급하도록 `write_char_pr`를 보정하고, `#000000` 보존 단위
   테스트를 추가했다.
4. HWP3/HWPX 저장 계약, formatter, clippy, 누적 전체 nextest, 한컴 2020 PDF를 차례로
   재검증했다.

## 검증 순서와 결과

| 단계 | 명령 또는 판정 | 결과 |
| --- | --- | --- |
| 저장 계약 | `issue_4155_hwp3_char_shade_contract` | 7 passed |
| HWPX 보정 | `write_char_pr_preserves_opaque_black_shade` | 1 passed |
| 형식 | `cargo fmt --check` | 통과 |
| 정적 분석 | `cargo clippy --profile release-test --all-targets -- -D warnings` | 통과 |
| 전체 | `cargo nextest run --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast` | 5,730 passed, 7 slow, 36 skipped, 437.285s |
| #4499 한컴 2020 | 차트 원본 HWPX와 rhwp HWP 산출 PDF 144 DPI 비교 | 1쪽 래스터 SHA-256 동일 |
| #4366 한컴 2020 | `SO-SUEOP.hwp` 원본과 rhwp HWP 산출 PDF 3쪽 육안 검토 | 검정 막대 소멸, 본문 판독 가능 |

## 다음 단계와 경계

- 현재 branch는 로컬 누적 검증과 보정용이다. 원 PR의 source를 force-push하거나 contributor
  commit을 rewrite하지 않는다.
- 원 PR에 보정을 반영할지, 별도 통합 PR로 낼지는 작업지시자 승인 후에만 정한다.
- remote push 뒤에는 해당 최신 head의 required checks와 mergeability를 다시 확인한다.
- PDF 페이지 수와 자동 번호, 머리말, 들여쓰기 fidelity의 기존 차이는 #4366 결함과 분리하며
  [#3820](https://github.com/edwardkim/rhwp/issues/3820)에서 추적한다.
- HWP5의 과거 기본값 `0`과 의도된 순검정 음영을 renderer IR에서 구분하지 못하는 한계도
  저장 보정과 분리한다. HWPX/HWP 저장은 `0`을 보존하지만 renderer는 현재 보수적으로
  음영 없음으로 처리하며, 실제 표본과 함께 #3820에서 모델 표현 확장 여부를 판단한다.

## rollback

메인터너 보정만 철회해야 하면 `7e37e5b08`만 revert한다. 원 PR의 누적 체리픽은 검토용 branch에만
있으므로 원 contributor branch와 `upstream/devel`에는 영향을 주지 않는다.
