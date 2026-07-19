---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# webhwp 분석: 툴바 및 메뉴 시스템

> 분석 대상: `webhwp/js/hwpApp.*.chunk.js` 외 7개 JS/CSS 파일
> 분석 일자: 2026-02-09

## 1. 파일 구조

| 파일 | 크기 | 역할 |
|------|------|------|
| `js/hwpApp.*.chunk.js` | 5.0MB | 메인 앱 번들 (Actor/Updater 정의) |
| `js/root.*.chunk.js` | 196KB | 메뉴/UI 정의 |
| `js/nls12.*.chunk.js` | 327KB | 다국어 문자열 (툴팁, 메뉴 라벨) |
| `js/360.*.chunk.js` | 564KB | 이벤트 핸들링, data 속성 바인딩 |
| `js/138.*.chunk.js` | 827KB | UI 컴포넌트 |
| `commonFrame/skins/default/css/hcwo.css` | 379KB | UI 스타일링 |
| `js/webhwpapp/style/webhwp.css` | — | 앱 스타일 |

## 2. 메인 메뉴 구조

8개 최상위 메뉴로 구성되며, `menu_*` / `main_menu_*` 두 가지 접두사로 정의된다.

### 2.1 파일 (File)

```
menu_file_save                  저장
menu_file_save_as               다른 이름으로 저장
menu_file_download              다운로드
menu_file_download_as_pdf       PDF 다운로드
menu_file_print                 인쇄
menu_file_properties            문서 속성
menu_file_rename                이름 바꾸기
menu_file_page_setup            쪽 설정
menu_file_new_presentation      새 프레젠테이션
```

### 2.2 편집 (Edit)

```
menu_edit_undo                  실행 취소
menu_edit_redo                  다시 실행
menu_edit_cut                   잘라내기
menu_edit_copy                  복사
menu_edit_paste                 붙여넣기
menu_edit_select_all            모두 선택
menu_edit_find_and_replace      찾기 및 바꾸기
```

### 2.3 보기 (View)

```
menu_view_fit                   맞춤
menu_view_sidebar               사이드바
menu_view_slide_show            슬라이드 쇼
menu_view_slide_show_from_current_slide   현재 슬라이드부터
menu_view_show_slide_note       슬라이드 노트 표시
menu_view_hide_slide_note       슬라이드 노트 숨기기
```

### 2.4 삽입 (Insert)

```
menu_insert_table               표 삽입
menu_insert_image               그림 삽입
menu_insert_shape               도형 삽입
menu_insert_textbox             텍스트 상자
menu_insert_hyperlink           하이퍼링크
```

### 2.5 서식 (Format)

```
menu_format_bold                굵게
menu_format_italic              기울임
menu_format_underline           밑줄
menu_format_strikethrough       취소선
menu_format_superscript         위 첨자
menu_format_subscript           아래 첨자
menu_format_indent              들여쓰기
menu_format_outdent             내어쓰기
menu_format_alignment           정렬
menu_format_alignment_left      왼쪽 정렬
menu_format_alignment_middle    가운데 정렬
menu_format_alignment_right     오른쪽 정렬
menu_format_alignment_justified 양쪽 정렬
menu_format_vertical_alignment          수직 정렬
menu_format_vertical_alignment_top      위쪽
menu_format_vertical_alignment_middle   가운데
menu_format_vertical_alignment_bottom   아래쪽
menu_format_autofit                     자동 맞춤
menu_format_autofit_do_not_autofit      자동 맞춤 안 함
menu_format_autofit_resize_shape_to_fit_text   도형 크기 텍스트에 맞춤
menu_format_autofit_shrink_text_on_overflow    텍스트 축소
menu_format_wrap_text_in_shape          도형 내 텍스트 줄 바꿈
```

### 2.6 표 (Table)

```
menu_table_create_table         표 만들기
menu_table_add_row_above        위에 행 추가
menu_table_add_row_below        아래에 행 추가
menu_table_add_column_to_left   왼쪽에 열 추가
menu_table_add_column_to_right  오른쪽에 열 추가
menu_table_delete_row           행 삭제
menu_table_delete_column        열 삭제
menu_table_merge_cells          셀 병합
menu_table_unmerge_cells        셀 분할
```

