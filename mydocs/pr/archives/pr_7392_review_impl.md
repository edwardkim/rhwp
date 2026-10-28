---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7392 회귀 보강·통합 순서

1. 원 head `f3772351f24ef87426f8000224419c260702719d`를 base `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4` 위에 `-x`로 적용한 commit은 `106869f56acc611283b7b2c5131c0da60a74d1ef`이다. #7388 뒤 순서이며 체리픽 충돌은 없었다.
2. `7e3f0363c8987b5663ba411c5a0c7d0affc0549b`는 `tests/cases/issue_7063_empty_float_host_snap_intrudes_prev_line.rs`에 표 하단→빈 줄→뒤 본문 및 다음 쪽 첫 표 검사를 추가했다. 제품 배치 코드는 바꾸지 않았다. 검사 전에 저장 vpos, 흐름 원점, 실제 표 윗변·끝점을 분리해 기록했다.
3. code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 관련 집중 7건, 전체 release-test 10,229건, 필수 lint, Native/fresh WASM API 49·60쪽 gate를 통과했다. 추적 원본과 PDF의 해시는 [개별 review](pr_7392_review.md)에 있다. 실행 로그와 전체 임시 캡처는 무시되는 `output/pr-review/planet6897-20260924/`에만 둔다.
4. 같은 통합 PR에 archive review와 대표 시각 asset을 포함한다. 최신 head CI·mergeability와 원격 승인을 확인한 뒤 처리하며, 오류가 생기면 원 PR 코드·추가 검사·다른 PR 변경을 분리해 재검증한다. merge 뒤에만 원 PR comment와 소유 output 정리를 진행한다.
