# 인라인 개체 뒤 저장 탭의 시작점

## 독립 근거와 위반된 관계

`samples/HWP5-nopassword-123456.hwp` 1쪽의 문단 14는 글자처럼 취급하는
폭 19,840HU 글상자 뒤에 탭과 숫자 `4`를 둔다. 대응 HWPX의 탭은 오른쪽
종류와 13,888HU의 전진을 저장한다. 독립 한컴 2024 Print PDF
`pdf/HWP5-nopassword-123456-hwp-2024.pdf`에서 목차 숫자 3·3·4·5의
글리프 왼쪽 경계는 모두 462.36pt(96dpi 약 616.48px)다.

해시를 기록한 `8244f3455b` producer는 숫자 run x=616.4px이고,
`bddd5d58e4` producer는 x=351.9px이다. 후자는 숫자 `4`가 앞 글상자의
문자와 겹친다. 이 두 producer 사이에 다른 코드 변경도 있어 특정 커밋을
단독 원인으로 확정하지 않는다. 독립 PDF의 glyph 경계와 run 원점을 구별한다.

## 생산·소비 경로와 보정 조건

`resolve_last_tab_pending`은 수용한 저장 전진을 전달받은 스타일의
`line_x_offset`에 더한다. 측정은 `line_tac_offsets_for_width`를 지나
개체 뒤의 실제 fragment 시작을 계산하지만, 배치는 `run_tacs` 처리보다
먼저 pending을 만들어 개체의 선행 폭을 누락할 수 있다. 배치와 측정이
각자 재추측하지 않고 탭을 소유한 같은 fragment 시작과 저장 전진을
공통 helper로 해소하도록 보정한다. 특정 문서·쪽·수치 분기는 사용하지 않는다.

선행 개체 없이 탭만 있는 줄, 탭 앞뒤 텍스트와 여러 탭, 탭 뒤 공백 run,
자동 우측 정지, 저장 전진이 없는 재조판과 편람 기존 목차 페이지를 반례로
확인한다. 같은 원본의 전체 Native/fresh WASM, 독립 Print 및 페이지 수를
검증하고 수정 전후 해당 위치를 직접 판독해야 한다. 현재 후보 미적용,
최종 시각·회귀 검증 미실행이다.

### PDF 가져오기 글상자의 저장 줄 원점과 문단 여백

#7998 원 검사 `tests/cases/issue_7998_pdf_shared_tab_stops.rs`와
`issue_7998_pdf_source_space_tabs.rs`는 같은 PDF를 살아 있는 `DocumentCore`에서 읽고,
HWP/HWPX 저장·재열기와 편집까지 확인한다. 독립 PDF의 첫 `ALPHA`는 x=20pt
(96dpi에서 26.67px), `BETA`는 x=70pt(93.33px)다. 현재 live 출력의 `ALPHA`는
53.3px이며 세 실패의 `BETA`는 104.1px·130.4px·123.9px이다.

PDF 관측을 내린 첫 LineSeg `column_start=2000HU`와 문단 `margin_left=4000`은
문단 여백의 짝수 단위 변환을 적용하면 각각 26.67px이다. 생산 경로는
`structured_lowering.rs`의 `line.left - cell_x`와 문단 여백이며 소비 경로는
`paragraph_layout.rs`의 `stored_textbox_frame → effective_col_x → effective_margin_left`다.
줄 시작과 같은 문단 여백이 중복되지 않도록 공통 프레임에서 확인하고, 줄별 들여쓰기와
오른쪽 여백은 별도 유지한다. 원본 저장 프레임이 아닌 재조판 및 서로 다른 원점의 줄은
기존 경로를 유지한다. 독립 PDF 좌표, 원 검사 세 건 및 같은 경로의 기존 통과 사례,
편람 205쪽 글상자 정렬을 수정 전후 검증한다. 현재는 수정 전 진단이며 통과하지 않았다.
원본 좌표와 live 실행 결과는 ignored `output/pr-review/issue8153-handbook-raw100-20261022/`의
`pdf-import-tab-parashape-trace-20261028.log`에 있다.

추가 관찰: 세 실패의 첫 줄은 각각 `(column_start, segment_width, 오른 여백/2)`가
`(2000,23904,14096)`, `(2000,24891,13109)`, `(2000,24647,13353)`HU로,
세 값의 합이 모두 글상자 폭 40000HU다. 왼쪽 여백뿐 아니라 오른쪽 여백도 저장
줄 상자에서 이미 빠져 있다. 왼쪽만 제거한 후보는 `available_width`에서 오른쪽을
재차 빼 기존 정상 공백 사례를 포함해 4건이 실패했다. 이 사실을 바탕으로 저장
줄의 양쪽 끝이 문단 콘텐츠 경계와 일치하는 경우를 검사하며, 줄 프레임과 탭의
기준 원점이 같은지 실행 추적으로 확인한다. 다른 프레임과 추가 들여쓰기는 별도다.

실행 추적에서 글상자 원점은 0px, 저장 프레임 시작 26.667px·폭 318.720px,
오른쪽 여백 187.947px이었다. 여백을 다시 빼면서 탭 계산의 가용 폭이
130.773px이 되었고 독립 PDF의 160px 정지보다 작았다. 저장 프레임의 끝과
문단 오른쪽 경계가 1HU 이내로 일치하면 그 여백을 다시 차감하지 않는 후보를
적용했다. 단위 추적 로그는 ignored output의 `pdf-import-tab-frame-trace-20261028.log`,
보정 후 재생성 suite 집중 검사는 `textbox-frame-regenerated-suite-focused-20261028.log`다.
후보 통과·영향 페이지 시각 검증은 실행 결과 확인 전까지 미완료다.

저장 줄 양쪽 경계의 중복 여백 제거 후보와 인라인 개체 뒤 탭 원점 후보를 함께
검사했다. suite를 현재 소스에서 재생성한 뒤 PDF 탭·기존 원본 공백, 편람
그림·물리 행·빈 캐리어, 글자 겹침과 저장 프레임 대조군 29건이 모두 통과했다.
`textbox-frame-regenerated-suite-focused-20261028.log`의 실행 결과이며 최종
전체 회귀·Native/fresh WASM 전쪽 및 직접 판독은 아직 남았다. 실행 뒤 Rust
포맷의 줄바꿈만 정리했고 나머지 미확인 회귀를 재검사한다.
