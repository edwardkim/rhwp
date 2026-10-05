---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7491 메인터너 보정

## 독립 기준과 입력 구분

원 실패 입력 `samples/issue6190/center_align_first_line_indent.hwp`은 전체 문서3쪽을 줄인 저장본이며 첫 문단의 `secd`는 있으나 원문에 있던 `cold`가 없다. RHWP의 `getColumnDef(0)`는 columnCount0을 반환한다. 원본 전체 문서는 `/home/tsjang/Downloads/korea_downloads/기상청/156458354_210625_보도자료_강인식 서울대 명예교수 IMO상 한국인 최초 수상자로 선정.hwp`다. 직접 raw 레코드에서 원문 첫 `cold`를 확인했다. 원 실패본의25mm PageDef와 저장 segment_width45356은 서로 일치하므로 여백 숫자를 임의 보정하지 않는다.

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
