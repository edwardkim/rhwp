---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# webhwp 분석: 이벤트/명령 체계

> 분석 대상: `webhwp/js/hwpApp.*.chunk.js` (5.17MB minified)
> 분석 일자: 2026-02-09

## 1. 이벤트 아키텍처 개요

```
User Input (키보드/마우스/터치)
  ↓
addEventListener (keydown/mousedown/etc)
  ↓
Event Handler (i.ZTr, i.qTr 등)
  ↓
AID/ENTID 추출 (액션 ID / 엔티티 ID)
  ↓
PPt(명령) 또는 tbr.$mr(ENTID)
  ↓
Action Mapping (t.e_xxx = {action: handler})
  ↓
Actor 실행 (XXXActor.irh())
  ↓
Updater 실행 (XXXUpdater)
  ↓
nwr.Update() / nwr.SetEvent()
```

## 2. 명령 접두사 규칙

| 접두사 | 영역 | 예시 | 총 수 |
|--------|------|------|-------|
| `e_` | Edit (편집) | `e_copy`, `e_undo`, `e_refresh` | 61 |
| `d_` | Document (문서) | `d_save`, `d_print`, `d_download` | 10 |
| `p_` | Paragraph/Property (문단/속성) | `p_page_break`, `p_align` | 38 |
| `c_` | Cell (셀/표) | `c_merge`, `c_delete`, `c_border_fill` | 31 |
| `s_` | Shape/Style (도형/스타일) | `s_picture`, `s_insert_shape` | 30 |

## 3. 편집 명령 (e_ 접두사, 61개)

### 클립보드
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `e_copy` | `clipboardcopy` | 복사 |
| `e_cut` | `clipboardcut` | 잘라내기 |
| `e_paste` | `clipboardpaste` | 붙여넣기 |
| `e_paste_cell` | — | 셀 붙여넣기 |

### 실행 취소
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `e_undo` | `undo` | 실행 취소 |
| `e_redo` | `redo` | 다시 실행 |

### 삭제
| 명령 | 설명 |
|------|------|
| `e_delete` | 삭제 |
| `e_delete_field` | 필드 삭제 |
| `e_delete_link` | 링크 삭제 |
| `e_delete_header_footer` | 머리말/꼬리말 삭제 |
| `e_delete_bookmark_area` | 북마크 영역 삭제 |
| `e_delete_click_here_field` | 여기를 클릭 필드 삭제 |

### 삽입
| 명령 | 설명 |
|------|------|
| `e_insert_caption` | 캡션 삽입 |
| `e_insert_field` | 필드 삽입 |
| `e_insert_footer` | 꼬리말 삽입 |
| `e_insert_header` | 머리말 삽입 |
| `e_insert_hyperlink` | 하이퍼링크 삽입 |
| `e_insert_notes` | 각주/미주 삽입 |
| `e_insert_web_video` | 웹 동영상 삽입 |

### 검색
| 명령 | 설명 |
|------|------|
| `e_find_dialog` | 찾기 대화상자 |
| `e_find_replace` | 찾아 바꾸기 |
| `e_goto` | 이동 |

### 머리말/꼬리말
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `e_header_footer_close` | — | 머리말/꼬리말 닫기 |
| `e_next_header_footer` | `pagenextheaderfooter` | 다음 머리말/꼬리말 |
| `e_prev_header_footer` | `pageprevheaderfooter` | 이전 머리말/꼬리말 |
| `e_delete_header_footer` | `pagedeleteheaderfooter` | 삭제 |

### 스타일
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `e_style_add_edit` | — | 스타일 추가/편집 |
| `e_style_change` | — | 스타일 변경 |
| `e_style_item` | `styleitem` | 스타일 항목 |
| `e_style_list` | — | 스타일 목록 |
| `e_style_normal` | — | 기본 스타일 |

### 보기
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `e_ctrl_mark` | `viewctrlmark` | 컨트롤 마크 |
| `e_para_mark` | `viewparamark` | 문단 부호 |
| `e_ruler` | — | 눈금자 |
| `e_view_option_paper` | — | 용지 옵션 |
| `e_view_scale` | — | 보기 배율 |
| `e_view_zoom_page_one` | `viewzoomone` | 100% |
| `e_view_zoom_page_two` | `viewzoomtwo` | 200% |
| `e_view_zoom_page_three` | `viewzoomthree` | 300% |
| `e_zoom` | — | 줌 |

