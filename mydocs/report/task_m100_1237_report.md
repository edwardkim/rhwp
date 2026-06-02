# Task M100 #1237 완료 보고서

## 이슈

- #1237 — HWP 저장 계약: 문단 헤더와 LINE_SEG 필드 의미 정리

## 완료 내용

1. `LineSeg.tag` 공식 bit 의미를 코드 상수와 helper로 정리했다.
2. `0x00060000`을 범용 기본 플래그가 아니라 `TAG_SINGLE_SEGMENT_LINE`으로 명명했다.
3. HWPX lineseg 직렬화 주석의 6필드 오류를 9필드 계약으로 수정했다.
4. HWP5 `PARA_HEADER` 저장 시 재생성되는 필드와 보존되는 suffix 범위를 문서화했다.
5. `hwp_save_guide.md`와 `document_ir_lineseg_standard.md`를 보강했다.
6. HWP 3.0에는 HWP 5.0 `PARA_LINE_SEG` 수준의 줄 위치/세그먼트/wrap zone 정보가 부족하므로, HWP3 변환 추정값과 HWP5 기반 공통 IR 계약을 분리해 설명했다.

## 검증

| 항목 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo test -p rhwp task177_lineseg --lib` | 통과 |
| `cargo check -p rhwp` | 통과 |

## 판단

이번 작업은 공식 스펙 오류 수정이 아니라 저장 계약 명문화와 코드 상수화 작업이다. 따라서 `hwp_spec_errata.md`는 갱신하지 않았다.