### 2.7 배치 (Arrange)

```
menu_arrange_align_horizontally         수평 정렬
menu_arrange_align_horizontally_left    왼쪽
menu_arrange_align_horizontally_center  가운데
menu_arrange_align_horizontally_right   오른쪽
menu_arrange_align_vertically           수직 정렬
menu_arrange_align_vertically_top       위쪽
menu_arrange_align_vertically_middle    가운데
menu_arrange_align_vertically_bottom    아래쪽
menu_arrange_order                      순서
menu_arrange_order_bring_to_front       맨 앞으로
menu_arrange_order_bring_forward        앞으로
menu_arrange_order_send_backward        뒤로
menu_arrange_order_send_to_back         맨 뒤로
menu_arrange_group                      그룹
menu_arrange_ungroup                    그룹 해제
```

### 2.8 슬라이드 (Slide)

```
menu_slide_new                  새 슬라이드
menu_slide_duplicate            복제
menu_slide_delete               삭제
menu_slide_hide                 숨기기
menu_slide_show_slide           슬라이드 보기
menu_slide_first_slide          첫 슬라이드
menu_slide_previous_slide       이전 슬라이드
menu_slide_next_slide           다음 슬라이드
menu_slide_last_slide           마지막 슬라이드
```

## 3. 빠른 도구 모음 (Quick Access Toolbar)

```
toolbar_save                    저장
toolbar_undo                    실행 취소
toolbar_redo                    다시 실행
toolbar_print                   인쇄
toolbar_find_and_replace        찾기 및 바꾸기
toolbar_insert_table            표 삽입
toolbar_insert_image            그림 삽입
toolbar_insert_shape            도형 삽입
toolbar_insert_textbox          텍스트 상자
toolbar_insert_hyperlink        하이퍼링크 삽입
toolbar_update_hyperlink        하이퍼링크 수정
toolbar_read_only               읽기 전용
toolbar_exit                    나가기
toolbar_help                    도움말
toolbar_main_menu_open          메인 메뉴 열기
toolbar_main_menu_close         메인 메뉴 닫기
```

## 4. 툴팁 문자열 (nls12 로케일)

### 4.1 텍스트 서식

| 툴팁 키 | 설명 |
|---------|------|
| `ToolTipBold` | 굵게 |
| `ToolTipItalic` | 기울임 |
| `ToolTipUnderline` | 밑줄 |
| `ToolTipStrikethrough` | 취소선 |
| `ToolTipTextColor` | 글자 색 |
| `ToolTipBackgroundColor` | 배경 색 |
| `ToolTipClearFormat` | 서식 지우기 |

### 4.2 정렬

| 툴팁 키 | 설명 |
|---------|------|
| `ToolTipAlignLeft` | 왼쪽 정렬 |
| `ToolTipAlignCenter` | 가운데 정렬 |
| `ToolTipAlignRight` | 오른쪽 정렬 |
| `ToolTipTop` | 위쪽 정렬 |
| `ToolTipMiddle` | 가운데 정렬 (수직) |
| `ToolTipBottom` | 아래쪽 정렬 |

### 4.3 기본 동작

| 툴팁 키 | 설명 |
|---------|------|
| `ToolTipUndo` | 실행 취소 |
| `ToolTipRedo` | 다시 실행 |
| `ToolTipSave` | 저장 |
| `ToolTipDownload` | 다운로드 |
| `ToolTipExit` | 나가기 |
| `ToolTipViewMainMenu` | 메인 메뉴 보기 |
| `ToolTipFindReplace` | 찾기 및 바꾸기 |

### 4.4 삽입

| 툴팁 키 | 설명 |
|---------|------|
| `ToolTipImage` | 그림 |
| `ToolTipChart` | 차트 |
| `ToolTipFilter` | 필터 |
| `ToolTipFunction` | 함수 |
| `ToolTipSymbol` | 기호 |
| `ToolTipHyperlink` | 하이퍼링크 |

### 4.5 표/셀

