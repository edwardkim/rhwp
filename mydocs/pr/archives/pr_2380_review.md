# PR #2380 검토 — fork upstream 원격 등록 절차 문서화 (kevin9327 첫 PR)

- PR: https://github.com/edwardkim/rhwp/pull/2380 — docs-only (+11/−0)
- 컨트리뷰터: kevin9327 **rhwp 첫 PR** (동시 #2381 대기 — 순서 처리)

## 변경 본질

실제 온보딩에서 막힌 지점의 정확한 보고 — 가이드들이 `upstream` 원격을
전제하는데 등록 방법이 어디에도 없음(`grep` 부재 증명까지 동봉). 정정:
dev_environment_guide 에 등록 절차 절 추가 + onboarding_guide 는 문서
자체의 "중복 금지" 원칙을 읽고 주석 한 줄만 추가 — **저장소 문서 규범을
먼저 읽고 맞춘 첫 기여**.

체크리스트 미체크 사유를 정직하게 명시(Windows Smart App Control 의 cargo
차단)하고 실행 가능한 검사(fmt·링크)만 통과 표기 — 검증 정직성.

## 로컬 재실증 (merged tree)

충돌 0 · 링크 384+ / 메타데이터 검사 green · 코드 diff 0 · 첫 기여자
워크플로 승인 후 CI 전 항목 pass.

## 판단

**merge 권고.** 온보딩 실경험 기반의 정확한 문서 결손 보수.
