# seo-rii PR 일괄 검토·메인테이너 보정 계획

## 범위와 승인

2026-10-30 사용자가 seo-rii의 PR 처리와 발견된 문제의 메인테이너 보정을 지시했다. Draft를 제외한 열린 PR은 #8270, #8275, #8276, #8279다. 원격 push·통합 PR 생성·comment·merge는 이 로컬 보정 승인과 구분한다.

- 기본 경로: `maintainer_general.md`.
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`, `visual_fixture_evidence.md`, `rework_and_exceptions.md`.
- 읽은 진입 문서: `CLAUDE.md`, 프로젝트 Codex MEMORY, mydocs/manual 지도, `pr_review_workflow.md`, `pr_review/README.md`, GitHub 운영·문서/Git 가이드와 위 보조 절차.
- reviewer: 네 원 PR에 `edwardkim`을 지정하고 requested_reviewers API로 확인했다.
- 기준 devel: `96ccd557592a711f215e7ad28822f4c7a3f00c15`.
- 작업 브랜치: `review/integration-536-seo-rii-native`.
- 현재 본인 계정의 열린 PR: 없음. 다른 작업 브랜치와 원 contributor fork는 유지한다.

## 원본과 통합 순서

| PR | 원 head | 독립 변경 |
| --- | --- | --- |
| #8270 | `70d66d8ac206a899072362a6322ceaa6ab51a547` | `453f4dd793` 다중 페이지 LRU, `70d66d8ac2` Native cache CI |
| #8275 | `0d5940ac1bc7078dc69901d1fa985b0c3a83184a` | region planner·host surface Native CI |
| #8276 | `06bf9c11002879934831f5c2276fe2b267cc8be0` | 고정 격자 타일 렌더 API |
| #8279 | `19f332edc9258467cb991f4196c53119f357aeb8` | `f9897cd4bd` PDF 연결선, `19f332edc9` f32 화살촉 붕괴 거부 |

마지막 head가 네 PR의 모든 원 commit을 포함한다. 최신 devel에서 이 head를 merge해 원 저자·commit을 보존한다. 관련 이슈 #536은 전체 Native 추적 범위이므로 닫지 않는다.

## 최신 devel 확인과 충돌 보정

기준 devel의 Native cache는 단일 `Option<RegionEntry>`이고 타일 API가 없다. 직접 PDF 검사는 connector line style을 거부한다. 따라서 기능 변경은 아직 필요한 후보이며 실제 전후 실행으로 확인한다.

충돌은 `.github/workflows/ci.yml`, `scripts/ci-impact-classifier.cjs`, classifier test 세 파일이다. 최신 devel의 Skia glyph·#8153 이미지/글꼴 검사와 이번 세 Native 검사를 합집합으로 보존한다. CI 보안 정책을 완화하지 않는다. #7932의 기존 메인테이너 인수 기록을 확인했고 fork enforcement 변경 실패를 동일 저장소의 검증된 통합 후보로 처리하는 경로를 따른다.

## 완료할 검증

1. 원 이슈·선행 구현·CI 실패 원인, 실제 소비 경로와 독립 검증 자료를 확인한다.
2. cache·tile·PDF 집중 실행과 기존 정상 경계, CI 정책·workflow 계약을 검증한다.
3. Native/fresh WASM 실제 출력과 독립 Print를 대조한다. 합성 API 계약은 문서 피델리티와 구분한다. 최신 devel에서 보정된 글꼴·이미지·caption 경로의 유지도 확인한다.
4. 새 후보에서 필수 Rust lint·전체 회귀·Native Skia·최적화 WASM 검증을 완료하고 원 PR별 review에 결과를 기록한다.
5. 검증된 후보와 증적을 구체화한 뒤 승인된 원격 범위에 따라 통합한다. 종료 정리는 정본 post_merge 절차를 따른다.

실행하지 않은 항목은 완료로 표시하지 않는다. 캐시 상한은 bitmap 예산이며 프로세스 RSS 상한이나 속도 개선 전체를 주장하지 않는다.