| 툴팁 키 | 설명 |
|---------|------|
| `ToolTipMergeCells` | 셀 병합 |
| `ToolTipUnmergeCells` | 셀 분할 |
| `ToolTipWrapText` | 텍스트 줄 바꿈 |
| `ToolTipInsertRow` | 행 삽입 |
| `ToolTipInsertColumn` | 열 삽입 |
| `ToolTipDeleteRow` | 행 삭제 |
| `ToolTipDeleteColumn` | 열 삭제 |
| `ToolTipFreezeUnfreezePanes` | 틀 고정/해제 |

### 4.6 테두리

| 툴팁 키 | 설명 |
|---------|------|
| `ToolTipOuterBorders` | 바깥쪽 테두리 |
| `ToolTipAllBorders` | 모든 테두리 |
| `ToolTipInnerBorders` | 안쪽 테두리 |
| `ToolTipTopBorders` | 위쪽 테두리 |
| `ToolTipHorizontalBorders` | 가로 테두리 |
| `ToolTipBootomBorders` | 아래쪽 테두리 |
| `ToolTipLeftBorders` | 왼쪽 테두리 |
| `ToolTipVerticalBorders` | 세로 테두리 |
| `ToolTipRightBorders` | 오른쪽 테두리 |
| `ToolTipClearBorders` | 테두리 지우기 |
| `ToolTipDiagDownBorders` | 대각선 (↘) |
| `ToolTipDiagUpBorders` | 대각선 (↗) |
| `ToolTipBorderColor` | 테두리 색 |
| `ToolTipBorderStyle` | 테두리 스타일 |

### 4.7 숫자 서식

| 툴팁 키 | 설명 |
|---------|------|
| `ToolTipCurrency` | 통화 |
| `ToolTipPercent` | 백분율 |
| `ToolTipComma` | 쉼표 |
| `ToolTipIncreaseDecimal` | 소수점 증가 |
| `ToolTipDecreaseDecimal` | 소수점 감소 |

## 5. 버튼→명령 매핑 (Actor/Updater)

### 5.1 텍스트 서식

| Actor | Updater | 명령 |
|-------|---------|------|
| `boldActor` | `boldUpdater` | 굵게 |
| `italicActor` | `italicUpdater` | 기울임 |
| `underlineActor` | `underlineUpdater` | 밑줄 |
| `strikethroughActor` | `strikethroughUpdater` | 취소선 |
| `fontcolorActor` | `fontcolorUpdater` | 글자 색 |
| — | `fontnameUpdater` | 글꼴 이름 |
| — | `fontsizeUpdater` | 글꼴 크기 |
| `highlightcolorActor` | `highlightcolorUpdater` | 형광펜 |

### 5.2 셀/표 작업

| Actor | Updater | 명령 |
|-------|---------|------|
| `cellmergeActor` | `cellmergeUpdater` | 셀 병합 |
| `cellunmergeActor` | `cellunmergeUpdater` | 셀 분할 |
| `cellinsertseqActor` | `cellinsertseqUpdater` | 셀 삽입 |
| `celldeleteActor` | `celldeleteUpdater` | 셀 삭제 |
| `cellremoveseqActor` | `cellremoveseqUpdater` | 셀 순서 제거 |
| `cellwidthdistributeActor` | `cellwidthdistributeUpdater` | 열 폭 균등 배분 |
| `cellheightdistributeActor` | `cellheightdistributeUpdater` | 행 높이 균등 배분 |

### 5.3 클립보드

| Actor | Updater | 명령 |
|-------|---------|------|
| — | `clipboardcopyUpdater` | 복사 |
| — | `clipboardcutUpdater` | 잘라내기 |
| — | `clipboardpasteUpdater` | 붙여넣기 |

### 5.4 문서 작업

| Actor | Updater | 명령 |
|-------|---------|------|
| `downloadActor` | `downloadUpdater` | 다운로드 |
| `desktopconfirmActor` | `desktopconfirmUpdater` | 데스크톱 확인 |
| `desktopexecuteActor` | `desktopexecuteUpdater` | 데스크톱 실행 |
| `closeAppActor` | — | 앱 닫기 |
| `docInfoActor` | — | 문서 정보 |
| `edithelpActor` | — | 편집 도움말 |

### 5.5 삽입

