# 현재 보정 문서의 기존 회귀 검사 점검

2026-11-12, 제품 source `ae5c72c710ac116132d2b9dcf62078bbc884beff`.
정확한 입력 경로를 따라 `tests/cases/`와 공용 corpus 참조를 검색하고 assertion을 확인했다.
`samples/hwp3-sample16-hwp5.hwp`는 `-2024.hwp`와 SHA가 다르므로 같은 입력으로 확대하지 않았다.
이 선행 단계에서는 기존 정상 검사·허용치·golden을 바꾸지 않았고 교체한 기존 px 핀은 **0개**였다.
최신 devel 통합 뒤 추가한 공용 root 검사 점검과 영역 선택 교체는 문서 아래에 구분한다.

| 기존 검사 | 해당 입력에서의 보호 내용·판정 |
| --- | --- |
| `nested_declared_word_space_budget` | 중첩 표 3쪽 원문 숫자의 단일 소유·명시 공백 예산 대조와 독립 원점 ±2px. 현재 양쪽 raw 100%; 같은 숫자·리더를 확대 판독했다. 유효한 완료 쪽 좌표 유지. |
| `source_frame_control_ownership` | 중첩 표의 선언 높이·줄 메트릭, 내부 포함·내용 순서·하이픈 소유. `close(px(source height))`는 원본 HWPUNIT 계약이지 화면 맞춤 핀이 아니다. 유지. |
| `nested_table_height_cache_contract` | 같은 편집의 cache 유무에 따른 결과 동등성. 절대 좌표·쪽수 핀 없음. 유지. 별도 편집 저장본의 과거 저점은 현재 원본 검사 제거의 근거로 사용하지 않음. |
| `issue_4068_unclipped_cell_honors_valign` | 해당 중첩 표는 continuation 내부 셀의 포함·정렬·글줄 보존 관계. 이 파일의 다른 원본 px 핀으로 범위를 확대하지 않음. |
| `issue_8160_hwp2022_paragraph_relations` | 2022 입력의 본문·표 소유, 내용 순서·중복·포함과 독립 Print의 3·7·63·64쪽 완료 좌표. 현재 해당 쪽 양쪽 raw 100%. 원본 계약·좌표 유지. |
| `issue_8160_stored_shape_host_physical_frame` | HWPX 8쪽 제목·표의 같은 쪽 소유, 순서·완전성·선언 높이·하단 괘선. 독립 기준선·표 시작 ±1px는 현재 양쪽 raw 100%; 제목·괘선 확대 확인. 다른 HWP 입력은 그대로 유지. |
| `issue_8160_terminal_style_cell_frame` | HWPX 26쪽의 원문 두 문구·같은 셀·기준선 순서·내부 포함, 독립 원점 ±1px. 현재 양쪽 raw 100%; 같은 영역 확대 확인. 유지. |
| `issue_8160_saved_page_boundaries` | 2022/2024의 실제 내용 쪽 소속·분할 컷 연속·순서, 2022 23쪽 독립 라틴 원점·기준선 ±1px. 현재 23·24쪽 양쪽 raw 100%; S/W와 뒤 한글 확대 확인. 다른 저장 버전은 수정하지 않음. |
| `issue_8160_hwpx_stored_page_ownership` | 독립 쪽의 내용 소유·앞뒤 순서·분할 컷의 누락/중복 방지. 과거 쪽수만 고립해서 고정하지 않음. 유지. |
| `issue_8160_equation_reading_order` | HWPX 6쪽의 실제 두 수식·원본 script·TAC·줄/본문 내부 포함·순서. 픽셀 핀 없음. 실제 분수는 별도 Native/WASM/Print 확대 판독. |
| `issue_8160_toc_and_latin_font` | 2022 입력의 실제 HCI Poppy 가족 연결. 다른 `-2010` 입력의 목차·폭 검사로 확대하지 않음. |
| `issue_8160_fullwidth_cjk_quote` | HWPX 전각 인용부호의 같은 줄 advance·뒤 글자 간격 관계. 유지. |
| `issue_8160_hft_print_frame` | HWPX의 source/paint 글자·쪽 소유·bbox 동등성. 출력 위치를 임의 절대 px에 고정하지 않음. 유지. |
| `issue_8160_gradient_and_font_mapping` | HWPX 원본 그라데이션 색·방향의 parser 속성 계약. 유지. |
| `hwp3_table_grid_left_edges` | 해당 HWPX는 HWP3 cell 주소·span의 독립 원본 대조군. 화면 px 핀 없음. 유지. |
| `issue_8160_save_version_metadata` | 2022/2024 저장 제품·버전의 파일 메타데이터 계약. 조판 핀 없음. 유지. |
| `issue_8306_hwpx_verify_normalization` | 해당 중첩 표의 저장 IR 차이·정규화·손실 보고. 조판 핀 없음. 유지. |
| `off_canvas_baseline` | 공용 corpus 참조와 기존 이관 목록을 확인. 이번 보정으로 baseline·활성 제외 목록을 변경하지 않음. |
| `issue_8479_stored_ladder_missing_content`, `issue_8201_seal_pua` | 해당 경로는 설명·선행 사례 인용이며 실제 입력 assertion은 다른 원본. 범위를 확대하지 않음. |

전쪽 raw·원본/PDF SHA는 [정본 보호 증거](protection/comparison.json),
새로 판독한 기존 완료 쪽은 [확대 패널](font-supply/direct-review/)에 있다.
기존 관계 검사와 유효한 완료 쪽 좌표를 보존하면서 이번 신규 보호는 원 PUA 등록·해제,
WMF 명시 공급·중복 억제·저장 원본 보존, 실제 구형 분수 가족 연결로 추가했다.


## 최신 devel 통합 뒤 공용 root 검사 추가 점검

기준 devel `eb84afc036…`은 앞서 점검한 22개 focused-case 파일을 바꾸지 않았다.
추가 source·root contract 참조도 현재 입력 경로로 다시 확인했다. 공용
`tests/issue_2007_nested_cell_pagination.rs`의 종료 괘선 검사 두 곳은 `y > 820`, `폭 > 500`과
SVG의 `x < 80`, `폭 > 650` 영역으로 대상을 골랐다. 이를 원본 바깥 표·9×2 중첩 표의 소유,
그 표 자신의 폭·하단과 획 폭, 조상 셀의 실제 clip 관계로 바꿨다. 기존 비종료 2쪽의 종료선 부재,
종료 3쪽의 단일 괘선, 모든 조상 clip 내부 포함은 유지하고 SVG도 같은 소유 선·상자를 소비하는지 확인한다.

교체 전에 현재 Native·fresh 최적화 WASM 전 17쪽을 실제로 완료했다. 2쪽은 각각
99.99305%·99.96735%, 3쪽은 양쪽 모두 100.00000%다. 전쪽 최저는 각각
97.99083%·98.05646%이며 같은 종료 영역을 독립 Print와 확대 판독했다.
실제 source `759212053e4b…`의 집중 9건이 최소 글꼴 환경에서 통과했다.

이는 기존 관계 검사의 일괄 교체가 아니라 **두 검사의 고정 영역 선택을 관계 선택으로 바꾼 것**이다.
유효한 완료 쪽의 독립 ±1/2px 좌표와 원본 단위 계약은 유지한다. 추가 원본·golden·baseline·
판정 한도 변경은 없다. 제품 source도 바꾸지 않았다. 최신 측정과 집중 검사 영수증은
[최신 검증 원장](latest-eb84/README.md)에 구분한다.
