---
kind: pr-review
pr: 6703
reviewed_at: 2026-09-04
source_head: 219868e86f94b47f0b033bf2b50d64ca655ef8d0
maintainer_correction: 547277a1a
---

# PR #6703 검토 - HWP5 찬 쪽의 near-top reset

## 판정: 메인터너 보정 후 수용 가능

**보정 상태: 메인터너 보정 완료.** 다만 이 문서는 원 PR을 그대로 승인하는 기록이
아니다. 원 PR head `219868e86f94b47f0b033bf2b50d64ca655ef8d0`의 비공개 경로 탐색과
silent skip을 제거한 뒤에만 수용 가능한 상태가 됐다.

| 구분 | commit |
| --- | --- |
| 원 조판 변경 | `ceadaf94a` |
| 원 PR rustfmt 후속 | `2dd41febf` |
| 메인터너 fixture/계약 보정 | `547277a1a` |

## 원본의 병합 보류 사유

원 회귀 테스트는 환경 변수와 Windows 개인 경로를 탐색하고 파일이 없으면 성공처럼
종료했다. 이는 공개 CI에서 회귀를 검출할 수 없고, 비공개 자료를 테스트 계약에
포함시키므로 그대로는 수용할 수 없었다.

## 메인터너 보정 내용

- 원본을 `samples/issue5941/1480000-201900698-native-neartop-reset.hwp`로 정식
  sample로 등록했다.
- `MANIFEST.json`, `README.md`, `.gitattributes`를 추가해 출처·무결성·binary
  취급을 명시했다.
- 테스트가 이 공개 sample을 반드시 읽도록 바꾸고, 환경 변수·개인 경로 탐색과
  silent skip을 제거했다.
- 전체 통합 실행 중 확인된 임시 HWP 파일명 충돌은 원자적 serial을 추가해
  `apply_para_format_in_hf_contract`와 `delete_equation_contract`에서 분리했다.
- sample의 현행 text-overlap 수 11 및 off-canvas 수 2를 baseline으로 등록했다.
  보정 전 통합 기준은 각각 12와 2였으므로 보정이 기존 이상을 확대하지 않았음을
  확인했다.

## 실행한 검증

다음은 로컬 호환/통합 검증의 실제 성공 기록이다. nextest 공식 full lane을 실행한
것으로 표기하지 않는다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  node scripts/run-rust-test.mjs issue_5941_neartop_reset_only_on_empty_page
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --test regression_suite_006 \
  ir_field_sweep_baseline::ir_field_sweep_does_not_regress
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --test regression_suite_010 \
  off_canvas_baseline::off_canvas_does_not_grow_partition_0
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 본문 중심 검토와 기준 PDF 참고값

이 PR의 수용 판단은 작업지시대로 PR 본문의 원인 분석, 제한된 적용 조건, 공개 fixture와
회귀 계약을 중심으로 한다. 시각 비교나 한컴 PDF와 rhwp의 페이지 수 차이는 최종 판정 근거로
사용하지 않는다.

Hancom 2018 저장 원본의 수동 한컴 PDF는
`pdf/issue5941-1480000-201900698-2020.pdf`에 보관했다.

| 항목 | 값 |
| --- | --- |
| 한컴 PDF 물리 페이지 | 205 |
| rhwp 회귀 계약 | 202 |
| PDF SHA-256 | `fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc` |

본문이 설명한 HWP5 저장 조판 계약의 rhwp 기대값은 202이고, 한컴 PDF의 205페이지는
참고 출력값일 뿐 이 PR의 blocker가 아니다.

## 병합 전 남은 조건

보정 commit을 포함한 최종 통합 PR head에서 required CI, mergeability 및
`mergeStateStatus=CLEAN`을 다시 확인한다. 이 기록은 원 PR의 GitHub approve, 직접 merge,
수용 완료 comment를 수행하지 않는다.