### 북마크/링크
| 명령 | 설명 |
|------|------|
| `e_bookmark_list` | 북마크 목록 |
| `e_modify_bookmark` | 북마크 수정 |
| `e_modify_link` | 링크 수정 |
| `e_open_link` | 링크 열기 |

### 메모
| 명령 | 설명 |
|------|------|
| `e_menu_insert_memo` | 메모 삽입 |
| `e_menu_view_memo` | 메모 보기 |
| `e_menu_delete_memo` | 메모 삭제 |
| `e_menu_view_memo_guide` | 메모 가이드 |

### 기타
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `e_refresh` | `refresh` | 새로고침 |
| `e_review` | `review` | 검토 |
| `e_select` | — | 선택 |
| `e_help` | `edithelp` | 도움말 |
| `e_format_copy` | `formatcopy` | 서식 복사 |
| `e_formula` | — | 수식 |
| `e_edit_formula` | — | 수식 편집 |
| `e_object_properties` | — | 개체 속성 |
| `e_shape_copy_paste` | — | 도형 복사/붙여넣기 |
| `e_desktop_confirm` | `desktopconfirm` | 데스크톱 확인 |
| `e_desktop_execute` | `desktopexecute` | 데스크톱 실행 |
| `e_dialog_message_box` | — | 메시지 박스 |
| `e_chart_axis_title` | — | 차트 축 제목 |
| `e_chart_data` | — | 차트 데이터 |

## 4. 문서 명령 (d_ 접두사, 10개)

| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `d_save` | `save` | 저장 |
| `d_save_as` | `saveas` | 다른 이름 저장 |
| `d_save_as_button` | — | 저장 버튼 |
| `d_download` | `download` | 다운로드 |
| `d_pdf_download` | `pdfdownload` | PDF 다운로드 |
| `d_print` | `print` | 인쇄 |
| `d_insert_file` | — | 파일 삽입 |
| `d_page_setup` | — | 페이지 설정 |
| `d_grid` | `viewgrid` | 눈금선 |
| `d_delete_ctrls` | — | 컨트롤 삭제 |
| `d_rename` | `rename` | 이름 변경 |

## 5. 문단/속성 명령 (p_ 접두사, 38개)

### 페이지/단
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `p_page_break` | `pagebreak` | 쪽 나누기 |
| `p_page_hiding` | `pagehiding` | 쪽 감추기 |
| `p_column_break` | `pagecolumnbreak` | 단 나누기 |
| `p_break_new_column` | `pagebreaknewcolumn` | 새 단 나누기 |
| `p_column_one` | `pagecolumnone` | 1단 |
| `p_column_two` | `pagecolumntwo` | 2단 |
| `p_column_three` | `pagecolumnthree` | 3단 |
| `p_column_left` | `pagecolumnleft` | 왼쪽 단 |
| `p_column_right` | `pagecolumnright` | 오른쪽 단 |
| `p_new_number` | `pagenewnumber` | 새 쪽 번호 |

### 목록
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `p_list` | `paranumbullet` | 번호/글머리 |
| `p_bullet_list` | `parabulletlist` | 글머리 기호 |
| `p_number_list` | `paranumlist` | 번호 매기기 |
| `p_bullet_number_list` | — | 번호/글머리 목록 |

### 문단 수준
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `p_level_decrease` | `paraleveldec` | 수준 감소 |
| `p_level_increase` | `paralevelinc` | 수준 증가 |
| `p_align` | `paraalign` | 정렬 |
| `p_line_spacing` | `paralinespacing` | 줄 간격 |
| `p_language` | `planguage` | 언어 |
| `p_properties` | — | 속성 |

