---
kind: pr-review
status: ci-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4683 검토 - planet6897 Studio·오라클·HWPX 직렬화 통합

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4683](https://github.com/edwardkim/rhwp/pull/4683) |
| source branch | `pr/planet6897-20260812-r2` |
| base / code head | `devel` / `fd71b181a674a44de2d4c1607b2c49e761fadea2` |
| 기준 devel | `88012c7e09a6bcd6ec3c4065abb194dae9209e01` |
| 통합 원 PR | [#4670](https://github.com/edwardkim/rhwp/pull/4670), [#4679](https://github.com/edwardkim/rhwp/pull/4679), [#4681](https://github.com/edwardkim/rhwp/pull/4681) |
| 관련 이슈 | [#4675](https://github.com/edwardkim/rhwp/issues/4675) close 대상, [#4678](https://github.com/edwardkim/rhwp/issues/4678) 참조 |
| 상태 | GitHub Actions 대기 |

외부 contributor의 세 PR을 최신 `devel` 위에 오래된 기능 commit부터 누적 적용했다. #4670은
원 source가 `devel`과 충돌하므로 직접 병합하지 않고 이 통합 PR로 반영한다.

## 포함 내용

- Studio JavaScript bridge, plugin host, HwpCtrl plugin·adapter, browser E2E와 bridge gate
- HWP/HWPX load/save 매트릭스 및 한글 COM 오라클 전수검사 도구
- U+2007 고정폭 빈칸의 `hp:fwSpace` 복원과 U+00A0 리터럴 보존
- plugin swap 중복 알림 제거, 현재 dispatcher 계약·Node 24 gate 출력 호환
- loadsave_sweep의 공통 경로·사전 준비·한글 버전 확인 절차

세 원 PR의 개별 판단과 실제 검증 결과는 다음 archive 기록에 분리했다.

- [PR #4670 검토](pr_4670_review.md)
- [PR #4679 검토](pr_4679_review.md)
- [PR #4681 검토](pr_4681_review.md)
- [통합 구현 기록](pr_4670_4679_4681_review_impl.md)

## 완료한 로컬 검증

- 전체 Rust nextest: 5,881/5,881 통과, 고정 `target/pr-review` 재사용, `CARGO_INCREMENTAL=0` 미지정
- HWPX 집중: lib 638/638, roundtrip·CLI·form 31/31 통과
- `cargo fmt --check`, clippy(`-D warnings`), WASM web build 통과
- Studio unit 862/0, HwpCtrl 21/0, browser E2E 87 assertions, CI-unit TypeScript와 production build 통과
- load/save Python 문법·CLI help 및 Windows PowerShell 오라클 스크립트 parser 통과

## 병합 전 조건

이 문서는 code head의 로컬 검증 결과를 기록한다. GitHub Actions와 CodeQL은 이 PR의 최신
code head에서 성공해야 하며, 작업지시자의 병합 승인이 필요하다. 병합 뒤 #4670·#4679·#4681에는
통합 반영 사실과 메인터너 보정 이유를 남기고 close한다. #4675의 자동 close도 확인한다.
