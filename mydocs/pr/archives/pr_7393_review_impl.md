---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7393 메인터너 보정·통합 순서

1. 원 head `2f35381aa6f080a24cf6398967f552e26ff1a856`를 base `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4` 위 누적 브랜치에 `-x`로 적용한 commit은 `774ab3844d1340aa24e67b1c64b684c6eaa7f60f`이다.
2. 원 PR의 균등 분포 표본은 잘못된 구역 배분에서도 개수가 우연히 맞는 한계가 있었다. 메인터너 `096e42a98cee9a955f143dd68a639def9fc64a40`은 14구역·비균등 바탕쪽 실문서의 저장·재열기 검사와 한컴 2024 원본·왕복본 PDF 두 개를 추가했다. contributor 코드를 다시 쓰지 않았다.
3. 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`에서 집중 3건, 전체 10,229건 및 필수 Rust lint를 확인했다. PDF 대조는 3–8쪽의 픽셀 동일성을 확인했다. 실행 로그는 `output/pr-review/planet6897-20260924/`에만 둔다.
4. 최신 base·head와 원격 CI·mergeability를 통합 PR 생성 뒤 다시 확인한다. 실제 merge 뒤 contributor comment에서 기여 변경과 메인터너 증적 보강을 구분한다.
