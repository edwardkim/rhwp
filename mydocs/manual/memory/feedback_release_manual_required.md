---
name: 릴리즈/배포 작업 시 매뉴얼 정독 필수
description: 빌드/배포/버전 갱신 작업 시작 시 mydocs/manual/ 의 관련 매뉴얼 전체 정독 필수. 부분 검색만으로 진행 금지
type: feedback
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
빌드/배포/버전 갱신 등 매뉴얼이 존재하는 영역의 작업은 **시작 시 관련 매뉴얼 전체 정독** 필수.

**Why**: 2026-04-19 v0.2.0 (확장) 배포 작업 시 `mydocs/manual/chrome_edge_extension_build_deploy.md` 의 §5.1 (버전 변경 대상 4개 파일) 을 정독하지 않고 부분 검색 (`manifest.json`, `package.json` 만) 으로 진행. 결과적으로 `dev-tools-inject.js` 의 `VERSION` 상수와 `content-script.js` 의 `data-hwp-extension-version` 2개 파일 갱신 누락 → 사용자 console / DOM attribute 에서 0.1.1 표시 → hotfix v0.2.1 발생.

**How to apply**:
- 작업 시작 첫 단계로 `Read` 도구로 매뉴얼 **전체 정독** (Glob/Grep 부분 검색 X)
- 매뉴얼의 체크리스트 항목 (예: §5.1 의 4개 파일) 모두 작업 항목으로 옮김
- 각 항목 처리 후 매뉴얼과 대조하여 누락 검증
- 작업 완료 보고 시 매뉴얼 명시 항목과 1:1 대조 표 첨부

관련 매뉴얼 (현재 시점):
- `mydocs/manual/chrome_edge_extension_build_deploy.md` (Chrome/Edge 확장)
- `mydocs/manual/browser_extension_dev_guide.md` (확장 개발 일반)
- `mydocs/manual/dev_environment_guide.md` (개발 환경)
- `mydocs/manual/onboarding_guide.md` (신규 온보딩)
- 기타 `mydocs/manual/*.md`
