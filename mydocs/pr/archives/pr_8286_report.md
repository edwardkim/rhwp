---
kind: report
status: active
last_verified: 2026-11-07
---

# PR #8286 병합 전 판단 — #8226 반영 뒤 통합 보정

## 최종 판정

**메인터너 보정 후 수용 가능 — 최신 devel 통합 후보의 로컬 필수 검증 완료.**

검증 code head는 `ab1491d40b43c7d54941337b852f7e7f0a5eff5e`, base는
`b8dd67c5001dfb0410e9e192528676396cdd6a4e`다. [검토 원장](pr_8286_review.md)을 정본으로 사용한다.

- 전체 nextest 12,789 PASS / 0 FAIL / 45 skipped.
- Native Skia lib 3,857 및 보조 lib 182, 그림 누락 2, 직접 PDF 22 PASS.
- fmt·Native/WASM/workspace Clippy·workspace build·doc·manifest·unit-tier 정책 PASS.
- 관련 회귀 32종, 양쪽 backend 176쪽 모두 90% 이상. 같은 조건의 최신 devel과 PNG까지 모두 바이트가 같다.
- 승인된 최종 저장본은 새 저장 API 실행에서도 바이트가 같으며, 해당 한컴 PDF에 Native 95.51%, WASM 99.56%, 각 1쪽으로 통과했다.
- Linux Native 편람 383쪽 PASS, 최저 99.96%, 공식 기준 대비 회귀 0쪽.
- 실행 입력·PDF 90개를 검증 code commit의 Git blob·SHA와 대조해 모두 일치함을 확인했다. 편람 입력·PDF 2개도 별도로 일치 확인했다.

#8226은 별도 #8711로 이미 병합됐고 원 PR도 대체 완료로 닫혔다. 따라서 최신 devel에 포함된
그 기능을 보존하며 #8286의 panic·컨테이너 배치 보정을 후속 통합 PR로 제출한다. 원 기여자의
author와 source SHA를 보존했고 메인터너 보정은 별도 커밋이다. 닫힌 #8226/#8711에는 중복 조치를 하지 않는다.

기존 DocumentCore의 차트 OLE 저장 한계와 일부 글리프·래스터 차이는 검토서에 명시했다.
이 작업은 그 기능을 해결했다고 주장하거나 #4680·#4687 전체를 닫지 않는다.

로컬 필수 검증과 검토 기록 확정을 완료했다. 사용자 문서 확인·push 승인 뒤 통합 PR을 등록한다. 최신 PR head CI,
실제 병합 및 #8286 후속 comment·close는 그 다음 승인 단계이며 완료된 사실로 기록하지 않는다.
