---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7491 메인터너 보정

## 독립 기준과 입력 구분

원 실패 입력(현재 보존 경로 `tests/fixtures/issue7491/center_align_first_line_indent_missing_column.hwp`)은 전체 문서3쪽을 줄인 저장본이며 첫 문단의 `secd`는 있으나 원문에 있던 `cold`가 없다. RHWP의 `getColumnDef(0)`는 columnCount0을 반환한다. 원본 전체 문서는 `/home/tsjang/Downloads/korea_downloads/기상청/156458354_210625_보도자료_강인식 서울대 명예교수 IMO상 한국인 최초 수상자로 선정.hwp`다. 직접 raw 레코드에서 원문 첫 `cold`를 확인했다. 원 실패본의25mm PageDef와 저장 segment_width45356은 서로 일치하므로 여백 숫자를 임의 보정하지 않는다.

공식 Windows MCP client `engine:2020` PDF에서 원 축소본의1984. 첫 x는91.5104pt, 여백35mm로 바꾼 진단 사본도91.5104pt다. 단 정의1개를 복원한 사본은77.3581pt, 실제 전체 원문3쪽은77.4pt다. 단 정의 복원 사본의 Native Visual Sweep은 내용 실루엣100%이며 review를 직접 확인했다. 이 결과를 원 축소본의73.57805%가 개선된 것이라고 보고하지 않는다. 원본과 실패 증거는 보존하고 유효한 사본의 생성 절차·바뀐 메타데이터와 별도 기준 PDF를 공개한다.

## 편집 뒤 표 줄 나눔

기존 실패와 동일한 삽입을 적용한 MCP PDF에서는 `가`가 자기 줄에 있고 뒤 표는 원래 왼쪽 위치를 유지한다. RHWP의 trailing TAC table은 `flow_inline_controls`에서 제외되고, `inline_control_requires_own_line`은 셀 분할 재조판에서만 호출되어 본문 편집이 이 경계를 게시하지 못한다.

소비 경로: `reflow_line_segs` 줄 나눔 → `LineSeg.text_start` → composer의 ComposedLine.char_start → layout의 table_owner_line/inline position → 최종 Table 원점과 줄 점유 높이. 공통의 너비 부족 판정·object 줄 메트릭을 본문 끝 TAC 표에도 사용하여 현재 줄에 수용할 수 없으면 다음 줄을 게시한다. 독립 기대값은 MCP의 글자/표 줄 분리와 표의 왼쪽 원점 보존이며 특정 px는 기대값으로 고정하지 않는다. 작은 표가 같은 줄에 들어가는 경우, 글자 없는 원본 host, 저장 줄 재사용과 편집 재조판을 대조한다.

## 단계와 완료 조건

1. 원본 실패·누락 단 정의·정상 원문/복원 사본의 독립 MCP PDF 증거를 보존한다.
2. 본문 trailing TAC 표의 실제 줄 나눔/배치 소비를 보정하고 기존 실패 검사를 실행한다.
3. 유효한 입력 원본/삽입 뒤 입력과 각자 대응 MCP PDF에서 Native/fresh WASM90% 이상·같은 페이지 직접 review/overlay를 확인한다. 원 실패본은 별도 결과로 유지한다.
4. 선행 시각 증거 이후 기존 회귀의 절대 px를 상대 배치·줄/개체 소속 계약으로 바꾸고 수정 전 FAIL/수정 후 PASS를 실행한다. 기준값을 완화하거나 원 실패를 원문 일치로 바꾸지 않는다.
5. 필수 lint/build/정책·영향 회귀·fresh WASM·Studio public JS와 개별 리뷰/오늘할일을 갱신한다.

## 실행 상태

단 정의 누락의 독립 대조와 복원 사본 Native100%까지 확인했다. 본문 끝 TAC 표의 너비 부족 판정을 기존 helper와 공유하고, 들여쓰기 줄 폭과 발행 플래그가 같은 정책을 사용하도록 보정했다. 관련 검사 `issue_7490_edited_paragraph_indent`는10 PASS/200 SKIP이다. 공개 Native 삽입 API의 실제 host는 text_start0/1의 두 줄을 발행하고 표 왼쪽98.29333을 유지했다. 이는 좌표 관측이며 절대 px를 회귀 기대값으로 사용하지 않는다. 편집본 MCP 대조·fresh WASM·회귀 교정은 아직 완료 전이다. 금지된 Linux 변환 스크립트/경로는 이후 사용하지 않는다.

### 별도 소비 경로 확인

17946cb17 편집본의 Native 직접 review는87.37273%였다. 가로 원점 assertion은 통과했으나 앞 글자 누락과 표의 세로 배치 차이가 남았다. 강제 object 줄이 본체 높이만 발행해 조판의 outer-box 소유 줄 조회와 맞지 않았다. 표의 폭/높이에 바깥 여백을 포함하는 점유 메트릭을 기존 own-line 판정에서 생산하도록 추가 보정하고 재실행 중이다. 이전10 PASS와 가로 원점 관측을 시각 완료로 바꾸지 않는다.