| Actor | Updater | 명령 |
|-------|---------|------|
| — | `insertsymbolsUpdater` | 기호 삽입 |
| — | `inserttableUpdater` | 표 삽입 |
| — | `insertimageUpdater` | 그림 삽입 |

### 5.6 차트

| Actor | Updater | 명령 |
|-------|---------|------|
| `ChartTypeActor` | `ChartTypeNStyleUpdater` | 차트 유형 |
| `ChartTypeNStyleActor` | — | 차트 유형+스타일 |
| `DataLabelPositionActor` | `DataLabelPositionUpdater` | 데이터 레이블 위치 |
| `LegendPositionActor` | `LegendPositionUpdater` | 범례 위치 |
| `HorizontalGridLinesActor` | `HorizontalGridLinesUpdater` | 가로 눈금선 |
| `VerticalGridLinesActor` | `VerticalGridLinesUpdater` | 세로 눈금선 |
| `SwitchRowColumnActor` | `SwitchRowColumnUpdater` | 행/열 전환 |
| `EditChartDataActor` | — | 차트 데이터 편집 |

### 5.7 변경 내용 추적

| Actor | Updater | 명령 |
|-------|---------|------|
| `OnTrackChangeActor` | `OnTrackChangeUpdater` | 변경 추적 시작 |
| `ApplyTrackChangeActor` | `ApplyTrackChangeUpdater` | 변경 적용 |
| `CancelTrackChangeActor` | `CancelTrackChangeUpdater` | 변경 취소 |
| `NextTrackChangeActor` | `NextTrackChangeUpdater` | 다음 변경 |
| `PrevTrackChangeActor` | `PrevTrackChangeUpdater` | 이전 변경 |
| `ViewTrackChangeAllActor` | `ViewTrackChangeAllUpdater` | 모든 변경 보기 |
| `ViewMyTrackChangeOnlyActor` | `ViewMyTrackChangeOnlyUpdater` | 내 변경만 보기 |
| `ViewTCInsAndDelActor` | `ViewTCInsAndDelUpdater` | 삽입/삭제 보기 |
| `ViewTrackChangeShapeActor` | `ViewTrackChangeShapeUpdater` | 도형 변경 보기 |
| `TrackChangeFinalActor` | `TrackChangeFinalUpdater` | 최종 결과 |
| `TrackChangeFinalInfoActor` | `TrackChangeFinalInfoUpdater` | 최종 결과 정보 |
| `NextStepAfterApplyTCActor` | `NextStepAfterApplyTCUpdater` | 적용 후 다음 |
| `NextStepAfterCancelTCActor` | `NextStepAfterCancelTCUpdater` | 취소 후 다음 |

### 5.8 기타

| Actor | Updater | 명령 |
|-------|---------|------|
| — | `paraalignUpdater` | 문단 정렬 |
| — | `formatcopyUpdater` | 서식 복사 |
| — | `equationUpdater` | 수식 |
| `accessibilityActor` | `accessibilityUpdater` | 접근성 |
| `commonMessageBoxActor` | — | 공통 메시지 박스 |
| `hwpActorUpdater` | — | HWP 통합 |
| `coreActor` | — | 코어 |

## 6. HTML data 속성 바인딩

버튼과 명령을 연결하는 `data-*` 속성 체계:

### 6.1 명령 바인딩

| 속성 | 용도 |
|------|------|
| `data-ui-command` | UI 명령 매핑 |
| `data-command` | 직접 명령 매핑 |
| `data-dbl-click-command` | 더블클릭 명령 |
| `data-non-exec` | 실행 안 함 플래그 |

### 6.2 값 바인딩

| 속성 | 용도 |
|------|------|
| `data-value` | 버튼 값 |
| `data-value-key` | 값 참조 키 |
| `data-default-value` | 기본값 |
| `data-replace-value` | 값 교체 플래그 |
| `data-unit` | 단위 정보 |

### 6.3 위젯 바인딩

| 속성 | 용도 |
|------|------|
| `data-bind-widget` | 위젯 바인딩 |
| `data-spinner` | 스피너 설정 |
| `data-input-exec` | 입력 실행 플래그 |
| `data-lazy-convert-name` | 지연 변환 이름 |

### 6.4 UI 표시

