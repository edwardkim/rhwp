# Task M100 #2072 Stage 13 - 비권위 브레인스토밍 분류

## 목표

`mydocs/tech` 루트의 재감사에서 발견한 iPad 앱 브레인스토밍을 장기 기술 사실·설계 문서와 분리한다.

## 판단

`brainstorm_ipad_app.md`는 2026-04-09의 사고 실험과 시장 가설을 기록한 문서다. 확정된 제품 계획,
기술 계약 또는 현재 구현 reference가 아니므로 `tech/archive/`의 historical snapshot으로 분류한다.

GitHub issue/PR 본문·코멘트와 저장소 Markdown에서 기존 파일명 참조는 0건이다. redirect는 남기지 않고
이전 경로 금지 검사만 추가한다.

## 검증 계획

- 기본 상대 링크 검사
- 전체 `mydocs` 이전 경로 금지 검사
- `actionlint .github/workflows/docs-link-check.yml`
- `git diff --check`

## 결과

- `brainstorm_ipad_app.md`를 `tech/archive/`로 이동하고 historical snapshot으로 색인했다.
- 외부·내부 참조가 없어 redirect는 만들지 않았고 이전 경로를 CI 금지 목록에 추가했다.
- 기본 상대 링크 검사 308개와 전체 `mydocs` 7,103개의 이전 경로 금지 검사가 통과했다.
- `actionlint`와 `git diff --check`가 통과했다.
