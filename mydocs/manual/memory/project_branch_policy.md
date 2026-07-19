---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: 프로젝트 브랜치 정책 + iOS 분기
description: main/devel/local-devel/local-task{N} + ios/devel(맥북 전용) 장기 분리
type: project
originSessionId: 4861649d-834a-43c6-a262-9f08333360e8
---
**브랜치 정책 (canonical: `mydocs/manual/codex/docs_and_git_workflow.md` — 구 CLAUDE.md에서 이관, 2026-07-17):**

| 브랜치 | 역할 | 비고 |
|--------|------|------|
| `main` | 릴리즈 | 태그(v0.5.0 등) 안정 버전 |
| `devel` | 개발 통합 (원격) | 메인테이너 + 외부 PR 머지 대상 |
| `local/devel` | 로컬 작업 (push 금지) | 메인테이너 로컬에서 통합 |
| `local/task{N}` | 타스크 작업 (push 금지) | issue별 분리 |
| `ios/devel` | iOS 앱 (맥북 전용 — alhangeul-macos) | **장기 분리, 추후 통합** |
| `skia/devel` | 별도 브랜치 운영 검토 중 | 미확정 |

**Why**: macOS에서만 빌드 가능한 iOS 앱(Core Graphics/Swift/Xcode)은 Linux 메인 환경과 분리. 충돌 위험 최소화 + iOS 작업이 main devel 사이클에 영향 미치지 않도록.

**How to apply**:
- iOS 앱 작업은 `ios/devel`에서만 진행. 일정 주기로 devel 머지로 동기화
- 외부 컨트리뷰터 PR base 동기화 점검 (PR base가 devel 뒤처지면 close 권장)
- `local/devel`/`local/task*`는 절대 원격 push 금지
- iOS 작업 이슈는 GitHub 마일스톤(M1/M2/M3...)에 분리 등록 (현재 M2)

**관련 메모리**: `feedback_first_pr_courtesy.md` (외부 PR base 처리)
