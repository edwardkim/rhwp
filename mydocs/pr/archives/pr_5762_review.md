---
kind: pr-review
status: review-complete-pending-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5762 검토 - 버그헌트 r1 renderer/tools

- source head `2f6e26462b59165166c0d390ffc160f86e3efd96`, `MERGEABLE/CLEAN`, Full CI, Rust CodeQL,
  Canvas visual diff, Native Skia, Proptest가 모두 성공했다.
- #5717 SHOW_FIRST 배경/테두리, #5730 밑줄 기준선, #5725 수식 OLE CONTENTS, #5724 WMF CONTENTS,
  #5726 Windows HWP oracle의 범위를 하나의 회귀 묶음으로 고정한다.
- 통합 후보에서 전체 release-test nextest **8,025 passed, 3 slow, 39 skipped**, native-Skia library
  **3,950 passed / 13 ignored**, native 표적 6/6, clippy, web WASM build를 통과했다.
- Windows PowerShell 5.1에서 `tools/hwp_oracle_pdf.ps1` 구문 파싱을 통과했다. COM 실행은 남아 있는
  Hwp.exe를 강제 종료하는 부작용이 있어 검증 범위에서 제외했다.
- 수용 권고. 적용 commit은 `c81673ea9`이다.