### 90% 선행 증거와 입력 보정 공개

source `b3c933a88cdb4f7ceed3fbbf1a6bc46375d18aee`의 Native/fresh WASM을 순차 빌드·캡처했다. 복원본100.0%, 삽입본90.06833%로 네 gate PASS이며 review·standalone overlay를 직접 확인했다. 앞 `가`는 자기 줄에 한 번 보이고 뒤 표의 왼쪽/폭과 내용이 유지된다. 뒤 표 두 줄에는 작은 세로 차이가 남아 완전 일치라고 쓰지 않는다. fresh WASM 공개 편집 API의 export SHA `358a8f6bfa4aaf5728f4be1050e363fa195c0ab6966ac10a5deee68f3fbe38c0`은 Native 편집본과 같다.

원 누락 샘플의 바이트·실패 PDF와 점수는 보존했다. 작업 입력의 canonical 파일은 원문에 존재하는 단 정의를 공개 `setColumnDef`로 복원한 사본으로 교정하고, 입력 변화는 fixture README와 MCP 입력 SHA에 공개했다. 원 실패본의 여백 무시 동작을 렌더러에 문서별 예외로 복제하지 않는다. 원문 전체와 그 MCP PDF, 복원본·편집본 및 각각 대응 PDF를 모두 커밋한다.

선행 증거 뒤 기존10개 회귀의 절대 px 기대값을 없앴다. 같은 부모 문단의 표/본문 글자 소속, 원점·폭 보존, 글자→표 순서·비겹침, 표 내용 누락·중복, LineSeg0/1 경계와 공개 API의 들여쓰기 방향·undo 복원을 검사한다. 다른 문서 좌표로 허용치를 바꾸지 않고 동일 배치의 부동소수점 오차만 상대적으로 허용한다. 교정 후10 PASS. 수정 전 검출 재확인과 최종 전체 게이트는 다음 단계다.

대조군은 #7491 보정 전/후 Native SVG와 render tree 바이트가 동일하다. biz-plan p3 96.13616%, tac-img p7 98.74020%, paste-indent p1 88.42273%, SO-SUEOP 표지43.52474%다. 사용자 범위대로 마지막 두 문서는 비교 대조군이며 이 기존 차이를 이번 수정으로 해결했다고 쓰지 않는다. [중간 증거](../assets/semanticist21-20261005/pr7491-interim-evidence.json)와 개별 대표 이미지를 보존했다.

### 같은 줄에 들어가는 작은 표의 반례

공개 API로 새 문서에 짧은 본문·글자취급 작은 표·셀 텍스트를 생성하고 독립 MCP PDF를 산출했다. 같은 줄인 표의 외곽과 셀 내용은 겹쳐 있으나 표 앞 글자의 기준선이 어긋나 Native/fresh WASM85.73551%였다. 이를 회귀 fixture로 추가하지 않고 출력 원인부터 보정한다. 별도 object 줄에는 바깥 여백을 계상했지만 같은 줄 fallback의 높이 발행은 본체만 쓰는 소비 경로가 남아 있었다. 표의 전체 점유 메트릭을 helper로 공유하여 own-line 판정과 fallback LineSeg 발행이 함께 사용하게 하고 같은 MCP 입력에서 재캡처한다. 작은 표가 너비를 만족하면 강제 다음 줄은 발행하지 않아야 한다.

관계 검사의 수정 전 검출: 검증 전용 worktree에 같은 최종 검사/입력을 유지하고 own-line 추가 전 production 파일만 복원했다. 10건 중9 PASS/1 FAIL이며 원점 유지의 의미를 직접 검출했다. 실행·변경 파일 hash는 `output/pr-review/semanticist21-20261005/run-records/pr7491-relative-before.json`와 실패 요약은 `output/pr-review/semanticist21-20261005/run-records/pr7491-relative-before-summary.txt`에 있다. 공유 target을 삭제하지 않고 root source를 재빌드한 뒤 최종 결과를 채택한다.

### 작은 표의 실제 본체 높이 확인

source357ff494e의 바깥 여백 공유만으로 작은 표의 점수는 바뀌지 않았다(85.73551%). 공개 API가 발행한 표 선언 높이는282HU이나 실제 셀 글자와 패딩을 소비한 `HeightMeasurer::measure_section`의 표 높이는1282HU다. 선언282+바깥 여백566은 기존 글자 줄1000보다 작아 줄 높이 갱신이 발생하지 않았다. 따라서 선언 높이만 재사용한 설명은 불충분하다.

다음 보정은 별도의 추정식을 만들지 않고 표 배치가 소비하는 `measure_table`의 셀/행 측정 결과를 편집 재조판의 점유 높이에도 사용한다. 단순 선언/여백 보정과 실제 콘텐츠 높이 측정을 구분하며, 저장 줄 재사용 경로는 기존 입력으로 재검증한다. 작은 표의 Native/fresh WASM 및 독립 MCP PDF가90%를 충족한 뒤에만 새 정식 회귀/fixture를 추가한다. 실행 로그와 진단 출력은 ignored `output/pr-review/semanticist21-20261005/`에만 보존한다.