### 화살표/선 스타일
| 명령 | 설명 |
|------|------|
| `p_arrow_normal` | 일반 |
| `p_arrow_arrow` | 화살표 |
| `p_arrow_spear` | 창 |
| `p_arrow_concavearrow` | 오목 화살표 |
| `p_arrow_box` | 상자 |
| `p_arrow_circle` | 원 |
| `p_arrow_diamond` | 다이아몬드 |
| `p_arrow_nofillbox` | 빈 상자 |
| `p_arrow_nofillcircle` | 빈 원 |
| `p_arrow_nofilldiamond` | 빈 다이아몬드 |
| `p_arrow_start` | 시작점 |
| `p_arrow_end` | 끝점 |
| `p_endcap_flat` | 평평한 끝 |
| `p_endcap_round` | 둥근 끝 |
| `p_line_arrowstart_size` | 시작 화살표 크기 |
| `p_line_arrowend_size` | 끝 화살표 크기 |

## 6. 셀/표 명령 (c_ 접두사, 31개)

| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `c_merge` | `cellmerge` | 셀 병합 |
| `c_unmerge` | `cellunmerge` | 셀 병합 해제 |
| `c_split` | — | 셀 분할 |
| `c_delete` | `celldelete` | 셀 삭제 |
| `c_insert_row_col` | — | 행/열 삽입 |
| `c_insert_row_col_list` | `cellinsertrowcol` | 행/열 삽입 목록 |
| `c_remove_row_col` | — | 행/열 삭제 |
| `c_remove_row_col_list` | `cellremoverowcol` | 행/열 삭제 목록 |
| `c_cell_calc_sum` | `cellcalcsum` | 합계 |
| `c_cell_calc_average` | `cellcalcavg` | 평균 |
| `c_cell_calc_multiplication` | `cellcalcmul` | 곱셈 |
| `c_cell_insert_sep` | `cellinsertseq` | 일련번호 삽입 |
| `c_cell_remove_sep` | `cellremoveseq` | 일련번호 삭제 |
| `c_cell_thousands_sep` | — | 천 단위 구분 |
| `c_height_distribute` | `cellheightdistribute` | 높이 균등 |
| `c_width_distribute` | `cellwidthdistribute` | 폭 균등 |
| `c_border_fill` | — | 테두리/채우기 |
| `c_div_border_fill` | `cellborderfill` | 영역 테두리 |
| `c_zone_border_fill` | `cellzoneborderfill` | 구역 테두리 |
| `c_context` | — | 컨텍스트 |
| `c_page_break` | — | 쪽 나누기 |

## 7. 도형/스타일 명령 (s_ 접두사, 30개)

### 그림 효과
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `s_picture` | — | 그림 기본 |
| `s_picture_brightness_increase` | `picturebrightnessinc` | 밝기 증가 |
| `s_picture_brightness_descrese` | `picturebrightnessdec` | 밝기 감소 |
| `s_picture_brightness_none` | `picturebrightnessnone` | 밝기 초기화 |
| `s_picture_contrast_increase` | `picturecontrastinc` | 대비 증가 |
| `s_picture_contrast_descrese` | `picturecontrastdec` | 대비 감소 |
| `s_picture_contrast_none` | `picturecontrastnone` | 대비 초기화 |
| `s_picture_effect_blackandwhite` | `pictureeffectblackwhite` | 흑백 |
| `s_picture_effect_gray` | `pictureeffectgray` | 회색조 |
| `s_picture_effect_noeffect` | `pictureeffectnoeffect` | 효과 없음 |
| `s_picture_effect_watermark` | `pictureeffectwatermark` | 워터마크 |
| `s_picture_to_original` | `picturetooriginal` | 원본 복원 |

### 삽입/서식
| 명령 | 설명 |
|------|------|
| `s_insert_line` | 선 삽입 |
| `s_insert_shape` | 도형 삽입 |
| `s_insert_textbox` | 텍스트 상자 삽입 |
| `s_format_copy` | 서식 복사 |
| `s_format_paste` | 서식 붙여넣기 |
| `s_order` | 순서 |
| `s_detach_textbox` | 텍스트 상자 분리 |

## 8. 직접 명령 (접두사 없음)

### 텍스트 서식
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `bold` | `bold` | 굵게 |
| `italic` | `italic` | 기울임 |
| `underline` | `underline` | 밑줄 |
| `strikethrough` | `strikethrough` | 취소선 |
| `font_color` | `fontcolor` | 글자색 |
| `font_highlight_color` | `highlightcolor` | 형광펜 |
| `font_name` | `fontname` | 글꼴 이름 |
| `font_size` | `fontsize` | 글꼴 크기 |
| `char_shape` | `charshape` | 글자 모양 |

