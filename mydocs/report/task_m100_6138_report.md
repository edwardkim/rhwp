# 최종 보고서 — #6138 기본 도구 상자 한 줄 그룹 스크롤

- **이슈**: [#6138](https://github.com/edwardkim/rhwp/issues/6138)
- **작업 브랜치**: `codex/issue-6118-responsive-style-bar`
- **기준**: `upstream/devel@1011a8947`
- **완료일**: 2026-08-26 KST
- **판정**: #6138 로컬 완료, #6118 통합 PR 승인 대기

## 1. 결과

기본 도구 상자를 viewport에 따라 1~3행으로 접거나 label을 없애던 정책에서 56px 한 줄 group scroll
정책으로 전환했다.

- 1280px 이상처럼 track 전체가 들어가면 이동 버튼을 숨기고 모든 group을 그대로 표시
- track이 넘치면 양쪽 이중 꺾쇠와 native horizontal viewport를 표시
- 모든 지원 너비에서 label 포함 44px desktop button 밀도와 56px 높이 유지
- 다음/이전은 command 하나가 아니라 track 기준 가시 group의 시작 경계로 이동
- 시작에서는 이전, 끝에서는 다음 버튼을 숨기고 24px slot은 양 끝 padding처럼 유지
- wheel·touch·keyboard·focus와 mode·resize·toolbox visibility 변화 지원
- 기존 group·command DOM, 순서, ID, listener와 상태 authority 재사용

외부 `#icon-toolbar` ID를 유지해 #6115의 접기/펴기와 충돌하지 않는다. 새 controller는 레이아웃과 이동
상태만 소유하고 command를 복제하거나 실행하지 않는다.

## 2. 단계별 산출물

| 단계 | 산출물 | 핵심 결정 |
| --- | --- | --- |
| 계획 | [수행계획](../plans/task_m100_6138.md), [구현계획](../plans/task_m100_6138_impl.md) | 모든 너비 56px 한 줄·group scroll |
| Stage 1 | [기준선 계측](../working/task_m100_6138_stage1.md) | desktop track 1219px, 기존 1~3행 회귀 고정 |
| Stage 2 | [구현 결과](../working/task_m100_6138_stage2.md) | 단일 DOM track·동적 overflow controller |
| Stage 3 | [통합 검증](../working/task_m100_6138_stage3.md) | 12 viewport, 18 theme cases, #6118 동시 검증 |

## 3. 시각 결과

| 1280px 전체 표시 | 1024px group scroll | 375px group scroll |
| --- | --- | --- |
| ![1280px 전체 표시](assets/task_m100_6138/toolbar-wide-1280.png) | ![1024px 한 줄 스크롤](assets/task_m100_6138/toolbar-scroll-1024.png) | ![375px 한 줄 스크롤](assets/task_m100_6138/toolbar-scroll-375.png) |

넓은 화면의 nav는 레이아웃과 접근성 트리에서 모두 사라진다. 좁은 화면에서는 한글 2024와 같은 이중
꺾쇠 의미를 사용하되, 명령을 메뉴로 옮기지 않고 기존 group 순서를 안정적으로 탐색한다.

## 4. 최종 검증

| 게이트 | 결과 |
| --- | --- |
| TypeScript | 통과 |
| Studio 전체 test | 1,148 passed, 0 failed, 1 skipped |
| Studio production build | 통과 |
| responsive/theme/#6118 통합 E2E | 610 passed, 0 failed |
| 대표 화면 육안 검토 | 1280·1024·375px 통과 |
| Markdown 상대 링크·diff whitespace | 603문서 이상 없음·통과 |
| review checkout Rust manifest·format | 942 sources·32 harnesses·9 exceptions, fmt 통과 |

이 변경은 Studio chrome만 대상으로 하므로 renderer PDF/SVG visual sweep 대상이 아니다. 최신 기준선의
E2E manifest 미등재 세 파일은 이번 변경 밖의 기존 상태다. Rust source 변경은 없으며 파생 suite를
준비한 별도 review checkout에서 Rust manifest와 필수 format 게이트까지 통과했다.

## 5. 통합 제출 전략

#6118은 아래쪽 `#style-bar`의 1·2행·더보기 정책이고 #6138은 위쪽 `#icon-toolbar`의 한 줄 group scroll
정책이다. 두 이슈의 문서와 커밋은 분리했지만 사용자에게는 한 번에 보이는 인접 chrome 변경이므로
12개 viewport와 18개 theme 조합을 함께 검증했다. 원격 push와 PR 생성은 아직 수행하지 않았으며 사용자
승인 뒤 두 이슈를 연결한 PR 한 건으로 제출한다.

사용자 시각 검토에서 발견한 32px anchor 오차는 group 좌표를 track 기준으로 정규화해 보정했다. 시작·
중간·끝의 방향 버튼 표시도 한글 2024 참고 동작과 맞추되, 숨긴 slot을 제거하지 않아 viewport가 이동 중
늘거나 줄지 않게 했다.
