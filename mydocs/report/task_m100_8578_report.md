# Issue #8578 최종 PR 검토 — 묶음 글상자의 클릭·Esc 편집 복귀

## 최종 판정

**머지 보류 — 사용자 승인 후 새 devel의 코드 통합을 리베이스했으며, 해당 코드의 재검증을 진행 중이다.**
새 base `6e1de406209ba9a58032f6ca6de45b18aa687d1c`의 #8609·#8622 등을 포함한다.
아래 최종 검토는 이전 후보 `47f061fd9a`에 대한 기록이며 새 head의 통과 판정이 아니다.
원격 push·PR 생성은 승인되었으나 재검증 완료 전에는 실행하지 않는다. PR·CI·병합은 미실행이다.
이 문서는 사용자 요청에 따른 제출 전 로컬 검토다. PR 번호를 예측하지 않으며, 채번과 정확한
code head CI 성공 후 작성할 `pr_N_review.md` 및 오늘할일을 대신하지 않는다.

검토 갱신: 2026-11-05 04:14 KST. 아래 SHA와 원격 상태는 이 시점의 관측이다.

## 접수 정보와 검증 신원

| 항목 | 확인 결과 |
| --- | --- |
| 이슈 | [#8578](https://github.com/edwardkim/rhwp/issues/8578), OPEN, 담당 `jangster77` |
| 작업 branch | `task/8578-group-text-editing` |
| 검토한 제출 후보 | `47f061fd9a359214e54796fa3f84cc383f4ab646` |
| 코드 검증 head | `67c7814f6e2bc02cc6f69365f1c24f581a6826ca` |
| 실제 프로그램·전체 회귀 producer | `e12096b9da2e7f00c03c5c175f76d198df284e56` |
| 최신 정책 base | `e96b500e3ea259c821f05440f0388355916a9e9b`, `upstream/devel`의 실제 원격 SHA와 일치 |
| 최초 base 프로그램 producer | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| PR·CI·reviewer | PR 미생성, GitHub CI 미실행, reviewer 지정 없음 |
| 본인 미병합 PR 선행 확인 | `jangster77`의 열린 PR 0건; 조회 성공 결과 `[]` |

최초 base 이후 #8603은 과거 검토 문서 3개만 추가했다. 리베이스 전후 source/test 하위 tree와
upstream 문서 보존은 [동일성 증거](../pr/assets/issue_8578/20261105/source-equivalence.json)로 확인했다.
검토 후보 `47f061fd9a`도 코드 검증 head에 문서·증적만 더한 상태다. 이 보고서를 담는 후속
로컬 commit도 문서만 추가한다. 과거 producer를 새 head에서 재빌드한 결과로 표시하지 않는다.

## 결함·원인과 공통 수정 경로

원본 HWPX의 section0 본문 문단 4에는 제목과 안내 본문의 두 글상자가 Group 자식으로 있다.
두 자식에 같은 글상자 번호 0을 부여하고, 편집 조회는 독립 도형만 해석해 클릭·입력이 실패했다.
문단 6의 `BEHIND_TEXT` 장식 Group도 외곽만으로 글상자 후보가 되어 4쪽 목차를 가로챘다.
Esc는 개체 삭제용 인접 문단 착지를 재사용해 원 소유 본문 문단 4 대신 문단 5로 이동했다.

| 수정·소비 경로 | 실제 코드와 검사 근거 |
| --- | --- |
| 원본 순서의 글상자 식별 | `src/model/shape.rs`의 `textbox_shape_count/at/at_mut`가 실제 글상자만 깊이 우선으로 센다. 장식·중간 Group은 번호를 소비하지 않고, 실제 빈 글상자는 소비한다. |
| 생산·조회·변경의 주소 공유 | `shape_layout.rs`가 자식 offset을 해당 경로 depth에 반영한다. `helpers.rs`의 indexed helper를 커서·본문 편집·clipboard·중첩 표·inline control·hyperlink가 사용한다. normalization cache도 글상자 index를 포함한다. |
| 실제 글상자만 클릭 후보 | `RenderNode.textbox_source`는 실제 글상자 배치에서만 생산하는 내부 metadata이며 직렬화하지 않는다. `cursor_rect.rs`는 이를 소비해 장식 도형을 제외하고 소유 경로를 반환한다. |
| Esc 복귀와 인접 착지 분리 | Studio `returnToSelectedObjectOwner()`가 원 본문·중첩 셀 주소를 복구한다. 삭제·붙여넣기의 기존 인접 이동은 유지한다. |
| 빈 Group 소유 본문 캐럿 | `layout_shape`의 확정 본문 흐름 원점을 기존 숨은 문단 표시기에 전달한다. Column/Body만 적용하며, 중복 본문 캐럿과 Header/Note 주소 누출을 방지한다. |
| 입력 후 재조판 | 자식 글상자의 원본 orgSz와 변환으로 입력 폭을 구한다. 바깥 Group 외곽이나 비어 있는 child common 폭으로 대신하지 않는다. |

문서 이름·쪽 번호·문자열로 구현을 분기하지 않는다. Group을 해제·복제하거나 저장 원문을 바꾸지
않는다. 기존 #1892의 실제 공백 보존과 #919의 중첩 표 바깥 글상자 소유 계약을 확인했고,
실패 기대값을 완화하지 않고 공통 구현을 보정했다. 원 해결 PR·통합 보정 근거와 중간 실패는
[단계 기록](../working/task_m100_8578_group_text_editing.md#전체-회귀에서-발견한-호환성-보정)에 연결한다.

## 검증 입력과 최종 결과

- 원본: [2025 행정업무운영 편람(최종).hwpx](../../samples/2025%20행정업무운영%20편람%28최종%29.hwpx),
  SHA-256 `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`.
- 독립 한컴 Print: [편람 HWPX KoPub 2024 PDF](../../pdf/2025%20행정업무운영%20편람%28최종%29-hwpx-kopub-2024.pdf),
  SHA-256 `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`.
  기존 #8389에서 검증한 383쪽 자료를 재사용했고 새 MCP Print job을 만들지 않았다.
- 실제 실행 파일과 검토 commit의 저장 blob을 SHA로 대조한다. 새 입력·기준 PDF·글꼴·허용치 변경은 없다.

| 항목 | 실제 결과·증거 |
| --- | --- |
| Linux 최소 글꼴 전체 nextest | 12,489 PASS / 0 FAIL / 45 skip, 실행 1,018.448초, exit 0; [결과](../pr/assets/issue_8578/20261105/full-nextest.json) |
| Native Skia | root lib 3,858 PASS·12 ignore, 다른 libs 182 PASS, PNG 2 PASS, PDF 22 PASS; [Linux gates](../pr/assets/issue_8578/20261105/linux-gates.json) |
| 집중·기존 계약 | 새 관계 검사 8건 + 기존 #1892 5건 + #919 5건 = 18 PASS, Native·Linux 모두 통과; 리베이스 후에도 18 PASS |
| Studio 단위 검사 | 2,719 PASS / 0 FAIL / 2 skip; 이후 Studio production source 불변 |
| Mac 필수 게이트 | fmt·Native/WASM/all-target Clippy·workspace build·manifest·unit tiers·E2E manifest·tsc 등 13단계 exit 0; [Mac gates](../pr/assets/issue_8578/20261105/mac-gates.json) |
| 실제 브라우저 | 원본 file input→Canvas2D 383쪽. 3쪽 입력·드래그·삭제·Enter·undo/redo, 4쪽 본문 입력, Group 선택→Esc→본문 입력 통과; 관련 소유·캐럿 경고 0 |
| Native/fresh WASM 전쪽 | 각383쪽, 최저 raw 99.96%·130쪽, 90% 미만 0, 누락 0, base 대비 하락 0 |

서로 다른 실행 묶음을 고유 검사 총수로 합산하지 않는다. Linux의 시스템·사용자 글꼴은 bwrap으로
차단하고 프로젝트/fixture 프로그램 64개만 노출했다. Mac 시각 비교는 기존 Print와 같은 글꼴을
공급한 환경이다. 이를 최소 글꼴 전쪽 시각 검증이라고 표시하지 않는다.

Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
Docker 최적화 통과가 아니다. pkg/public의 실제 JS·WASM 및 브라우저 초기화 응답을 확인했다.
전체 회귀와 프로그램은 `e12096b9da` producer다. 문서 전용 rebase 후에는 집중·브라우저·Native
383쪽 출력 및 전쪽 TSV를 다시 검증했고, WASM PNG는 source·프로그램·입력·PDF 동일성을
확인해 재사용했다. [최종 검증 상세](../pr/assets/issue_8578/20261105/verification.json)와
[post-rebase checks](../pr/assets/issue_8578/20261105/post-rebase-checks.json)에 실행·재사용을 구분했다.

## 조판 원칙 준수 검토

| 검토 항목 | 실제 대조 근거 | 판정 |
| --- | --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 필수 로컬 검사 완료. 일반 Open PR 제출 후보이며 원격 승인을 대기한다. CI·merge 완료로 판정하지 않는다. | 로컬 충족; 원격 미실행 |
| 기존 이슈·해결 PR 선행 분석 | #8578 원본·한컴 편집기 기대 동작, #919 원 해결 `14920220ac`, #1892 원 PR #1905·후속 `6097253468`과 기존 검사 확인 | 충족 |
| 구현 근거와 일반성 | 원본 자식 순서의 실제 글상자 주소를 공통 helper로 공급. 장식/실제 빈 글상자·중첩 표·Header 반례 검사. 샘플 전용 분기 없음 | 충족 |
| 측정·배치 일관성 | 편집 주소와 폭은 실제 원본 자식에 연결. 캐럿은 확정 본문 원점 사용. 원래 paint·clip·문단 전진을 유지하고 전쪽 PNG 동일성 확인 | 충족 |
| 리베이스 반영 PR 기능 보존 | #8603의 검토 문서 3개 원문 보존, source/test tree 동일, 새 base focused·브라우저·Native gate 및 전쪽 TSV 확인 | 충족 |
| 최종 출력 소비 계약 | Native/fresh WASM 인쇄 출력과 실제 Studio Canvas2D 편집을 각각 확인. CanvasKit 또는 Native 직접 PNG로 화면 증적을 바꿔 표시하지 않음 | 충족 |
| 분할·이어받기 계약 | 컷·예산·이월·쪽 할당 정책을 변경하지 않음. Enter/문단 병합의 편집 내용 보존은 별도 관계 검사 | 비해당 |
| 줄 소속과 점유 높이 | 기존 저장 LineSeg·줄 높이 정책 변경 없음. 편집 후 해당 자식 폭으로 재조판하며 형제 문단 소유·내용 보존 검사 | 충족 |
| 사례·증거 독립성 | 원본 HWPX의 자식/본문 구조, 기존 독립 Print, 실제 입력 UI와 합성 Header/중첩 표 반례를 구분 | 충족 |
| 기준값 변경 | baseline·golden·90%·하락 허용폭 및 글꼴 변경 없음 | 비해당 |
| 회귀 추가 선행 조건 | Native/fresh WASM 전383쪽 최저99.96%; 3·4쪽100.00%. 새 검사는 주소·순서·보존 관계를 단언하며 실물 px 좌표를 고정하지 않음 | 충족 |
| 90% 통과 후 직접 판독 | 동일 쪽의 제목·안내 본문·목차·배경 장식·줄 배치 review/overlay 확인. 편집 화면과 미편집 인쇄를 구분 | 충족 |
| 주장과 검증 범위 | #8578 클릭/입력/Esc 오류의 해결과 기존 출력 보존. 편람 전체 피델리티 개선 또는 미실행 backend 완료를 주장하지 않음 | 충족 |

## 시각·동작 증적과 남은 차이

| 해당 쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 3쪽 제목·안내 본문 | [review](../pr/assets/issue_8578/20261105/native-p003-review.png) / [overlay](../pr/assets/issue_8578/20261105/native-p003-overlay.png) | [review](../pr/assets/issue_8578/20261105/wasm-p003-review.png) / [overlay](../pr/assets/issue_8578/20261105/wasm-p003-overlay.png) |
| 4쪽 목차·장식 | [review](../pr/assets/issue_8578/20261105/native-p004-review.png) / [overlay](../pr/assets/issue_8578/20261105/native-p004-overlay.png) | [review](../pr/assets/issue_8578/20261105/wasm-p004-review.png) / [overlay](../pr/assets/issue_8578/20261105/wasm-p004-overlay.png) |

3·4쪽은 양쪽 raw100.00%이며 직접 판독에서 새로운 내용·간격·장식 배치 결함을 확인하지 않았다.
130쪽99.96%는 base와 동일한 기존 차이다. TSV의 두 자리 표시에서 나머지382쪽은100.00%지만
반올림 전 정확히100%인 페이지 수는 Native376쪽·WASM380쪽으로 구분한다.
[Native base](../pr/assets/issue_8578/20261105/native-base.tsv) / [후보](../pr/assets/issue_8578/20261105/native-head.tsv),
[WASM base](../pr/assets/issue_8578/20261105/wasm-base.tsv) / [후보](../pr/assets/issue_8578/20261105/wasm-head.tsv)의
각383쪽 행은 모두 동일하며, [전후 비교](../pr/assets/issue_8578/20261105/tsv-before-after.json)에 기록했다.

[실제 UI 결과](../pr/assets/issue_8578/20261105/browser-result.json)는 3쪽 본문 `parentParaIndex=4,
cellIndex=1`, 4쪽 실제 본문 `paragraphIndex=8`, Esc 후 본문 `paragraphIndex=4`를 보인다.
[3쪽 실제 편집](../pr/assets/issue_8578/20261105/studio-page003-edit.png)과
[4쪽 실제 편집](../pr/assets/issue_8578/20261105/studio-page004-body.png)은 검사 중 입력 문자를 포함한다.
인쇄 raw PNG·review·overlay는 미편집 원본이며 서로의 결과를 대신하지 않는다.

## 승인 대기와 후속 조건

최종 PR 본문안은 ignored `output/issue8578/pr-body-draft.md`에 있으며, 템플릿·전체 TSV 두 경로
원문·입력/Print SHA·주장한 3·4쪽 raw/review/overlay를 포함한다. 로컬 로그·임시 driver는
ignored output에만 보존한다. 제출 증적 34개와 source hash는
[producer](../pr/assets/issue_8578/20261105/producer.json)와 재대조했다.

이번 완료 범위는 최종 로컬 검토 작성과 PR 승인 대기다. 승인 전 push·PR·comment·issue close·merge를
실행하지 않는다. 승인 후에는 최신 upstream/head와 기존 열린 PR을 재확인하고 push·PR 생성,
raw 다운로드 바이트/SHA 및 게시 본문 API를 확인한다. 이후 정확한 head CI 성공, 정식 review·
오늘할일 기록, trailing head 검증과 별도의 merge 권한 확인을 거쳐 후속 처리한다.