### 문단 서식
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `para_shape` | `parashape` | 문단 모양 |
| `paragraph_style` | `styleitemex` | 문단 스타일 |

### 삽입
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `insert_image` | `insertpicture` | 그림 삽입 |
| `insert_table` | `inserttable` | 표 삽입 |
| `insert_symbols` | `insertsymbols` | 기호 삽입 |
| `equation` | `equation` | 수식 |

### 기타
| 명령 | Actor/Updater 이름 | 설명 |
|------|-------------------|------|
| `accessibility` | `accessibility` | 접근성 |
| `closeApp` | — | 앱 닫기 |
| `share` | `share` | 공유 |
| `dialog_search_doc` | `searchdoc` | 문서 검색 |
| `dialog_shortcut_info` | `shortcut` | 단축키 정보 |

## 9. Actor 클래스 (92개)

### 카테고리별 분류

| 카테고리 | Actor 수 | 주요 항목 |
|----------|---------|----------|
| TrackChange | 13 | ApplyTC, CancelTC, NextTC, PrevTC, OnTC, ViewTC |
| Chart | 8 | ChartType, DataLabel, GridLines, Legend, SwitchRowColumn |
| Cell/Table | 13 | cellcalcsum, cellmerge, cellunmerge, celldelete, tablebordertransparent |
| Picture | 11 | brightness(inc/dec/none), contrast(inc/dec/none), effect(bw/gray/watermark) |
| Page | 11 | pagebreak, pagecolumn(break/1/2/3/left/right), headerfootetr(del/next/prev) |
| Paragraph | 2 | paraleveldec, paralevelinc |
| Text Format | 4 | bold, italic, strikethrough, underline |
| Color | 2 | fontcolor, highlightcolor |
| View | 7 | viewbasic, viewctrlmark, viewformat, viewgrid, viewparamark, viewzoom(1/2/3) |
| Core | 21 | save, download, print, undo, redo, refresh, review, rename, search 등 |

## 10. Updater 클래스 (111개)

| 카테고리 | Updater 수 | 주요 항목 |
|----------|-----------|----------|
| TrackChange | 13 | ApplyTC, CancelTC, ViewTC 등 |
| Chart | 6 | ChartTypeNStyle, DataLabel, GridLines 등 |
| Cell/Table | 16 | cellborderfill, cellinsertrowcol, cellmerge, tablecellprop 등 |
| Picture | 11 | brightness/contrast/effect 전체 |
| Page | 13 | pagebreak, pagehiding, pagenewnumber, pagecolumn 전체 |
| Paragraph | 8 | paraalign, parabulletlist, paralinespacing, parashape 등 |
| Text/Font | 9 | bold, italic, charshape, fontcolor, fontname, fontsize 등 |
| Clipboard | 3 | clipboardcopy, clipboardcut, clipboardpaste |
| Insert | 4 | equation, insertimage, insertsymbols, inserttable |
| View | 7 | viewbasic, viewctrlmark, viewgrid 등 |
| Core | 11 | undo, redo, download, print, saveas, formatcopy 등 |

## 11. Handler 클래스 (15개)

| 클래스 | 역할 |
|--------|------|
| `MainHandler` | 메인 핸들러 (최상위) |
| `CoreHandler` | 코어 기능 |
| `NormalHandler` | 일반 처리 |
| `PageHandler` | 페이지 처리 |
| `TableHandler` | 표 처리 |
| `PictureHandler` | 그림 처리 |
| `ChartHandler` | 차트 처리 |
| `ViewHandler` | 뷰 처리 |
| `DialogHandler` | 대화상자 |
| `FileHandler` | 파일 처리 |
| `MessageHandler` | 메시지 처리 |
| `CollaboHandler` | 협업 처리 |
| `TrackChangeHandler` | 변경 추적 |
| `HwpProviderHandler` | HWP 제공자 |
| `CHwpMouseCaretPosHandler` | 마우스/캐럿 위치 |

## 12. Manager 클래스 (9개)

