# #7028 결과보고서 — 분할 표·반복 제목 셀의 대각선 복원

- Issue: #7028 — https://github.com/edwardkim/rhwp/issues/7028
- 일자: 2026-09-11 (KST), 작업 브랜치 `task_m100_7028`.
- 검증 후보: `4592264c007a4f6616f38d7fda30784f7936fc51`.
- 승인: 메인테이너 대각선 확인 및 빈 문단 문제 분리·이번 타스크 완료 절차 진행 지시.
- 상태: 메인테이너 결과보고서·push·PR 승인 후 [PR #7036](https://github.com/edwardkim/rhwp/pull/7036)
  Open 생성 완료. 제출 candidate CI 성공, 메인테이너 승인에 따라
  [self-review](../pr/archives/pr_7036_review.md) 완료. 후행 문서 head CI·병합 승인은 별도다.

## 1. 해결한 문제

일반 표에서는 호출하던 공통 대각선 생성기가 분할 표 경로에는 연결되지 않았다.
그 때문에 셀 자체가 잘리지 않았어도 표가 여러 쪽에 걸치면 첫 쪽과 반복 제목 셀의
대각선이 모두 사라졌다. 파서의 선 속성 누락이나 Studio만의 문제는 아니었다.
`v0.8.6` 소스에서도 호출 누락을 확인했으며 최초 유입 커밋까지 확정한 것은 아니다.

`src/renderer/layout/table_partial.rs`에서 해당 페이지에 온전히 표시되는 셀을 판정하고,
기존 공통 생성기를 가로쓰기·세로쓰기 두 경로에 연결했다. 표시 행의 실제 셀 좌표로
선을 한 번 생성한다. 샘플 이름이나 특정 페이지를 조건으로 사용하지 않는다.

실물 `samples/task2146/21761835_jeonjik_exemption_table.hwp`의 6쪽 모두에서
제목 셀의 대각선이 복원됐다. 메인테이너도 대각선 렌더링을 확인했다.

## 2. 변경 범위와 보호한 동작

- 제품 변경: `table_partial.rs` 한 파일. 테스트: `tests/cases/issue_7028_partial_table_diagonal.rs` 9개.
- 제목행 선택·반복 높이 예약·페이지 분할·셀 높이·텍스트 배치·파서·IR·Studio 속성 UI는 변경하지 않았다.
- native 6쪽 before/after에서 각 쪽 대각선 1개 외 나머지 내보낸 노드와 텍스트 원장은 동일했다.
- 실제 잘린 셀, 높이 override, 페이지 경계를 가로지르는 rowspan, 활성 대각선 zone과 겹치는 셀은
  승인된 범위대로 기존 출력을 유지했다. 이 유형까지 지원을 완료했다는 뜻은 아니다.
- 새 fixture·baseline·generated suite·Cargo 파생물을 제출 변경에 추가하지 않았다.

## 3. 검증 결과

| 검증 | 결과 |
| --- | --- |
| 실물·보호 반례 focused | 29개 통과: 신규 9개 + 기존 셀 높이 1개 + zone 19개 |
| 전체 release-test nextest | 9,481개 통과, 기존 skip 46개 |
| fmt·native/WASM/workspace all-target Clippy·workspace build·manifest | 모두 통과 |
| Native Skia lib | 4,112개 통과, 기존 ignore 13개 |
| Native Skia PNG / 직접 PDF | 2개 / 4개 통과 |
| 최신 devel 통합 후 Docker WASM | 최적화 포함 7분 02초, 성공 |

최신 `upstream/devel c378fbe3c7`을 먼저 병합한 뒤 전체 게이트를 실행했다. 충돌은 오늘할일의
독립 기록 추가뿐이었으며 양쪽 기록을 모두 보존했다. 제품 소스 충돌은 없었다.
위 수치는 겹치는 검증 집합을 포함하므로 합산한 고유 테스트 수로 해석하지 않는다.
상세 명령·로그·도구 경고는 [Stage 3](../working/task_m100_7028_stage3.md)에 기록했다.

### 시각·브라우저 확인 범위

native 6쪽 SVG와 표준 비교 자료는 `output/7028/stage2/fidelity/svg/` 및
`output/7028/stage2/visual/`에 보관했다. 문서 전체가 한컴과 완전히 일치한다는 판정은 아니다.

최종 WASM은 `pkg/`에 제공되어 기존 Studio dev 서버에서 사용할 수 있다.
SHA-256: `9ddc53befe22db3cba060e6a69944ea6b560b7107143cb1aea07ff67ce97c9a5`.
브라우저 자동 확인은 제공된 WASM 해시 일치·6쪽 로드·첫 쪽 대각선 1개까지 확인했다.
이후 SVG 문자열 검사의 한계로 스크립트를 보정했지만 Chrome 새 탭 응답 지연으로 재확인을
마치지 못했다. **최종 브라우저 6쪽 검증·캡처 통과 또는 메인테이너의 새 WASM 판정은 선언하지 않는다.**

## 4. 별도로 등록한 빈 문단 문제

메인테이너는 두 줄을 비우고 세 번째 줄에 `직렬`을 배치하려는 편집 의도가 반영되지 않음을
보고했다. 이를 [#7032](https://github.com/edwardkim/rhwp/issues/7032)로 등록했다.
담당 edwardkim, milestone v1.0.0, bug/layout/rendering 분류를 적용하고 API로 확인했다.

현재 확보한 HWP 원본 레코드는 빈 문단 1개와 `직렬` 문단 1개이며 저장 LINE_SEG는 없다.
따라서 사용자 설명을 근거 없이 원본에 빈 문단이 2개 있다고 바꾸어 기록하지 않았다.
빈 문단 높이를 측정하는 경로와 실제 텍스트를 배치하는 경로의 차이가 조사 단서다.
수정 전후 텍스트 원장이 같으므로 이번 대각선 변경으로 발생한 회귀는 아니다.
후속 이슈에서 편집기 표시와 원본 구조를 대조하여 해결하며 이번 PR 범위에는 넣지 않는다.

## 5. 남은 완료 절차

1. **완료**: 결과보고서·push·PR 생성 승인, `task_m100_7028` 원격 push 및 devel 대상 PR #7036 생성.
2. **완료**: assignee edwardkim, milestone v1.0.0, bug/layout/rendering 적용 및 API 재조회.
   게시된 본문 한글·이슈 참조·BOM/치환 문자 없음 확인.
3. **완료**: 제출 candidate CI 6개 workflow 성공 확인 및 self-review, 집중 회귀 9개 재통과.
4. **대기**: 후행 문서 head의 CI 확인·승인된 병합 절차 후 #7028 종료 및 로컬 후속 정리.

제출 head는 `d59e83c798d1fef800575b648a1ef71781376791`이며 전체 검증 후보 이후 차이는
`mydocs/` 문서뿐이다. 제출 직전 최신 `upstream/devel`은 `376c6b605c6be3b735bf6b8b9464fcd16b833a10`으로,
PR #7033·#7034 변경이 추가됐다. 병합 시뮬레이션 tree `cde8c4554b2f3985a81397313235e40646211e3d`는
충돌이 없고 diff 검사도 통과했다. 그 통합 tree의 전체 실행 결과를 이전 로컬 결과로 대체하지 않으며,
PR CI에서 별도로 확인한다. GitHub 작성 시점 `MERGEABLE` / `BLOCKED`는 병합 승인이나 CI 성공이 아니다.

기본 경로는 `collaborator_self_merge`, 보조는 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `rework_and_exceptions`다. diff 1,000줄 초과는 대부분 단계 문서이며,
대형 PR 규칙대로 즉시 admin merge하지 않는다. 최신 devel에서 추가한 동작 기반 회귀 지침도 확인했다.
신규 9개 검사는 실제 렌더 경로를 호출하며 수정 전 실패·보호 반례는 Stage 2에 기록했다.
PR 번호 기반 self-review 판정은 CI 이후 별도로 수행한다. 이 생성 기록은 로컬 문서 commit으로
보존하고, 현재 code CI를 중복 실행시키지 않도록 후속 review 기록과 함께 push한다.

#7028은 아직 OPEN이다. #7032 등록을 이유로 대각선 구현의 통합·CI 절차를 생략하지 않는다.
현재 브라우저 자동 검증의 미완료 항목은 후속 검토에서 명시적으로 확인한다.

## 6. 근거 문서와 용어

- [수행계획](../plans/task_m100_7028.md), [구현계획](../plans/task_m100_7028_impl.md).
- [Stage 1 조사](../working/task_m100_7028_stage1.md), [Stage 2 구현·기하 대조](../working/task_m100_7028_stage2.md).
- [Stage 3 최종 검증](../working/task_m100_7028_stage3.md).
- LINE_SEG: 저장된 줄 배치 정보. 빈 문단 자체와 동일한 개념은 아니다.
- rowspan: 셀이 차지하는 행 수. cell zone: 여러 셀에 걸쳐 적용되는 테두리·배경 영역.
- WASM (WebAssembly): Studio에서 실행하는 엔진 빌드. native는 로컬 실행 파일 경로다.
