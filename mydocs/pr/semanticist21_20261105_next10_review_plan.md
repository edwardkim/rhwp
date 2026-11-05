# semanticist21 미지정 리뷰어 PR 10건 누적 검토 계획 — 2026-11-05 두 번째 묶음

## 현재 상태와 범위

작업지시자는 비 Draft·reviewer 미지정 PR을 오래된 순서로 10건 선정해 묶어 검토하고, CI 실패의 원인에 따라 메인터너 보정 또는 기여자 댓글을 선택하도록 지시했다. 접수와 읽기 전용 사전 조사를 완료했다. 수용 판정·구현 완료·시각 통과를 뜻하지 않는다.

- 기준 `upstream/devel`: `ed057276c1e0e07e743218d95641e015b46b6715`.
- 검토 branch: `review/integration-20261105-semanticist21-next10`.
- 검토 worktree: `/home/edward/mygithub/rhwp-worktrees/review-integration-20261105-semanticist21-next10`.
- 기본 작업공간 `/home/edward/mygithub/rhwp`의 `devel`과 기존 별도 `pr-8296-seal-pua` worktree는 보존했다.
- 앞선 같은 날 10건은 [통합 PR #8609](https://github.com/edwardkim/rhwp/pull/8609)에서 완료했다. 이번 선정과 증적은 별개다. 당시 #8331·#8332의 예외 승인을 이번 PR에 적용하지 않는다.
- 현재 인증 계정의 열린 PR은 0건이다. 전체 semanticist21 Open 29건 가운데 reviewer가 있는 #8226·#8286을 제외한 27건에서 생성 시각 오름차순 첫 10건을 선정했다.
- 리뷰어는 원 PR마다 `edwardkim`을 지정하고 `requested_reviewers.users` API 및 지정 전후 head 동일성을 확인했다. 사용자·팀 기존 reviewer를 제거하지 않았다.

## 라우팅

- base route: `maintainer_general.md`.
- modifiers: `intake_and_review.md`, `local_validation.md`, `multi_pr_update_branch.md`, `visual_fixture_evidence.md`, `post_merge.md`; 1,000줄 초과 #8504에는 `rework_and_exceptions.md`도 적용한다.
- source 보정과 시각 검증 뒤 원격 처리가 필요하면 당시의 exact head와 승인 범위를 확인한다. 이번 지시의 CI 처리 선택·필요한 댓글 작성 권한을, 새 통합 branch push·PR 생성·merge의 승인으로 확대하지 않는다.

## 선정과 고정 head

| 원 PR | 원 head SHA | 주요 범위 | reviewer API 확인 시각(UTC) |
| --- | --- | --- | --- |
| [#8368](https://github.com/edwardkim/rhwp/pull/8368) | 7915a7ed9f75f559f3e82aa9a18feb80c0326ce1 | 원형 점선·물결선 테두리 | 2026-11-05T01:40:59.983552+00:00 |
| [#8369](https://github.com/edwardkim/rhwp/pull/8369) | 3c5accd7b604594585ebc1b1fe96ea155069821e | 글자 외곽선 종류 | 2026-11-05T01:41:03.191392+00:00 |
| [#8385](https://github.com/edwardkim/rhwp/pull/8385) | 6a31cb47a088ac13ecbe641900318c9dbc9ff4e3 | 셀 높이·너비 같게 | 2026-11-05T01:41:06.321854+00:00 |
| [#8484](https://github.com/edwardkim/rhwp/pull/8484) | eff6d08cfc2bf781a934f358250d676a4fc775cd | 표 계산식 함수 | 2026-11-05T01:41:09.772554+00:00 |
| [#8488](https://github.com/edwardkim/rhwp/pull/8488) | 9fe4a4df5e2271f128be49dc4457e2053ef9f25b | 각주 모양 API | 2026-11-05T01:41:13.448553+00:00 |
| [#8490](https://github.com/edwardkim/rhwp/pull/8490) | 6b86197e31f820c7ba5c098f3bf34d9e50b739e7 | 구역 개요 번호 ID | 2026-11-05T01:41:16.808289+00:00 |
| [#8492](https://github.com/edwardkim/rhwp/pull/8492) | c1dc63845879c92506a1da1aaa17ef233b7c68d2 | 문단 첫 글자 장식 저장 | 2026-11-05T01:41:20.115903+00:00 |
| [#8494](https://github.com/edwardkim/rhwp/pull/8494) | d8f560f7e62c9f70cdf935d10a26b89c7362d014 | 구역별 미주 번호 | 2026-11-05T01:41:23.378731+00:00 |
| [#8500](https://github.com/edwardkim/rhwp/pull/8500) | ddba007430877afdc2bc2c11d6fdcf6676c72801 | 문단 머리 첫 줄 플래그 | 2026-11-05T01:41:26.955346+00:00 |
| [#8504](https://github.com/edwardkim/rhwp/pull/8504) | a7e665eb44f47a50481368722fae71461f15f901 | FORMULA 필드·재계산 | 2026-11-05T01:41:30.342137+00:00 |

원 contributor commit·fork branch는 보존한다. 원 PR의 devel merge는 기능 체리픽 대상에서 제외하고, 실제 적용 SHA·순서·후속 보정은 각 `pr_N_review.md`에 기록한다.

## CI 원인과 메인터너 판단

| 원 PR·실패 작업 | 실제 관측 | 판단과 다음 검증 |
| --- | --- | --- |
| [#8369 Archive A](https://github.com/edwardkim/rhwp/actions/runs/40848621019/job/123362362200) | checkout 단계가 약 30분 동안 종료되지 않아 job 상한 30분으로 취소됐다. 컴파일·테스트가 시작되지 않았다. | 기여자 코드 재작업 댓글을 요구하지 않는다. 최신 base의 archive 60분 정책을 보존하며 새 통합 후보에서 필수 로컬 검증을 완료한다. 과거 취소를 PASS로 기록하지 않는다. |
| [#8484 Archive C](https://github.com/edwardkim/rhwp/actions/runs/41479287092/job/125399830800) | `issue_6612_sample14_wmf_pictures_reach_pdf`의 PDF 이미지 개수는 기대 21·실제 101이었다. 계산식 함수 검사가 실패한 것이 아니다. 로그에 글꼴 부재와 subset fallback이 함께 있다. | #6612 → 원 해결 PR #6618 → `7946a457d9` → #8187의 `8c97771760` 보정과 독립 전후 증거를 추적했다. 해당 보정이 최신 base에 포함됨을 확인했다. 원 WMF 21개와 문자용 Emoji 이미지 자원을 분리하는 명시 글꼴 계약을 그대로 재검증하며 기대값을 완화하지 않는다. |
| [#8488 Frontend](https://github.com/edwardkim/rhwp/actions/runs/41490425984/job/125432091345) | fresh WASM의 `librhwp.rlib` 생성 중 `No space left on device (os error 28)`로 종료됐다. | #8565에서 반영한 현행 Frontend 빌드 전 디스크 확보 절차를 확인했다. 기여자 기능 재작업을 요구하지 않고 최신 base 정책과 fresh WASM 실제 빌드로 검증한다. 재실행 성공을 근본 원인 해결의 증거로 단정하지 않는다. |

세 실패 모두 현재 단계에서는 메인터너 측 최신 base 통합·검증을 택한다. 공개 재작업 댓글은 게시하지 않았다. 로그는 `output/pr-review/semanticist21-20261105-next10/ci/`의 임시 자료이며, `gh run view --log`의 빈 응답은 증거로 사용하지 않았다. jobs REST 원본 로그·annotation·시각·실패 step을 확인했다. workflow 취소·재실행·설정 변경은 하지 않았다.

## 충돌 보정안 — 작업지시자 결정 대기

동일한 base와 각 원 head로 수행한 `git merge-tree --write-tree --name-only`에서 아래 세 파일에 충돌이 있다. 나머지 8건의 개별 병합 시뮬레이션은 충돌 없이 종료했다. 누적 통합의 추가 충돌·기능 간 상호작용은 아직 판정하지 않았다.

| 원 PR | 충돌 파일 | 제안 방침·함께 보호할 기존 계약 |
| --- | --- | --- |
| #8368 | `src/renderer/layout.rs`, `src/renderer/layout/paragraph_layout.rs` | 원 기여의 `far_side`와 공통 `create_box_border_line_nodes`를 통합하되, #8366·#8609의 `text_border_is_drawn` 가시성 계약을 유지한다. 경로 노드의 세로 이동은 기존 공통 payload 이동 helper를 소비하게 한다. 입체선 비표시·기존 아래 쇄선·일반 테두리와 원형/물결선 반례를 함께 검증한다. |
| #8369 | `src/renderer/skia/text_replay.rs` | 원 기여의 공통 `char_outline_stroke`(굵기·반복 구간)와 Native device 하한을 통합하되, #8403·#8410의 명시 `TextPaint` 채움·획 색·alpha·폭·join·miter 및 현재 shaping·그림자·양각 경로를 보존한다. HWP outline과 PDF 명시 paint가 서로의 획을 덮지 않는 실제 backend 대조군을 실행한다. |

[로컬 사전 검증 4.2](../manual/pr_review/local_validation.md#42-merge-simulation)의 충돌 해결 방침 승인 전에는 이 세 파일을 보정하지 않는다. 충돌 없는 8건의 자료 조사·원 해결 이력 확인은 계속할 수 있다. 이번 CI 원인 처리 판단을 충돌 방침 승인으로 간주하지 않는다.

## 한 단위의 통합·검증 계획

1. 위 방침이 승인되면 원 PR 순서대로 같은 검토 branch에 적용하고, 원 head와 기능 commit SHA·실제 적용·보정 SHA를 각 원 PR 기록에 연결한다. 최신 base의 공통 편집 API·Storage 예산·schema·이미 수용한 표/그림/글꼴/문자 획 동작을 보존한다.
2. 최신 base에 문제가 남았는지 실제 입력·제품 API로 확인한다. helper 이름 부재만으로 미해결을 판정하지 않는다. #8500의 bit 21은 기존 상수·경로가 있으므로 새/기존 문단·셀·각주 및 편집 후 저장 플래그를 따로 대조한다.
3. 구현 주장별로 적용/비적용 경로·독립 한컴 저장값·기대 결과·실제 assertion을 연결한다. #8385 격자 분할과 부분 블록, #8484 함수 값/인수, #8488 각주 페이지/구역 재시작·뒤 구역 전파, #8490 잘못된 ID의 원자적 거부·DocInfo dirty, #8492 CTRL_DATA 양쪽 저장, #8494 구역별 미주·빈 구역·새 번호, #8500 머리 첫 줄 플래그, #8504 실제 셀 편집·계산 필드·순환/자기 참조·필드 뒤 텍스트를 확인한다. #8484가 함께 들어오므로 #8504가 제외했던 LN·자릿수 ROUND도 통합 경계로 확인한다.
4. 공용 Cargo target은 `/home/edward/mygithub/rhwp/target/pr-review` 하나를 순차로 사용한다. 기존 테스트 자동 수집 경로를 먼저 확인하고 불필요한 CI 호출을 추가하지 않는다. 파생 suite·manifest는 검증 checkout에서 준비하되 stage하지 않는다.
5. 조판·렌더링·편집 후 배치에 영향을 주는 범위는 같은 원본·독립 한컴 Print PDF의 전쪽 Native/fresh WASM TSV를 최신 base/후보에서 산출한다. 90% 미만·2%p 초과 하락·쪽수/내용 누락·미측정은 제출 보류다. 기존 screenshot만 있는 #8494·#8500·#8504와 파일로 된 입력이 없는 #8484·#8490은 필요한 독립 입력·Print 자료를 먼저 확보한다. 구조 저장 보존 전용 범위는 실제 소비 경로를 근거로 비해당 여부를 판정하며 편집 영향 전체를 임의로 면제하지 않는다.
6. 실제 SVG·Canvas2D·Native Skia 및 관련 CanvasKit 소비 경로를 확인한다. Native/fresh WASM의 해당 원본 전체 쪽수·소유·순서·누락·중복·저장 재열기와 대표 경계/미세 간격의 직접 확대 판독을 함께 완료한다. 회귀 추가/변경은 독립 측정의 선행 조건을 충족한 뒤 수행한다.
7. 최종 후보에서 최소 글꼴 환경의 집중·전체 nextest, fmt·Native/WASM32/workspace Clippy·workspace build, manifest/unit-tier, 필수 Native 묶음, Studio와 관련 API/E2E·package 검증, 신규 입력 보안·래칫, Native 편람 383쪽 gate를 변경 범위에 맞게 완료한다. 이미 같은 exact code head에서 성공한 검사의 정본 재사용 조건이 성립할 때만 재사용 이유를 명시한다.
8. 각 원 PR review의 판정과 수용 조건을 확정하고 최종 대표 PNG·전체 TSV·입력/PDF를 정식 경로에 보존한다. 중간 JSON·raw raster·실행 로그는 커밋하지 않는다. 원격 처리가 필요하면 그 시점의 concrete diff·검증 결과·PR 본문·댓글 초안으로 승인 범위를 확인한다.
9. 이후 승인된 merge·후속 처리는 개정된 [post_merge.md](../manual/pr_review/post_merge.md)를 따른다. CI 정의·판정 정책을 실제 변경하게 되면 옵션 2로 기록을 분리하며, 코드 PR에서 시험 trailing push를 하지 않는다. 원 PR마다 일치하는 review/overlay·수치와 게시 전 이미지 hash/응답·게시 후 API/화면 확인을 수행한다. 종료 시 전용 branch/worktree·root output 전체·검토 debug를 정리하고 다른 작업과 배포 WASM을 보존한다.

## 실제 완료·미완료 구분

완료: 10건 선정·원 PR별 reviewer 지정/API 확인·head 고정·본문/diff/CI 조사, 10건 개별 merge-tree, 격리 worktree 생성, CI 실패 3건의 실제 로그와 선행 보정 확인.

미완료: 충돌 방침 결정, 기능 누적 적용·메인터너 보정, 현재 base/후보의 실제 동작·시각·필수 로컬 검증, 수용 판정, 원격 push·통합 PR·merge·후속 댓글. 원 PR CI 성공이나 기여자 측 측정은 이번 통합 후보의 검증 완료로 간주하지 않는다.
