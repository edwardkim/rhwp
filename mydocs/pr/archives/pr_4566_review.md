---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4566 리뷰 - 최상위 표 y 겹침 진단

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4566](https://github.com/edwardkim/rhwp/pull/4566) · @planet6897 |
| base / 원 head | `devel` / `761ddfcfed3cdbee873326530bf60046254b803a` |
| 규모 | 진단 API, 단위·통합 회귀, 개인정보 치환 HWP fixture, stage 기록 |
| 작성 시점 상태 | OPEN, `MERGEABLE`; 원 head의 GitHub required checks는 모두 통과 |

## 범위

기존 `LAYOUT_OVERFLOW`는 본문 하단을 초과한 항목만 기록하므로, 하단 clamp 뒤 서로 겹친 최상위
표를 감지하지 못했다. 이 PR은 페이지 조립이 끝난 render tree에서 Page 직계 overlay 표와
Body→Column 직계 흐름 표의 y 구간을 모아 2px 초과 겹침을 `LAYOUT_TABLE_OVERLAP`으로 기록한다.
셀 안 중첩표와 비가시 표는 대상에서 제외한다.

진단은 관측 전용이다. 표의 좌표·페이지네이션·render tree를 바꾸지 않으며, 실제 배치 보정은
별도 원인별 수정에서 다뤄야 한다.

## 누적 통합과 검증

- 최신 #4520 `d6c5ac1`의 HWP5 중첩표 높이 캡과 호스트-마지막 문단 예약 간격 판별을 포함한 로컬
  통합 head에 충돌 없이
  적용했다.
- `cargo fmt --check`를 통과했다.
- `cargo test --profile release-test --target-dir target/pr-review --test
  issue_4515_table_overlap_diag -- --nocapture`를 실행해 통과했다.
  47쪽 `sample1-repro.hwp`의 6개 실제 결함 쪽에서 8개 overlap을 기록했고, 진단 결과는
  render tree JSON에서 독립 재계산한 표 쌍과 페이지·문단 쌍 단위로 일치했다.
- 원 PR head의 GitHub CI(Full Rust, Canvas visual diff, Native Skia, CodeQL 포함)는 모두 통과했다.
- 최신 누적 head에서도 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review
  --tests --test-threads 12 --no-fail-fast`가 **5,707 passed, 정책 skip 35**로 완료됐다.

## 판정

최상위 표 y 겹침을 기존 하단 overflow와 분리해 관측하는 범위가 명확하고, fixture 기반 통합 회귀가
실제 render tree와 자기일관적으로 일치한다. 이 PR은 **진단 추가**이며, 별도 HWP 2020 PDF raster
fidelity를 개선하거나 보증하는 변경이 아니다.

**최종 권고: 최신 통합 head의 로컬 CI는 통과했다. 작업지시자 승인 후 수용한다.**