| 속성 | 용도 |
|------|------|
| `data-name` | 요소 이름 |
| `data-label` | 라벨 |
| `data-nls-name` | 다국어 문자열 키 |
| `data-icon-name` | 아이콘 참조 |
| `data-icon-size` | 아이콘 크기 |
| `data-toggle-class` | CSS 토글 클래스 |
| `data-disabled-key` | 비활성화 키 |

### 6.5 템플릿/샘플

| 속성 | 용도 |
|------|------|
| `data-template-name` | 템플릿 이름 |
| `data-template-group-item` | 템플릿 그룹 항목 |
| `data-sample-name` | 샘플 요소 참조 |
| `data-sample-value` | 샘플 값 |
| `data-sample-value-key` | 샘플 값 키 |

### 6.6 드래그/색상

| 속성 | 용도 |
|------|------|
| `data-drag` | 드래그 가능 플래그 |
| `data-drop-cursor` | 드롭 커서 스타일 |
| `data-color-position` | 색상 위치 |
| `data-hsv-color` | HSV 색상 값 |
| `data-org-color` | 원본 색상 |

### 6.7 기타

| 속성 | 용도 |
|------|------|
| `data-area-type` | 영역 유형 |
| `data-device` | 장치 유형 |
| `data-app` | 앱 식별자 |
| `data-switch-button` | 버튼 전환 |
| `data-match-type` | 찾기 유형 |
| `data-match` | 찾기 데이터 |
| `data-match-group` | 찾기 그룹 |

## 7. CSS 클래스 체계

### 7.1 메뉴/네비게이션

```
.menu_bar                   메뉴 바
.menu_box                   메뉴 박스
.menu_box_subtitle          메뉴 박스 부제목
.menu_select_list           메뉴 선택 목록
.title_menubar              타이틀 메뉴바
.btn_menu_bar_view_mode     메뉴바 보기 모드 버튼
```

### 7.2 탭/그룹

```
.tab_box                    탭 박스
.tab_group                  탭 그룹
.tab_sheets                 탭 시트
.btn_tab                    탭 버튼
.s_group                    소그룹
.sub_group                  서브그룹
.sub_group_box              서브그룹 박스
.sub_group_title            서브그룹 제목
```

### 7.3 버튼

```
.aori_btn                   기본 버튼
.aori_btn_arrow             화살표 버튼 (드롭다운)
.aori_btn_close             닫기 버튼
.aori_btn_loading           로딩 버튼
.aori_btn_minimum           최소화 버튼
.aori_btn_opacity           투명도 버튼
.aori_btn_send              보내기 버튼
.aori_btn_user              사용자 버튼
.aori_main_btn              메인 버튼
```

### 7.4 컨텍스트 메뉴

```
.context_menu               컨텍스트 메뉴
.tool_cmd_contextmenu       명령 컨텍스트 메뉴
.quick_menu                 빠른 메뉴
.tool_quick_menu            빠른 메뉴 도구
.aori_action                액션 항목
```

### 7.5 도형 관련 탭

```
.drawn_tab_btn              그리기 탭 버튼
.effect_tab_btn             효과 탭 버튼
.size_tab_btn               크기 탭 버튼
.shape_cornerTabs           모서리 탭
.shape_leftRightRibbon      좌우 리본
.shape_plaqueTabs           플래크 탭
.shape_ribbon               리본
.shape_squareTabs           사각 탭
```

### 7.6 색상/레이아웃

```
.color_group                색상 그룹
.slide_layout_group         슬라이드 레이아웃 그룹
.slide_layout_menu          슬라이드 레이아웃 메뉴
```

## 8. 서식 관련 문자열 상수

### 8.1 글꼴

```
Font                        글꼴
FontName                    글꼴 이름
FontSize                    글꼴 크기
FontColor                   글꼴 색
FontColorDark               어두운 글꼴 색
FontColorLight              밝은 글꼴 색
FontHighlightColor          형광펜 색
LargerFont / IncreaseFontSize    글꼴 크게
SmallerFont / DecreaseFontSize   글꼴 작게
```

### 8.2 색상 사전정의

```
BlackFontColor              검정
BlueFontColor               파랑
GreenFontColor              초록
PurpleFontColor             보라
RedFontColor                빨강
TealFontColor               청록
WhiteFontColor              흰색
YellowFontColor             노랑
```