| 클래스 | 역할 |
|--------|------|
| `ActionManager` | 액션 관리 |
| `FontManager` | 글꼴 관리 |
| `FocusManager` | 포커스 관리 |
| `UndOperManager` | 실행 취소 관리 |
| `SetUndOperManager` | 실행 취소 설정 |
| `WidgetManager` | 위젯 관리 |
| `AccessibleManager` | 접근성 관리 |
| `DXDTCacheManager` | DXDT 캐시 |
| `DistanceCacheManager` | 거리 캐시 |

## 13. 키보드 이벤트 처리

### 이벤트 흐름

```
keydown → i.ZTr (키보드 핸들러)
  ↓
AID/ENTID 추출: l.aid = e[n].AID, l.$1t = e[n].ENTID
  ↓
tbr.$mr(ENTID) — 툴바 명령 실행
  또는
PPt(명령) — 직접 명령 실행
```

### 이벤트 타입

| 이벤트 | 용도 |
|--------|------|
| `keydown` | 키 누름 (주 핸들러) |
| `keyup` | 키 뗌 |
| `keypress` | 문자 키 (레거시) |
| `textinput` | 텍스트 입력 |
| `beforeinput` | 입력 전 처리 |
| `compositionstart` | 한글 조합 시작 |
| `compositionupdate` | 한글 조합 진행 |
| `compositionend` | 한글 조합 완료 |

## 14. 마우스 이벤트 처리

### 버튼별 이벤트 코드

```javascript
// mousedown
0 → U.default.Fwe  // 좌클릭
1 → U.default.Wwe  // 중간클릭
2 → U.default.Gwe  // 우클릭

// mouseup
0 → U.default.Kwe
1 → U.default.Ywe
2 → U.default.jwe

// mousemove → U.default.Uwe
// contextmenu → U.default.Vwe
```

### 터치 이벤트 매핑

```javascript
touchstart  → mousedown
touchmove   → mousemove
touchend    → mouseup
```

## 15. nwr (Native Wrapper) 인터페이스

| 메서드 | 역할 |
|--------|------|
| `nwr.Update()` | 뷰 업데이트 |
| `nwr.UpdatePosition()` | 위치 업데이트 |
| `nwr.UpdateSel()` | 선택 업데이트 |
| `nwr.UpdateUserCursor()` | 커서 업데이트 |
| `nwr.CalcCaretPos()` | 캐럿 위치 계산 |
| `nwr.InvalidateSel()` | 선택 무효화 |
| `nwr.FlushData()` | 데이터 플러시 |
| `nwr.SetEvent(u, type)` | 이벤트 전달 |
| `nwr.GetCurrentZoom()` | 현재 줌 |
| `nwr.SetZoom()` | 줌 설정 |
| `nwr.SetZoomFitInWindow()` | 창 맞춤 줌 |
| `nwr.GetScrollPos()` | 스크롤 위치 |
| `nwr.GetViewDivRect()` | 뷰 영역 |
| `nwr.GetPaperMargins()` | 용지 여백 |
| `nwr.GetRowWidth()` | 행 폭 |
| `nwr.MousePosToPagePos()` | 마우스→페이지 좌표 |
| `nwr.PagePosToMousePos()` | 페이지→마우스 좌표 |
| `nwr.PagePosToViewPos()` | 페이지→뷰 좌표 |
| `nwr.SetListener()` | 리스너 설정 |

## 16. PPt/yNt 명령 디스패치

### PPt 직접 호출 (문자열)

```javascript
PPt("e_refresh")       // 새로고침
PPt("d_save")          // 저장
PPt("d_download")      // 다운로드
PPt("d_pdf_download")  // PDF 다운로드
PPt("d_print")         // 인쇄
```

### yNt 호출 (상수 코드)

`dt` 객체의 상수 코드로 명령 전달:

```javascript
yNt(dt.mD)   // 쪽 나누기
yNt(dt.bD)   // 단 나누기
yNt(dt.QD)   // 1단
yNt(dt.XD)   // 2단
yNt(dt.qD)   // 3단
yNt(dt.Ez)   // 머리말/꼬리말 삭제
// ... 총 30+ 상수 코드
```

## 17. addEventListener 등록 이벤트

| 이벤트 | 용도 |
|--------|------|
| `keydown` | 키보드 입력 |
| `click` | 클릭 |
| `mousedown` | 마우스 누름 |
| `mouseover` | 마우스 올림 |
| `mouseout` | 마우스 나감 |
| `scroll` | 스크롤 |
| `change` | 변경 |
| `load` | 로드 완료 |
| `message` | 메시지 수신 |
| `beforeunload` | 페이지 떠나기 전 |
| `pagehide` | 페이지 숨김 |
| `unload` | 언로드 |

