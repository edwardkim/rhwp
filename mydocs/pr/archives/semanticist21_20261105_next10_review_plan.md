# semanticist21 미지정 리뷰어 PR 10건 누적 검토 계획 — 2026-11-05 두 번째 묶음

## 완료 상태

원격 처리 승인을 받아 [통합 PR #8640](https://github.com/edwardkim/rhwp/pull/8640)을 병합했고 원 PR 10건도 자동 MERGED로 반영됐다. [최종 기록](pr_8640_review.md)에 실제 head·CI·merge SHA를 기록했다. 아래는 접수·진행 당시의 단계별 기록이며 당시 미완료 표기를 현재 상태로 읽지 않는다.

## 최신 진행 상태 — 고정 head의 로컬 검증·현재 base 호환 완료

원 head 10개를 merge 조상으로 보존했고 최신 base `6e1de406209ba9a58032f6ca6de45b18aa687d1c`를 통합했다. 최종 제품 source는 `584c9503baa651c62d0403dad9cb77c5cb9ffde6`다. 필수 로컬 검증 19단계를 완료했고 전체 nextest 12,629 PASS·45 SKIP, Native 공식 3종 4,064 PASS·12 IGNORE, release lib 3,983 PASS·12 IGNORE, 모든 lint·workspace build·doc test·Studio 2,720건이 통과했다.

시각 producer `d53115fabc92bd392044d9b906c599baddb4368d`의 이번 10건 124쪽과 새 base 보존 406쪽은 Native/fresh WASM 양쪽에서 전부 90% 이상이다. 직접 확대 판독과 canonical Native 편람 383쪽도 완료했다. 이후 계산 API의 비유한 결과만 보정한 최종 source의 7쪽은 양쪽 모두 100%·이전 PNG와 같다. producer를 구분해 보존했다.

[최종 검토](semanticist21_20261105_next10_review.md), [실제 실행·TSV·확대 판독](../assets/semanticist21-20261105-next10/validation.md), [PR 본문](../assets/semanticist21-20261105-next10/pr-body.md), [원격 처리와 댓글 초안](../assets/semanticist21-20261105-next10/remote-action-plan.md)을 준비했다. 현재 승인은 로컬 보정·필수 검증이며 원격 push·새 PR·merge·공개 댓글은 수행하지 않았다.

최종 fetch에서 현재 base가 `a95d1d5afad1bfd61998794fa78fb0db100d690a`로 전진했다. #8630의 실제 편집·소비 경로를 조사하고 [충돌 없는 현재-base 병합 트리의 실제 실행](../assets/semanticist21-20261105-next10/current-base-compatibility.md)을 완료했다. 이번 124쪽씩의 PNG 248개 동일, 새 경로 3·4쪽 양쪽 100%, Rust 집중 18건·실제 브라우저·Studio 2,721건이 통과했다. 검증한 source 584c9503ba를 반복 merge/rebase하지 않으며, 그 전체 실행을 새 병합 트리의 실행으로 바꾸어 표시하지 않는다.

아래 접수와 누적 보정 기록의 미완료 표시는 각 중간 시점의 상태다. 최신 완료 여부는 이 절과 최종 검증 문서에서 확인한다.

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

## 충돌 보정안 — 작업지시자 승인 완료

동일한 base와 각 원 head로 수행한 `git merge-tree --write-tree --name-only`에서 아래 세 파일에 충돌이 있다. 나머지 8건의 개별 병합 시뮬레이션은 충돌 없이 종료했다. 누적 통합의 추가 충돌·기능 간 상호작용은 아직 판정하지 않았다.

| 원 PR | 충돌 파일 | 제안 방침·함께 보호할 기존 계약 |
| --- | --- | --- |
| #8368 | `src/renderer/layout.rs`, `src/renderer/layout/paragraph_layout.rs` | 원 기여의 `far_side`와 공통 `create_box_border_line_nodes`를 통합하되, #8366·#8609의 `text_border_is_drawn` 가시성 계약을 유지한다. 경로 노드의 세로 이동은 기존 공통 payload 이동 helper를 소비하게 한다. 입체선 비표시·기존 아래 쇄선·일반 테두리와 원형/물결선 반례를 함께 검증한다. |
| #8369 | `src/renderer/skia/text_replay.rs` | 원 기여의 공통 `char_outline_stroke`(굵기·반복 구간)와 Native device 하한을 통합하되, #8403·#8410의 명시 `TextPaint` 채움·획 색·alpha·폭·join·miter 및 현재 shaping·그림자·양각 경로를 보존한다. HWP outline과 PDF 명시 paint가 서로의 획을 덮지 않는 실제 backend 대조군을 실행한다. |

작업지시자는 이번 응답에서 “기존 계약 보존·메인테이너 보정 승인”을 선택했다. 위 세 파일과 같은 계약의 누적 충돌 보정 및 10건 필수 로컬 검증을 승인한 범위로 진행한다. 원격 push·PR 생성·merge는 이 승인에 포함되지 않는다.

## 한 단위의 통합·검증 계획

1. 위 방침이 승인되면 원 PR 순서대로 같은 검토 branch에 적용하고, 원 head와 기능 commit SHA·실제 적용·보정 SHA를 각 원 PR 기록에 연결한다. 최신 base의 공통 편집 API·Storage 예산·schema·이미 수용한 표/그림/글꼴/문자 획 동작을 보존한다.
2. 최신 base에 문제가 남았는지 실제 입력·제품 API로 확인한다. helper 이름 부재만으로 미해결을 판정하지 않는다. #8500의 bit 21은 기존 상수·경로가 있으므로 새/기존 문단·셀·각주 및 편집 후 저장 플래그를 따로 대조한다.
3. 구현 주장별로 적용/비적용 경로·독립 한컴 저장값·기대 결과·실제 assertion을 연결한다. #8385 격자 분할과 부분 블록, #8484 함수 값/인수, #8488 각주 페이지/구역 재시작·뒤 구역 전파, #8490 잘못된 ID의 원자적 거부·DocInfo dirty, #8492 CTRL_DATA 양쪽 저장, #8494 구역별 미주·빈 구역·새 번호, #8500 머리 첫 줄 플래그, #8504 실제 셀 편집·계산 필드·순환/자기 참조·필드 뒤 텍스트를 확인한다. #8484가 함께 들어오므로 #8504가 제외했던 LN·자릿수 ROUND도 통합 경계로 확인한다.
4. 공용 Cargo target은 `/home/edward/mygithub/rhwp/target/pr-review` 하나를 순차로 사용한다. 기존 테스트 자동 수집 경로를 먼저 확인하고 불필요한 CI 호출을 추가하지 않는다. 파생 suite·manifest는 검증 checkout에서 준비하되 stage하지 않는다.
5. 조판·렌더링·편집 후 배치에 영향을 주는 범위는 같은 원본·독립 한컴 Print PDF의 전쪽 Native/fresh WASM TSV를 최신 base/후보에서 산출한다. 90% 미만·2%p 초과 하락·쪽수/내용 누락·미측정은 제출 보류다. 기존 screenshot만 있는 #8494·#8500·#8504와 파일로 된 입력이 없는 #8484·#8490은 필요한 독립 입력·Print 자료를 먼저 확보한다. 구조 저장 보존 전용 범위는 실제 소비 경로를 근거로 비해당 여부를 판정하며 편집 영향 전체를 임의로 면제하지 않는다.
6. 실제 SVG·Canvas2D·Native Skia 및 관련 CanvasKit 소비 경로를 확인한다. Native/fresh WASM의 해당 원본 전체 쪽수·소유·순서·누락·중복·저장 재열기와 대표 경계/미세 간격의 직접 확대 판독을 함께 완료한다. 회귀 추가/변경은 독립 측정의 선행 조건을 충족한 뒤 수행한다.
7. 최종 후보에서 최소 글꼴 환경의 집중·전체 nextest, fmt·Native/WASM32/workspace Clippy·workspace build, manifest/unit-tier, 필수 Native 묶음, Studio와 관련 API/E2E·package 검증, 신규 입력 보안·래칫, Native 편람 383쪽 gate를 변경 범위에 맞게 완료한다. 이미 같은 exact code head에서 성공한 검사의 정본 재사용 조건이 성립할 때만 재사용 이유를 명시한다.
8. 각 원 PR review의 판정과 수용 조건을 확정하고 최종 대표 PNG·전체 TSV·입력/PDF를 정식 경로에 보존한다. 중간 JSON·raw raster·실행 로그는 커밋하지 않는다. 원격 처리가 필요하면 그 시점의 concrete diff·검증 결과·PR 본문·댓글 초안으로 승인 범위를 확인한다.
9. 이후 승인된 merge·후속 처리는 개정된 [post_merge.md](../../manual/pr_review/post_merge.md)를 따른다. CI 정의·판정 정책을 실제 변경하게 되면 옵션 2로 기록을 분리하며, 코드 PR에서 시험 trailing push를 하지 않는다. 원 PR마다 일치하는 review/overlay·수치와 게시 전 이미지 hash/응답·게시 후 API/화면 확인을 수행한다. 종료 시 전용 branch/worktree·root output 전체·검토 debug를 정리하고 다른 작업과 배포 WASM을 보존한다.

## 실제 완료·미완료 구분

완료: 10건 선정·원 PR별 reviewer 지정/API 확인·head 고정·본문/diff/CI 조사, 10건 개별 merge-tree, 격리 worktree 생성, CI 실패 3건의 실제 로그와 선행 보정 확인.

완료: 충돌 보정 방침의 작업지시자 승인.

미완료: 기능 누적 적용·메인터너 보정, 현재 base/후보의 실제 동작·시각·필수 로컬 검증, 수용 판정, 원격 push·통합 PR·merge·후속 댓글. 원 PR CI 성공이나 기여자 측 측정은 이번 통합 후보의 검증 완료로 간주하지 않는다.


## 누적 적용 후 확인한 보정과 독립 근거

원 head 10개를 순서대로 merge해 기여 commit을 조상으로 보존했다. #8368·#8369의 승인된 충돌 방침을 적용했고, CanvasKit 글자 외곽선도 Rust 공통 획 결과를 소비하도록 보정했다. 누적 source head `195522528b`에서 TypeScript와 Studio 2,720건이 통과했지만, 이 결과는 fresh WASM·최종 검증을 대신하지 않는다.

- #8385: `Table::equalize_cell_block → per_line → n`에서 빈 줄의 `n == 0` 확인보다 나눗셈이 먼저 실행됐다. 검증을 앞당겨 오류를 반환한다. 현재는 코드 검토로 확인했으며 실제 수정 전 실패·수정 후 통과는 아직 실행 전이다.
- #8494 독립 Print 준비 중 기존 바탕쪽 텍스트 좌표 오류를 확인했다. `restart-5.hwpx`의 `masterpage0/1.xml`은 `textWidth=42520`, `textHeight=65762`, 첫 저장 줄 `vertpos=0`이다. 독립 Windows 한컴 2020 Print PDF의 `바탕쪽표식`은 x=85.03391pt, y=99.38459pt에 있으며 본문 텍스트 프레임 원점과 일치한다. 기준 Native는 이를 용지 (0,0)에 그린다. `build_master_page → paper_area 기반 mp_y_offset 및 layout_paragraph 영역 → 각 backend` 연결에서 텍스트 프레임 원점을 잘못 공급했다. 이미 바탕쪽 단 구분선이 소비하는 본문 원점과 같은 프레임을 일반 텍스트에도 공급한다. 도형·그림의 Paper/Page 기준 위치는 기존 경로를 유지한다. 보정 후 전쪽 측정과 실제 master 원점·도형 대조는 아직 완료 전이다.
- #8488: 독립 Mac Print와 기준 Native의 각주 위치·선·위첨자가 다르다. API의 저장 보존과 실제 각주 배치를 구분해 원인을 조사하고 있으며, 낮은 점수를 시각 통과로 기록하지 않는다.


각주 배치의 원인 연결: `build_single_column`은 이미 마지막 본문 항목의 실제 점유 하단(`last_item_content_bottom`)을 반환하지만, `build_columns`가 이를 각주 배치에 전달하지 않았다. `print_inline_after_text`가 설정된 경우 그 공통 결과를 각주 원점으로 사용한다. 빈 줄의 점유와 부동 도형의 잉크를 혼동하지 않는다. 기존 쪽별 각주 예약·분할 큐는 유지한다. 구분선은 고정 실선 대신 기존 HWP 선 스타일 변환을 소비한다. 자동 번호의 위첨자는 composer가 `footnote_marker`로 생산했으나 `ComposedTextRun::text_style`에서 소비하지 않았다. 일반 글자 위첨자와 다른 주석 번호의 3/4 크기·기준선(-0.21em) 계약을 공통 `ScriptMetricsContract`에 명시해 측정·각 paint로 전달한다. 기존 `FootnoteMarkerNode`의 독립 계약 및 Mac Print의 본문 9pt/번호 6.72pt와 대조한다. 현재 보정 후 실제 전쪽·반례 검증은 진행 전이며 수용 판정을 보류한다.


첫 보정 Native(`0838c170b7`, binary SHA-256 `72492c533280e439953e91b61c0e0a2944745c186aff4b4942adf3fcf6399f66`)에서 미주 원본 세 개의 전 12쪽은 모두 raw 100%였다. 각주는 1쪽 95.10%·2쪽 85.28%로 아직 미달이다. 확대 판독에서 2쪽의 로마자 문단 머리 II가 선두 각주 참조 폭만큼 우측으로 밀려 참조와 겹치는 기존 오류를 확인했다(base와 후보가 동일). `leading_note_markers → endnote_marker_x_advance → x_start`는 본문 참조의 흐름 전진인데, 문단 머리도 그 전진을 더한 x를 사용했다. 문단 머리의 실제 배치 원점에서 그 참조 advance를 제외하고 문단 머리 → 참조 → 본문 순서를 보존한다. 각주 본문 번호의 재투영도 원 AutoNumber의 위첨자 정보를 명시적으로 전달하고 주변 본문에는 전달하지 않는다. 점수를 맞추는 고정 좌표나 문서별 조건을 추가하지 않는다. 수정 후 전쪽·정상 대조군은 재검증 전이다.

실제 제품 API 임시 driver에서 셀 균등화의 한컴 격자 대조, 미주 이어 세기·삽입·삭제, 번호 ID 2의 `가)제목`, 머리 첫 줄 플래그와 저장을 실행했다. #8484/#8504 통합 경계의 LN(0)·ROUND 7자리는 한컴 저장본 C7·D8과 같았다. driver는 정식 집중/전체 회귀나 최종 시각 gate를 대체하지 않는다. 최소 글꼴 집중 회귀 첫 실행은 컴파일 중 SIGTERM(143)으로 종료돼 테스트 결과가 없으며 통과로 기록하지 않는다.


`c6120525f8` Native(`7c47905d1857c84c00839239d35ad0adc3b977956cc061c986fabd8c0df4e939`)에서 각주 전 2쪽과 미주 세 원본 전 12쪽은 모두 raw 100%다. 같은 head의 원 PR 회귀 모듈 10개를 실제 Native library에 연결해 최소 글꼴 환경에서 실행한 진단 검사 42건은 통과했다. 이 실행은 정식 nextest 전체 회귀·fresh WASM 완료와 구분한다.

추가 API 경계의 실제 실패: `set_section_def_all_native({"outlineNumberingId":65536,"pageNum":5})`는 성공을 반환하면서 ID 1→0·pageNum 0→5로 바뀌었다. 새 ID를 기존 `json_u16`의 `as u16`으로 읽어 overflow가 감겼기 때문이다. 해당 새 필드만 typed JSON의 `Option<u16>`으로 먼저 검증하고 ID 존재를 확인한 뒤 기존 갱신 경로에 전달한다. 다른 구역 필드의 파싱·저장 의미는 유지한다. 기존 무효 ID 검사를 overflow·음수·분수·문자열·객체로 넓혀 전체 상태가 불변인지 확인한다. Studio `SectionDef`에도 선택적 ID 선언을 보완한다. 실제 실패 전 출력은 보존했고 수정 후 검증은 재실행 전이다.


## 확대 판독에서 확인한 저장 선 종류의 후속 보정

Native의 원본 38쪽·실제 편집 저장 60쪽·PDF 대조군 12쪽은 전 110쪽 raw 90% 이상(최저 96.60%)이다. 이를 최종 완료로 판정하지 않았다. `footnote-shape-applied.hwpx` 2쪽 확대 판독에서 후보는 짧은 점선, 동일 입력의 독립 Windows Print는 긴 파선이었다. HWP와 HWPX의 선 이름 변환이 서로 반대로 처리되는 기존 결함이다.

선행 해결을 추적했다: #3930 → 원 PR #4058 → `47216ca6bd0eab6540ff925eaeaee3291f7a7d94` → merge `efed25b43a`. 이 보정은 현재 base의 조상이며, header parser/serializer의 독립 계약 검사도 남아 있다. `parse_border_line_type_code`와 `border_line_type_str`는 HWP5 raw 2 ↔ HWPX `DOT`, raw 3 ↔ `DASH`를 보존한다. 그런데 각주의 `section.rs` parser와 `note_line_type_str`는 반대의 별도 매핑을 유지했다.

같은 raw 3의 HWP 저장본 Print는 잉크 1.679pt/간격 2.519pt, 기존 HWPX 저장본(`type=DOT`) Print는 잉크 8.276pt/간격 5.037pt였다(획 폭은 둘 다 약 1.079pt). 임의로 둘 중 하나의 화면에 맞추지 않고, 기존 공통 LineType2 parser/serializer 결과를 각주도 소비하도록 보정한다. 기존 잘못 저장된 HWPX도 해당 파일의 실제 `DOT` 의미대로 읽고, 새 저장본은 원 HWP raw 3을 `DASH`로 보존하게 한다. 현재는 원인·선행 근거 확인을 완료했으며, 해당 source 보정과 새 저장본 Print 재확인은 아직 진행 전이다.


저장 선 종류 보정은 header의 기존 LineType2 변환 함수를 각주·미주도 재사용하도록 적용했다. paint도 같은 `BorderLine` 생산 결과와 `create_border_line_nodes`를 소비해 16개 선언 굵기의 공통 Print 반복 구간을 쓴다. `stroke_pattern/border.rs`의 raw 폭 index 6은 짧은 점/공백 14/21, 긴 점/공백 69/42(1/600 inch)이며, 이번 독립 Print에서 측정한 1.679/2.519pt와 8.276/5.037pt에 부합한다. 문서 ID나 화면 픽셀에 맞춘 상수를 추가하지 않았다. 각주·미주의 기존 예약·흐름 높이는 유지하며 실제 선만 공통 결과를 소비하게 했다. 수정 후 전쪽·왕복·정상 대조군은 재검증 전이다.
