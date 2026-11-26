# PR #9306 리뷰 — 중첩 표의 실제 컷과 저장 프레임

## 최종 판정

**로컬 검증과 코드 head CI 통과.** [#7114](https://github.com/edwardkim/rhwp/issues/7114)의 120자 편집은 HWP·HWPX의 편집, 두 차례 저장·재열기에서 115쪽과 102·103쪽의 74자·46자 소속을 유지했다. 승인한 보호표의 4,046행이 모두 통과했다.

| 항목 | 확인 결과 |
| --- | --- |
| PR·제품 코드 | [#9306](https://github.com/edwardkim/rhwp/pull/9306), `b0f7b0c60c80d8a020d3c15ccbabdfc013b9f5fa` |
| 검토한 코드 head | `f3efcf7fc8d0164a4dc941c68edd5a946cc1bba3`, 대상 `devel`, collaborator 셀프 리뷰 |
| 코드 head CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/45864409097) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/45864401721) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45864408676) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45864408614) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45864408730) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45864408512) 통과 |
| 기록 head | 기록 커밋의 필수 검사 확인 뒤 병합한다. |

## 원 해결 범위와 남은 저장 경계

[#7404](https://github.com/edwardkim/rhwp/pull/7404), [#7568](https://github.com/edwardkim/rhwp/pull/7568), [#8148](https://github.com/edwardkim/rhwp/pull/8148)이 처리한 문단 5, 오프셋 130의 HWP 56자 편집은 115쪽을 유지했다. 원본 `samples/issue1949_giant_cell_nested_tables_perf.hwp`의 셀 경로 `(0,0,2,2,2276,0)`에 `1`을 120자 삽입하면 `c198dc3ef1` WASM과 변경 전 Native의 최초 그리기 전 저장에서 115쪽 화면이 116쪽 저장본으로 바뀌었다. 페이지 수 외에도 14행 자식 표의 이어받기와 뒤따르는 두 삽화의 소속을 독립 한컴 Print 및 한컴 재저장 줄 레코드로 확인했다.

[#8958](https://github.com/edwardkim/rhwp/pull/8958)은 셀별 자식 행 컷을 도입했고, [#9292](https://github.com/edwardkim/rhwp/pull/9292)에 포함된 [#9273](https://github.com/edwardkim/rhwp/pull/9273)은 직접 편집한 자식 셀의 줄 프레임을 썼다. 남은 반례에서는 부모 호스트를 편집해 뒤따르는 자식의 컷이 옮겨졌다. 저장기는 초기화 상태가 그리기 순서에 따라 달라지는 레이아웃 엔진으로 컷의 소속을 다시 읽었다. 부모 줄이 가리킬 수 없는 컷을 옛 자식 프레임에 남겨, 재열기에서 같은 경계를 재현하지 못했다.

| 검토 항목 | 구현과 관측 결과 |
| --- | --- |
| 적용 범위와 원본 보존 | 직접 편집한 단일 전폭·비병합 셀의 유효한 편집 전 저장 줄을 보존한다. 영향 구간의 자식 원본 줄 유효성, 행 분할 속성, 셀 소유권을 확인한다. 무편집, 짧은 HWP 편집, 부모 줄에 소속을 기록할 수 없는 기존 중첩 컷 대조군을 유지했다. |
| 측정·배치·저장 | `host_lines_following_projected_nested_table`로 표 앞뒤 글줄의 소속을 구하고, `cell_units`가 표와 위아래 여백이 차지하는 공간을 컷에 예약한다. `row_block_cut`은 그림의 전체 줄 상자가 남은 예산 안에 들어오는지 판정한다. `table_partial`은 같은 컷과 흐름 높이로 표를 앵커 줄 앞에 배치하고, 선택된 줄의 텍스트와 그림을 그린다. |
| 실제 컷의 저장 | `writeback_reflowed_table_frames_with_storage`가 프로필, 단·탭, 정규화 오버레이와 개시 배치 정보를 설정한 엔진으로 `reflowed_nested_source_frames_with_storage`를 호출한다. 첫 물리 조각의 상자 높이와 컷을 여는 자식 셀의 `LINE_SEG`만 저장한다. 기존 0-based 행 9의 시작은 해제하고 실제 행 8의 이어받기 줄을 0 원점으로 저장한다. 뒤 줄과 원점이 같아도 페이지 시작 플래그로 구분한다. |
| 형식과 출력 경로 | HWP와 HWPX는 자식 컷의 공통 저장 결과를 쓴다. HWP의 부모 저장 줄 되쓰기는 기존 형식별 경로에서 수행한다. Native와 fresh WASM은 102쪽 74자, 103쪽 8행 표 뒤 46자, 103·104쪽 터보 과급기 행과 104·105쪽 삽화 소속을 각각 보존했다. 실제 Studio 키 입력과 저장·인쇄 메뉴에서 지연 조판을 완료하고 115쪽으로 열었다. |

## 회귀와 독립 시각 증거

한컴 2024 `13.0.0.3901`에서 검토 제품이 내보낸 각 형식을 열고 PrintMethod 0으로 인쇄했다. PDF·출력 파일의 해시, 전체 115쪽 TSV와 확대 비교는 [이슈의 검증 증거](../assets/issue_7114/README.md)에 있다. Native Skia와 최적화 fresh WASM Canvas의 96 DPI 전쪽 최저 raw는 두 형식에서 각각 98.46%와 99.31%였다. 쪽수는 독립 Print가 정한 115쪽을 사용한다. 새 회귀는 표·숫자 줄·같은 행의 이어받기·그림·뒤 제목의 쪽 소속과 상대 순서를 검사한다.

편집 전후와 저장·재열기·재저장에서 115쪽을 유지했고, 같은 형식의 반복 저장은 바이트가 같았다. HWP↔HWPX 교차 저장도 115쪽을 유지했다. 대상 편집의 Native HWPX 화면은 103쪽 본문 시작의 세로 위치에 저장 재열기 대비 약 4.5px 차이가 있었고, 확대 판독에서 순서와 소속은 같았다. 별도의 문단 5, 오프셋 130, 61자 HWPX 대조군은 기준 Native와 후보 양쪽에서 116쪽으로 다시 열렸다.

로컬 기본 회귀 13,938건과 Native Skia 공식 세 검사 묶음, Native/WASM 및 workspace Clippy, 포맷과 workspace 빌드가 통과했다. 편람 HWPX Native 383쪽 제출 게이트와 공통 보호 문서 4종의 Native/WASM 8개 조건도 통과했다.

## 초기 통합용 정본 보호 검토

[정본 4,046행](../assets/visual-protected-baseline/final.tsv)의 입력·Print PDF 해시를 확인했다. #9292의 화학물질 표시 기준과 형식 사양서에 기록된 58개 공급 프로그램 중 41개가 현재 공급본과 SHA-256으로 일치했다. 남은 공급 및 정본 미달은 다음 결과로 확인했다.

| 문서 | Native 정본 하락 | WASM 정본 하락 | 같은 조건의 기준 비교 |
| --- | --- | --- | --- |
| 화학물질 표시 기준, 103쪽 | 58쪽, 44쪽 100.00% → 99.58% | 37쪽, 89쪽 100.00% → 99.56% | 기준과 후보 PNG 103/103쪽씩, TSV 일치 |
| 형식 사양서 rev.1.3, 71쪽 | 2쪽 100.00% → 99.84% | 2쪽 100.00% → 99.94% | 기준과 후보 PNG 71/71쪽씩, TSV 일치 |
| 편람 HWP, 383쪽 | 37쪽, 277쪽 100.00% → 93.17% | 0쪽 | 하락 37쪽의 Native PNG가 기준과 후보에서 일치 |
| 영월 서식, 4쪽 | 1쪽 100.00% → 99.98% | 0쪽 | Native PNG 4/4쪽 일치 |

비교한 기준 WASM은 PR을 열 때의 `99a81bd7f03535d16c8241f7e45c554cc6839af7`에서 최적화해 빌드했다. Native는 `0adeca439f1481e7245a344b7713d7d563df2ace`에서 빌드한 실행 파일을 사용했다. 그 뒤 위 기준까지의 Rust 차이는 WASM 전용 Studio 글꼴 조회 API이며 Native 생산 경로에 영향이 없다. 형식 사양서 2쪽의 독립 PDF에서 한컴돋움과 HY견고딕의 임베디드 face를 확인했다. 정본 공급 목록에는 이 두 글꼴의 로컬 미확보 해시가 있다. 화학물질 표시 기준의 기호 전용 원본 두 프로그램과 일부 Windows 글꼴도 공급 해시가 일치하는 파일을 확인하지 못했다. 전체 정본 판정에는 실제 선택 face·공급 환경을 맞춘 재측정이 필요하다. 공식 하한이나 기존 기대값을 변경하지 않았다.

두 목차 보호 문서의 본문 글꼴은 독립 PDF의 함초롬바탕이다. 기존 증적과 해시가 같은 글꼴로 재측정해 두 문서의 Native·WASM 총 8쪽에서 모두 100.00%를 확인했다. 시행령은 정본의 양쪽 환경 JSON을 확보했다. WASM에서 한양중고딕 TTF와 Palatino의 실제 등록 해시를 확인했고, 74쪽 중 Native 47쪽과 WASM 7쪽이 정본보다 낮았다. 최저는 각각 99.76%와 99.89%였다. 편람 HWPX는 저장소 표준 Native 게이트의 383쪽이 정본과 모두 일치했다. 11월 15일 공급본만 연결한 별도 Native 실행에서는 37쪽이 낮았으며, 공급 환경에 따라 결과가 달라졌다. 보호 문서의 별도 17쪽 중첩 표는 기준·후보의 Native와 WASM PNG가 각각 17/17쪽 일치했다. 정본보다 낮은 쪽은 Native 7쪽과 WASM 3쪽이었다.

HWP 3.0 `sample10`의 Native 정본 환경 JSON을 확인했다. 문서에 명시된 22개 선택 face 중 19개를 프로그램 바이트와 face index로 복원했다. Courier New, Times New Roman, 바탕체의 정본 face 식별값은 현재 공급본에서 확인하지 못했다. 이 환경으로 다시 측정한 집중 15쪽의 Native PNG는 기준과 후보가 같았다. 608쪽은 공급 조건을 맞추면서 80.41%에서 100.00%로 회복했고 690쪽은 정본 99.94%에 대해 양쪽 모두 99.92%였다.

## 최신 devel과 함께 적용할 때의 범위

생성 전 기준에는 [#9302](https://github.com/edwardkim/rhwp/pull/9302)가 포함됐다. 병합 시뮬레이션의 Studio에서 실제 120자 입력과 파일 메뉴의 HWP/HWPX 저장·다시 열기·재저장을 실행해 115쪽을 확인했다. PR을 연 뒤 [#9304](https://github.com/edwardkim/rhwp/pull/9304)가 반영된 `ae60986a8d6bbec3d182c0548efe4fb293d31742`를 기준으로 병합 트리 `93d93287608246e2cac8e0aced13ded6a122c0a4`를 확인했다. #9304는 그림 헤더와 구성 요소의 `CTRL_DATA` 소유권을 고쳤다. 두 PR의 생산 소스 변경 경로 교집합은 비어 있다. 이후 [#9300](https://github.com/edwardkim/rhwp/pull/9300)이 반영한 `1a27faacff48ea7329e76ce8eca15aa9ca8aa877`까지의 추가 차이는 Render Diff 증적과 CI 요약 게시다.

같은 작성자의 공개 작업 #9300, #9303은 이 PR 생성 전 병합 시뮬레이션에서 파일이 겹치지 않았다. #9303의 #7910은 머리말·꼬리말과 본문 유동 배치 파일을 다룬다. 별도 #7407 작업은 문단 줄 나눔의 `src/renderer/composer/line_breaking.rs`를 다룬다.

## 11월 27일 최종 검증

최신 devel `ba4f9b8d502472de71068711276d039a425892b0`에서 Native 집중 검사 6건 중 HWP 102쪽의 삽입 글줄 소속 검사 1건이 실패했다. fresh WASM에서 HWP의 120자 입력은 첫 재열기에서 115쪽에서 116쪽이 됐다. 후보에서 Native 6건이 모두 통과했다. Node와 Chrome의 fresh WASM에서 두 형식의 첫 편집과 두 차례 재열기 모두 115쪽이었고, 102·103쪽 74자·46자와 104쪽 중복 0자를 확인했다. 문단 5·오프셋 130의 별도 HWPX 55·56·61·200자 입력은 기준과 후보 양쪽에서 재열기 116쪽이었다.

같은 Mac 공급에서 sample16 384쪽의 devel·후보 PNG 해시가 모두 같았다. 사용자 승인에 따라 HWP 2022 Native의 수식 6쪽은 99.96%, 다른 다섯 조건의 6쪽은 99.97%를 적용했다. 편람 HWP·HWPX WASM 200쪽은 함초롬바탕 공급으로 각각 100.00%가 됐다. sample10은 Native·WASM 각 763쪽을 통과했다. 게이트 단위 검사 30건이 통과했다. [전쪽 보호 결과](../assets/issue_7114/protected-mac-20261127/README.md) · [하한 조정 근거](../assets/issue_7114/sample16-mac-protection-20261127/README.md)

## 병합 후 기록

확정된 병합 SHA와 코드·기록 head CI를 #9306과 #7114에 연결한다. 120자 입력의 쪽수와 102·103쪽 소속, 보호 4,046행 판정, 같은 동작을 보인 별도 HWPX 대조군을 기록한다. HWP 103쪽의 대표 이미지는 원 PR의 Native 확대 증적을 사용한다. [HWP 103쪽](../assets/issue_7114/native-hwp-p103.png) · [수식 6쪽 확대](../assets/issue_7114/sample16-mac-protection-20261127/sample16-hwpx-p6-equation-detail.png)