## 18. SetEvent 이벤트 타입

| 타입 | 설명 |
|------|------|
| `WHEEL_H_EVENT` | 수평 휠 스크롤 |
| `WHEEL_V_EVENT` | 수직 휠 스크롤 |
| `SCROLL_EVENT` | 스크롤 |
| `ZOOM_EVENT` | 줌 |

## 19. 메뉴 카테고리별 명령 그룹

### 텍스트 서식

```javascript
["bold", "italic", "underline", "strikethrough", "superscript", "subscript",
 "fontName", "fontSize", "fontColor", "fontHighlightColor", "letterSpacing"]
```

### 문단 서식

```javascript
["justifyLeft", "justifyCenter", "justifyRight", "justifyFull",
 "alignment", "indent", "outdent", "gIndent", "gList",
 "indentAbsolute", "rightIndentAbsolute", "firstLine",
 "lineheight", "pBdr", "pBgColor", "paraStyle",
 "pTopMargin", "pBottomMargin", "tabs"]
```

### 삽입

```javascript
["hyperlink", "image", "shape", "movie", "footnote", "endnote",
 "bookmark", "header", "footer", "pageBreak", "tab",
 "textBoxEdit", "field", "symbol"]
```

### 표

```javascript
["table", "tRow", "tCell", "cColspan", "cRowspan",
 "tAlignment", "tStyle", "tblLook",
 "cBgColor", "cAlign", "cPadding", "cDisplay",
 "cBdrTop", "cBdrRight", "cBdrBottom", "cBdrLeft", "cVMerge"]
```

### 도형

```javascript
["sSize", "sPosition", "sRotation", "sHlink", "sMove", "sWrapText",
 "sBgColor", "sBdr", "sBgOpacity", "sBdrOpacity",
 "imageLineColor", "imageLinewidth", "imageOutline", "imageOrder", "imagePosition"]
```

### 페이지 설정

```javascript
["pageSetup", "pageMargin", "pageOrientation", "pageSize",
 "pageBackground", "ruler", "documentRename"]
```

## 20. window.HwpApp 전역 객체

### 주요 속성

| 속성 | 역할 |
|------|------|
| `ActionManager` | 액션 관리자 |
| `Core` | 코어 엔진 |
| `UIAPI` | UI API |
| `UIListener` | UI 리스너 |
| `FontManager` | 글꼴 관리자 |
| `Models` | 문서 모델 |
| `CaretView` | 캐럿 뷰 |
| `Bar` | 도구 모음 |
| `tbr` | 툴바 (명령 실행) |
| `document` | 문서 객체 |
| `appState` | 앱 상태 |
| `Eqt` | 수식 관련 |
| `Yv` | 서버 통신 (RPC) |
| `IMGLOADER` | 이미지 로더 |
| `HwpSecurity` | 보안 |
| `KeyLogger` | 키 로거 |

### 주요 메서드

| 메서드 | 역할 |
|--------|------|
| `Initialize` | 초기화 |
| `InitView` | 뷰 초기화 |
| `InitEventListener` | 이벤트 리스너 초기화 |
| `SetUIEventListener` | UI 이벤트 설정 |
| `UpdateView` | 뷰 업데이트 |
| `ResizeView` | 뷰 크기 조정 |
| `Focus` | 포커스 설정 |
| `Blur` | 포커스 해제 |
| `FlushData` | 데이터 플러시 |
| `TKs` | 엔진 데이터 로드 |
| `RESUME` | 재개 |

## 21. 통계 요약

| 항목 | 수 |
|------|-----|
| Actor 클래스 | 92 |
| Updater 클래스 | 111 |
| Handler 클래스 | 15 |
| Manager 클래스 | 9 |
| 명령 문자열 (e_/d_/p_/c_/s_) | 170+ |
| PPt 직접 호출 | 5 |
| yNt 호출 | 30+ |
| addEventListener 등록 | 12 |
| 메뉴 명령 그룹 | 7 |

---

*분석 일자: 2026-02-09*
