# semanticist21의 표 편집·주석·도형 PR 5건 검토

2026년 11월 23일 메인테이너 검토. 기준 devel은 `93e861523623bcd84d57b808edb62af5af2aa82a`이다. 아래 다섯 원 head를 순서대로 병합했다. 네 통합 테스트를 정리한 커밋은 `0af8e7e5db`, 중첩 표의 실제 포인터 검사를 보완한 커밋은 `326c0e348d`다.

| 원 PR | 보존한 head | 대상과 독립 대조 |
| --- | --- | --- |
| [#9284](https://github.com/edwardkim/rhwp/pull/9284) | `eeff0619f5` | 3×3 표의 가운데 행에서 이웃 칸 너비를 3,000 HWPUNIT만큼 반대로 변경했다. 기준은 열 3개를 유지한다. 기여 코드는 너비를 합산한 경계로 격자를 재구성한다. 중첩 표 개체는 바깥 표의 칸 경로를 제공한다. |
| [#9310](https://github.com/edwardkim/rhwp/pull/9310) | `67395e38c0` | 표 셀과 글상자에 넣은 주석의 번호·커서·소유 경로 및 문서 끝의 실제 글을 검사한다. 공통 미주 수집기와 가상 문단 조회가 같은 순서로 원래 문단에 연결되는지 대조한다. |
| [#9313](https://github.com/edwardkim/rhwp/pull/9313) | `963407d677` | 사각형의 그림자 외곽과 본체 위치를 한컴 2024 인쇄본에 대조한다. 레이어 변환과 외곽 계산은 같은 그림자 변환 행렬을 소비한다. |
| [#9315](https://github.com/edwardkim/rhwp/pull/9315) | `5a463d48a7` | 떠 있던 높이 6,000 HWPUNIT의 도형을 글자처럼 취급한 뒤 도형이 속한 줄과 뒤 문단을 대조한다. 기준은 원래 줄 높이 1,000을 HWPX에 다시 쓴다. |
| [#9323](https://github.com/edwardkim/rhwp/pull/9323) | `7c505b07e4` | 가로 병합 셀만 있는 열의 너비를 인접한 셀 제약으로 구한다. 기준의 새 행 가운데 열 너비는 1,800·1,800이고 원 서식의 열 너비는 9,269·6,226 HWPUNIT이다. 행·열 삽입과 저장 재열기를 대조한다. |

## 통합 보완

#9310과 #9313의 기존 CI는 라이브러리 내부 테스트 수가 증가해 실패했다. 각주·미주 검사 한 개와 그림자 검사 세 개를 자동 수집하는 통합 테스트로 옮겼다. 해당 판정기를 devel 대비로 실행해 내부 테스트 4,092개, 회귀 소스 2,481개와 suite 28개를 확인했다. 두 판정기의 자체 테스트 36개도 통과했다. [#9310의 원 Lint 작업](https://github.com/edwardkim/rhwp/actions/runs/45042231478/job/137269131371) · [#9313의 원 Lint 작업](https://github.com/edwardkim/rhwp/actions/runs/44991383066/job/137113040611)

중첩 표에 실제 마우스 입력을 보내는 Studio 검사 두 곳에서 첫 파일의 글꼴 선택창과 화면 클릭 좌표를 보완했다. 표 편집 명령과 테스트의 개체 소유·Undo 검사는 원래 경로를 사용한다.

공개 대조군은 중첩 표·한 행 경계 이동·병합 표·인라인 도형·셀 주석·글상자 미주의 HWP/HWPX와 그림자 HWPX다. 한컴에서 각 파일을 직접 PDF로 인쇄하고 원본과 PDF의 SHA-256을 함께 기록했다. [검증 입력 17개와 한컴 Print PDF](assets/semanticist21-20261123-table-notes-shadows/hancom-print.tsv)

원 PR 다섯 건의 GitHub 종료 연결 이슈는 0개다. #9310 본문에서 선행 사례로 언급된 #7860은 접수 당시 이미 닫혀 있었다.

## 편집에서 배치까지 확인한 경로

| 변경 | 생산한 값과 실제 소비 지점 |
| --- | --- |
| 중첩 표와 한 행의 셀 경계 | 표 노드의 경로에서 마지막 표 자체를 제외한 부모 경로를 Studio 개체 목록으로 보낸다. 셀 경계 편집은 행의 셀 너비를 바꾼 뒤 `regrid_columns_from_widths`로 열 번호와 병합 범위를 다시 계산한다. 중첩 표 포인터 검사에서 안쪽 표와 바깥 표의 크기를 각각 확인했다. |
| 표 셀·글상자의 미주 | `paragraph_endnote_refs_with_storage`가 실제 문서 순서로 소유 경로를 수집한다. `resolve_endnote_at`은 그 경로로 원문을 찾고 `get_endnote_source_at_native`는 편집기에 같은 가상 문단의 소유자를 돌려준다. 인쇄본에 두 번째 미주가 실제로 표시되는지도 비교했다. |
| 도형 그림자 | 프리셋, 도형의 너비·높이와 오프셋을 공통 `shadow::matrix`에 전달한다. 조판은 `body_origin_hu`로 본체 원점을 구하고 paint는 같은 변환으로 레이어와 그 경계를 계산한다. |
| 인라인 도형 | 도형 속성을 바꾸기 전 문단의 저장 끝 위치를 얻는다. 글자처럼 취급하기를 켜거나 끄면 `reflow_body_inline_object_change`가 저장 줄을 다시 구성하고, 재조판·쪽 나눔과 HWPX 저장이 그 결과를 사용한다. |
| 병합 표의 열 너비 | `get_column_widths`는 병합 셀 안에서 너비를 모르는 열이 하나일 때 셀 간격과 이미 아는 열 너비를 뺀다. 행·열 삽입이 이 열 너비를 사용한다. 여러 열의 너비가 아직 결정되지 않은 경우에는 기존 기본값 처리를 유지한다. |

## 원본에서 확인한 수정 전후 차이

#9310의 글상자 미주 문서를 한컴 2020으로 인쇄하면 본문 미주 1번 뒤에 2번과 ‘상자미주’가 표시된다. 같은 HWPX를 연 devel에서는 두 번째 항목이 없었다. 다섯 건을 통합한 Native에서 본문 끝의 번호와 글이 복원돼 raw가 91.41%에서 100.00%로 올랐다. 표 셀 주석의 HWP는 1쪽의 뒤 문단 글자 높이에 작은 차이가 있어 99.65%였고, 2쪽은 100.00%였다. 두 쪽 모두 devel과 5건 후보의 PNG SHA-256이 같았다. [원본·devel·후보 직접 비교](assets/semanticist21-20261123-table-notes-shadows/textbox-note-comparison.png) · [쪽별 raw와 PNG 해시](assets/semanticist21-20261123-table-notes-shadows/devel-notes.tsv)

## 공개 편집 API와 저장 재열기

같은 도구로 기준 devel과 후보의 WASM을 각각 새로 빌드했다. 후보의 `HwpDocument`는 기준의 이름 있는 메서드 609개를 모두 제공한다. 표 셀·글상자의 각주·미주를 입력하거나 번호, 커서와 소유 경로를 읽는 메서드 다섯 개가 추가됐다. Studio의 공개 JavaScript와 TypeScript 선언을 이 빌드에서 생성했다. 별도 TypeScript 소비자가 다섯 메서드를 엄격 모드로 읽는 검사도 통과했다. [양쪽 패키지·공개 API 대조](assets/semanticist21-20261123-table-notes-shadows/public-api.json)

후보에서 표·도형·주석을 편집해 저장한 HWP 8개와 HWPX 8개를 다시 열었다. 파일 16개 모두 한컴 Print에 사용한 원본과 SHA-256이 일치했다. 인라인 도형의 본문과 뒤 문단은 원래 순서를 유지했고, HWPX에 저장된 줄 높이는 각각 6,000·1,000 HWPUNIT였다. 글상자의 미주 2번을 조회하면 본문 미주 1번과 구별되는 소유 경로를 반환했다. [양 형식 재열기와 값 비교](assets/semanticist21-20261123-table-notes-shadows/roundtrips.json)

## 독립 인쇄와 Studio 편집

공개 입력 17개의 전체 19쪽을 Native와 최적화한 fresh WASM에서 96 DPI로 그렸다. 한컴 Print 대비 보정 전 2px 실루엣의 전쪽 최저값은 Native 99.65%, WASM 99.68%였다. 두 경로 모두 글상자의 미주와 뒤 문단, 이동한 행의 경계와 중첩 표의 위치가 인쇄본과 일치했다. 그림자 대조군은 양쪽 모두 99.72%였고, 확대해 본체를 기준으로 그림자의 축소·확대·기울기 방향을 확인했다. [전쪽 측정값](assets/semanticist21-20261123-table-notes-shadows/focus-pages.tsv) · [양쪽 프로그램 해시](assets/semanticist21-20261123-table-notes-shadows/focus-results.json) · [인쇄본·Native·WASM 비교](assets/semanticist21-20261123-table-notes-shadows/hancom-native-wasm.png)

Studio 검사 2,832건이 통과했고 기존 2건은 건너뛰었다. 배포 빌드와 실제 Chrome에서 중첩 표 선택·크기 조절·Undo, 행 경계 드래그, 각주 삽입·단축키 등 E2E 다섯 개를 통과했다. [실행 결과](assets/semanticist21-20261123-table-notes-shadows/studio-checks.json)

## 보호 문서 23종

Native와 fresh WASM에서 각각 2,023쪽을 다시 그렸다. 공식 하한 4,046행은 모두 같은 값으로 재현됐고, 누락된 쪽은 0개였다. 편람 HWPX 383쪽, HWP3 원본 763쪽, 시행령 74쪽도 전쪽을 비교했다. 46개 실행 조건에서 원본과 독립 PDF의 해시, 순서가 있는 글꼴 파일·해시 5,811행을 확인했다. [보호 전쪽 TSV](assets/semanticist21-20261123-table-notes-shadows/protected-pages.tsv) · [양쪽 렌더러의 측정 결과](assets/semanticist21-20261123-table-notes-shadows/protected-results.json) · [입력 조건](assets/semanticist21-20261123-table-notes-shadows/protected-inputs.json) · [글꼴 공급 순서](assets/semanticist21-20261123-table-notes-shadows/protected-font-programs.tsv)

## 새 표본의 용지 경계와 쪽수

열을 하나 추가한 표는 저장 너비가 55,754 HWPUNIT다. 이 HWP와 HWPX를 같은 조건으로 연 기준 devel과 후보 모두 표 오른쪽이 용지 밖으로 67.08px 나왔다. 독립 한컴 PDF의 실제 괘선 끝은 645.37pt이고 용지 오른쪽은 595pt다. ‘용역명’, ‘규격 및 세부사항’, ‘금액’, ‘대상’, ‘합계’는 양쪽 형식 모두 용지 안에 인쇄됐다. 두 원본의 Native·WASM 실루엣도 각각 100.00%였다. 이 두 파일에만 용지 밖 표 1개씩을 처음 등록했다. 원본 파일과 PDF의 SHA-256, 두 실행의 좌표와 동일한 내용은 비교 기록에 보존했다. [표본별 비교 근거](assets/semanticist21-20261123-table-notes-shadows/fixture-checks.json)

정본 쪽수 생성기로 입력 17개와 해당 한컴 인쇄물을 대조해 19쪽을 등록했다. 그림자 HWPX는 저장 제품 정보가 없어 생성기의 기본 엔진인 한컴 2020으로도 인쇄했다. 2024 인쇄와 96 DPI 렌더링 픽셀이 같았고, 2020 호환 조건의 Native·WASM 실루엣은 각각 99.86%였다. [그림자 한컴 2020 Print](../../pdf/semanticist21-20261123/shadow-presets-hwpx-2020.pdf)

추가된 17개 경로를 보안 코퍼스 검사에 직접 전달했다. 같은 원본을 공개 CLI에서도 확인해 숨은 글자·인젝션·유니코드 검사 51건을 통과했다. 이들 17개 문서의 표 세로 클리핑은 devel과 후보 모두 0건이었다. 기존 클리핑 대조군 92개와 이번 문서의 교집합은 0개다. 용지 경계·정본 쪽수·신규 문서 보안에 대한 집중 검사 19개도 통과했다. [신규 문서별 진단](assets/semanticist21-20261123-table-notes-shadows/fixture-checks.json)