### 8.3 삽입 항목

```
Insert                      삽입
InsertTable                 표 삽입
InsertRow                   행 삽입
InsertColumn                열 삽입
InsertRowAbove              위에 행 삽입
InsertRowAfter              아래에 행 삽입
InsertColumnLeft            왼쪽에 열 삽입
InsertColumnRight           오른쪽에 열 삽입
InsertRowCell               행 셀 삽입
InsertColumnCell            열 셀 삽입
InsertLink                  링크 삽입
InsertHyperlink             하이퍼링크 삽입
InsertCell                  셀 삽입
InsertImage                 그림 삽입
InsertBookmark              책갈피 삽입
InsertComment               메모 삽입
InsertTab                   탭 삽입
InsertShape                 도형 삽입
InsertMemo                  메모 삽입
```

### 8.4 텍스트 서식

```
Bold                        굵게
Italic                      기울임
Underline                   밑줄
DoubleUnderline             이중 밑줄
Strikethrough               취소선
StrikeThrough               취소선 (대소문자 변형)
Subscript                   아래 첨자
Superscript                 위 첨자
Alignment                   정렬
AlignLeft                   왼쪽 정렬
AlignCenter                 가운데 정렬
AlignRight                  오른쪽 정렬
Indent                      들여쓰기
RightIndent                 오른쪽 들여쓰기
FirstLineIndent             첫 줄 들여쓰기
```

## 9. 버튼 상태 관리

### 9.1 상태 메서드

```javascript
setEnabled(boolean)         // 활성/비활성
setActive(boolean)          // 활성 상태 (토글)
setVisible(boolean)         // 표시/숨김
setChecked(boolean)         // 체크 상태
```

### 9.2 CSS 상태 클래스

```
disabled                    비활성
active                      활성 (토글 ON)
checked                     체크됨
hover                       마우스 오버
```

### 9.3 상태 갱신 흐름

```
사용자 동작
  → Actor 실행
    → Updater 호출
      → 버튼 상태 갱신 (setActive/setEnabled 등)
```

## 10. 드롭다운/콤보 컴포넌트

| 컴포넌트 | Updater | 설명 |
|---------|---------|------|
| 글꼴 선택 | `fontnameUpdater` | 글꼴 이름 드롭다운 |
| 글꼴 크기 | `fontsizeUpdater` | 글꼴 크기 드롭다운 |
| 글자 색 | `fontcolorUpdater` | 색상 팔레트 드롭다운 |
| 형광펜 | `highlightcolorUpdater` | 형광펜 색상 드롭다운 |
| 문단 정렬 | `paraalignUpdater` | 정렬 방식 드롭다운 |

콤보 감지: `isCombo()` 메서드로 부모 요소가 콤보인지 판별

## 11. 컨텍스트 메뉴

### 11.1 CSS

```css
.context_menu { /* 우클릭 메뉴 */ }
.tool_cmd_contextmenu { /* 명령 컨텍스트 메뉴 */ }
```

### 11.2 이벤트 바인딩

```javascript
// contextmenu 이벤트로 트리거
addEventListener("contextmenu", handler);
// data-command 속성으로 메뉴 항목→명령 매핑
```

## 12. rhwp와의 비교

| 항목 | webhwp | rhwp |
|------|--------|------|
| 메뉴 구조 | 8개 최상위 메뉴 + 서브메뉴 | 미구현 (뷰어) |
| 툴바 | 빠른 도구 모음 + 탭 그룹 | 미구현 |
| 명령 바인딩 | data-* 속성 + Actor/Updater | 미구현 |
| 상태 관리 | setEnabled/setActive/setChecked | 미구현 |
| 컨텍스트 메뉴 | contextmenu 이벤트 + data-command | 미구현 |
| 다국어 | nls12 청크 (327KB) | 미구현 |
| 색상 팔레트 | HSV + 사전정의 8색 | 미구현 |
| 변경 추적 | 13개 Actor/Updater 쌍 | 미구현 |
| 차트 편집 | 8개 차트 Actor | 미구현 |
| 접근성 | accessibilityActor | 미구현 |

---

*분석 일자: 2026-02-09*
