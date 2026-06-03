# Task M100 #1139 Stage 26

## 목적

Stage25 커밋 이후 `3-09월_교육_통합_2022.hwp` 19쪽이 한컴오피스 기준 화면/PDF와 다르게 배치되는 문제를 별도 스테이지로 추적한다.

## 시작 기준

- 기준 커밋: `0d855a0d` (`task 1139: Stage25 미주 overflow와 선택 회귀 보정`)
- Stage25 변경은 커밋 완료했다.
- Stage26 문서는 Stage25 커밋 이후 새 변경으로 생성한다.
- Stage26이 완전히 해결되기 전까지 커밋하지 않는다.

## 보고된 문제

- 작업지시자 시각 검증에서 19쪽이 한컴오피스와 다르다고 보고되었다.
- 한컴오피스 기준 19쪽 우측 단에는 `문29)`가 이어져야 한다.
- 현재 RHWP 기준 19쪽 우측 단은 `pi=992` 일부 1줄만 배치되고, `문29)`는 20쪽으로 넘어간다.
- 이전에 정상 동작하던 문단 모양 기능도 회귀되었다.
  - Stage26에서 19쪽 분배 문제와 함께 원인과 수정 범위를 확인한다.
  - 문단 모양 기능은 선택/커서/문단 속성 조회 경로와 연결되어 있으므로, Stage25의 미주 가상 문단 선택 보정과 충돌했는지 함께 점검한다.
- 후속 시각 검증에서 17쪽 우측 하단 `문30)` 아래 `n(A)=2 또는 n(A)=3` 줄이 직전 조건 줄과 겹친다고 보고되었다.
- 후속 시각 검증에서 10쪽 미주 모양의 `미주 사이` 적용이 회귀되어 `문11)` 제목/본문 분배와 하단 overflow가 한컴오피스와 달라졌다고 보고되었다.
- 수식 속성 대화상자의 기본 탭 값이 전부 빈값으로 출력되는 회귀가 보고되었다.
  - 한컴오피스 기준 수식 개체는 너비/높이, 글자처럼 취급, 기준 offset 등이 표시되어야 한다.
- 수식 속성 대화상자의 `여백/캡션` 탭 값도 누락되었다고 보고되었다.
  - 한컴오피스 기준 `바깥 여백`은 좌/우/위/아래 값이 표시되어야 하며, 수식에 캡션이 없을 때는 캡션 위치가 `없음`으로 표시되어야 한다.
  - 현재 Studio UI는 고정 문자열만 표시하고 `Equation.common.margin` 값을 JSON/UI에 연결하지 못하고 있다.
- 개체 속성/수식 속성 대화상자는 열린 상태에서 제목 영역을 마우스로 잡아 이동할 수 있어야 한다고 보고되었다.
  - 현재 대화상자 드래그 핸들러는 있으나, `dialog-wrap` 위치 지정이 고정되지 않아 `left/top` 적용이 브라우저에서 이동으로 이어지지 않을 가능성이 있다.
- 후속 시각 검증에서 12쪽 우측 하단 `문15)` 시작 전 간격이 한컴오피스와 다르다고 보고되었다.
  - 렌더 로그 기준 `pi=664` 마지막 수식 줄 뒤 `pi=665`(`문15)`)로 이어지는 약 80px VPOS 간격이 `compact_endnote_new_note_jump` 조건에서 억제되고 있다.

## 초기 분석

- 비교 대상:
  - 한컴/PDF 19쪽: `pdf/3-09월_교육_통합_2022.pdf` page 19
  - RHWP 19쪽 SVG: `output/task1139_stage26_probe/3-09월_교육_통합_2022_019.svg`
  - side-by-side 비교: `output/task1139_stage26_probe/compare_019_side.png`
- `dump-pages -p 18` 결과:
  - 19쪽 좌측 단: `pi=976..991`
  - 19쪽 우측 단: `PartialParagraph pi=992 lines=0..1`
- `dump-pages -p 19` 결과:
  - 20쪽 좌측 단: `PartialParagraph pi=992 lines=1..3`, `pi=993..994`, `문29) pi=995`
- 후보 원인:
  - `pi=992` split 뒤 남은 줄과 후속 `pi=993..995`를 같은 19쪽 우측 단에 배치하지 못하고 있다.
  - 19쪽 우측 단 첫 미주 문단의 VPOS rewind/height fit 계산이 실제 한컴보다 보수적으로 처리되는지 확인해야 한다.
  - Stage25에서 추가한 미주 가상 문단 줄별 VPOS 렌더 보정은 시각 Y 보정이며, 페이지네이션 fit 판정과의 불일치가 19쪽에서 드러났을 가능성이 있다.

## 진행 계획

1. 18/19/20쪽 `dump-pages`와 SVG를 다시 추출해 `pi=992` split 지점을 고정한다.
2. `pi=992..995`의 lineSeg `vertical_pos`, line height, TAC 그림/수식 높이, column available height를 진단한다.
3. `typeset`의 partial paragraph split 뒤 잔여 높이 계산과 `HeightCursor`의 rewind 허용 조건을 후보 코드 라인별로 좁힌다.
4. 문단 모양 기능 회귀 재현 경로를 확인하고, `getParaProperties`, `applyParaFormat`, 선택 범위 계산, 미주 가상 문단 해석 중 어느 경로에서 깨지는지 분리한다.
5. 19쪽 우측 단에 `문29) pi=995`가 들어가는지 자동 회귀 테스트를 추가한다.
6. 문단 모양 기능에 대한 회귀 테스트 또는 최소 수동 검증 절차를 추가한다.
7. 18/19/20쪽 SVG export 및 PDF side-by-side 비교 후 작업지시자 시각 검증을 기다린다.

## 구현 내용

- `typeset` 미주 분배에서 기본 미주 간격 문서의 내부 vpos rewind 문단은 pre-advance로 다음 쪽에 밀지 않고 같은 단에서 줄 단위 분할되도록 보정했다.
  - `pi=992`의 후속 줄과 `문29) pi=995`가 19쪽 우측 단에 남는다.
  - 20mm 미주 간격 변형 파일은 기존처럼 다음 쪽 분기를 유지하도록 기본 간격 문서 조건으로 제한했다.
- 미주 가상 문단 인덱스가 본문 문단 범위를 벗어날 때 `endnote_para_sources`를 통해 원본 Endnote 문단으로 역해석하도록 문단 모양 API를 보정했다.
  - `get_para_properties_at(0, 868)`이 미주 원본 문단의 문단 모양을 조회한다.
  - `apply_para_format(0, 868, ...)`이 원본 Endnote 문단에 적용되고 section rebuild를 수행한다.
- 20쪽 하단 overflow 완화를 위해 compact 미주 lazy vpos 흐름에서만 제한적 deep backtrack을 허용했다.
  - page-base가 확정된 흐름에서는 같은 보정을 적용하지 않는다.
  - 17쪽 `문30)`의 `pi=930`이 위 조건 줄과 겹치던 회귀를 차단했다.
- 10쪽 `문11)`처럼 기본 미주 간격 문서에서 새 미주 제목만 단 하단에 남는 경우, 다음 단이 있는 상태에서는 제목과 본문이 함께 다음 단으로 이동하도록 보정했다.
  - 내부 VPOS rewind가 있는 미주 묶음은 이 선행 이동에서 제외해 20쪽 `문23)`이 한컴/PDF처럼 좌측 하단에 남도록 제한했다.
  - 마지막 단 하단에서 새 미주 제목 직후 다줄 문단의 마지막 줄이 시각적으로 넘치는 경우에는 줄 단위 분할을 적용해 20쪽 `문27)` 하단 overflow를 제거했다.
- 수식 속성 조회/적용 JSON에 `Equation.common`의 공통 개체 속성을 포함했다.
  - `width`, `height`, `treatAsChar`, `vertRelTo`, `horzRelTo`, offset, 설명 등 기본 탭에서 필요한 값을 반환한다.
  - 수식 속성 적용 시 공통 개체 속성도 반영하되, 수식 내용/폰트 크기 변경 후 실제 수식 bbox 기준으로 너비/높이를 다시 계산한다.
- Studio 수식 속성 대화상자의 기본 탭 입력칸을 실제 수식 개체 속성과 연결했다.
  - 너비/높이는 HWPUNIT에서 mm로 변환해 표시한다.
  - `글자처럼 취급` 체크 상태와 가로/세로 기준 offset을 조회값으로 채운다.
- 수식 속성 조회 JSON에 공통 개체 바깥 여백과 캡션 없음 상태를 포함했다.
  - `outerMarginLeft/Top/Right/Bottom`을 `Equation.common.margin`에서 반환한다.
  - 수식에는 캡션 모델이 없으므로 `hasCaption=false`, `captionDirection=None`을 반환하고 Studio UI는 위치를 `없음`으로 표시한다.
  - `여백/캡션` 탭의 바깥 여백 입력칸을 실제 조회값으로 채운다.
- 개체 속성/수식 속성 대화상자의 제목줄 드래그 이동을 보정했다.
  - drag 시작 시 현재 화면 좌표를 `position: fixed`의 `left/top`으로 고정해, flex 중앙 정렬 overlay 안에서도 이동이 적용되도록 했다.
  - 공통 제목줄 커서를 move로 표시한다.
- 12쪽 우측 하단 `문15)` 시작 전 간격을 한컴 미주 사이 7mm에 맞춰 보존했다.
  - 새 미주 제목이 단 하단에 있을 때 전체 절대 VPOS 점프를 모두 복원하지 않고, `미주 사이`에 해당하는 gap만 렌더에 남긴다.
  - 나머지 절대 VPOS 점프는 기존처럼 base에 흡수해 13쪽 `문17)` 하단 overflow가 재발하지 않도록 했다.

## 회귀 테스트

- `issue_1139_page19_question29_starts_on_right_column`
  - 19쪽에 `PartialParagraph pi=992 lines=1..3`과 `문29) pi=995`가 있고, 20쪽에는 `문29) pi=995`가 없어야 한다.
- `issue_1139_endnote_virtual_paragraph_para_shape_api_uses_source_note`
  - 미주 가상 문단 `pi=868`의 문단 모양 조회/적용이 원본 Endnote 문단에 연결되어야 한다.
- `issue_1139_page17_question30_followup_lines_do_not_overlap`
  - 17쪽 `문30)` 제목, 조건 줄, `n(A)=2 또는 n(A)=3` 줄, `(i)` 줄의 y 간격이 겹치지 않아야 한다.
- `issue_1139_exam_2022_page_count_matches_hancom_after_endnotes`
  - 10쪽 `문11)` 제목이 좌측 단 하단에 남지 않고 우측 단에서 시작해야 한다.
- `issue_1139_endnote_equation_exposes_note_ref_and_properties`
  - 미주 내부 수식 속성 조회가 `noteRef`와 함께 기본 탭에 필요한 `width`, `height`, `treatAsChar` 값을 반환해야 한다.
  - `여백/캡션` 탭에 필요한 바깥 여백 값과 `hasCaption=false`도 반환해야 한다.
- `issue_1139_page12_question15_keeps_hancom_endnote_gap`
  - 12쪽 우측 하단 `문15)` 시작 전 gap이 한컴 미주 사이 7mm에 해당하는 24~32px 범위로 보존되어야 한다.
- `HeightCursor` 단위 테스트:
  - `compact_endnote_deep_backtrack_uses_vpos_near_column_bottom`
  - `compact_endnote_deep_backtrack_skips_page_path`
  - `compact_endnote_deep_backtrack_allows_safe_new_note_title`
  - `compact_endnote_deep_backtrack_skips_after_note_title`

## 검증 기록

- `cargo fmt --check`: 통과
- `cargo build`: 통과
- `cargo test renderer::height_cursor::tests::compact_endnote_deep_backtrack -- --nocapture`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 16개 통과
- `git diff --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `npm run build` (`rhwp-studio`): 통과
- `3-09월_교육_통합_2022.hwp` SVG export:
  - page 10/12/13/17/19/20에서 `LAYOUT_OVERFLOW` 로그 없음
- 최종 `dump-pages` 확인:
  - 10쪽: `문11) pi=557`이 우측 단에서 시작
  - 12쪽: `문15) pi=665`가 우측 단 하단에서 미주 사이 gap을 두고 시작하고 12쪽 overflow 없음
  - 13쪽: `문17) pi=708`이 좌측 단 하단에 유지되고 overflow 없음
  - 17쪽: `문30) pi=928..930`, `pi=931 lines=0..4`가 우측 단 하단에 배치되고 겹침 없음
  - 19쪽: `문29) pi=995`가 우측 단에 유지
  - 20쪽: `문23) pi=1054`가 좌측 하단에 남고, `문27) pi=1088`은 줄 단위 분할되어 하단 overflow 없음
- SVG/PDF 비교 산출물:
  - `output/task1139_stage26_remaining_fix_pages/compare_page12_pdf_rhwp.png`
  - `output/task1139_stage26_remaining_fix_pages/compare_page13_pdf_rhwp.png`
  - `output/task1139_stage26_remaining_fix_pages/3-09월_교육_통합_2022_012.svg`
  - `output/task1139_stage26_remaining_fix_pages/3-09월_교육_통합_2022_013.svg`
  - `output/task1139_stage26_compare_final/compare_10.png`
  - `output/task1139_stage26_compare_final/compare_17.png`
  - `output/task1139_stage26_compare_final/compare_19.png`
  - `output/task1139_stage26_compare_final/compare_20.png`
  - `output/task1139_stage26_verify4/3-09월_교육_통합_2022_010.svg`
  - `output/task1139_stage26_verify4/3-09월_교육_통합_2022_017.svg`
  - `output/task1139_stage26_verify4/3-09월_교육_통합_2022_019.svg`
  - `output/task1139_stage26_verify4/3-09월_교육_통합_2022_020.svg`

## 해결된 부분

- 19쪽 `문29)`이 20쪽으로 밀리던 문제를 보정했다.
- 17쪽 우측 하단 `문30)` 후속 줄 겹침과 overflow를 보정했다.
- 10쪽 `문11)` 미주 사이/단 분배 회귀를 보정했다.
- 12쪽 우측 하단 `문15)` 시작 전 미주 사이 7mm gap을 보존했다.
- 13쪽 `문17)` 하단 overflow 재발을 막았다.
- 20쪽 하단 `문27)` 마지막 줄 overflow를 줄 단위 분할로 제거했다.
- 미주 가상 문단의 문단 모양 조회/적용 회귀를 보정했다.
- 미주 내부 수식 속성 기본 탭 빈값 회귀를 보정했다.
- 수식 속성 `여백/캡션` 탭에서 바깥 여백과 캡션 없음 상태를 표시하도록 보정했다.
- 개체 속성/수식 속성 대화상자 제목줄 드래그 이동을 보정했다.

## 미해결 및 Stage27 이월

- 작업지시자 시각 검증에서 22쪽 렌더링 자체가 한컴오피스와 다르다고 보고되었다.
  - 22쪽은 단순 overflow가 아니라 상단 표/그림/미주 흐름의 전체 렌더 배치가 다르다.
  - Stage26의 10/12/13/17/19/20쪽 보정과는 별도 원인일 가능성이 높으므로 Stage27에서 새 기준으로 분석한다.
- Stage15에서 남긴 9쪽 격자 원점 판단은 유지한다.
  - `--grid-origin=9mm,24mm`와 `--grid-origin=auto`의 SVG pattern 차이는 약 0.002px 수준이라 남은 미세 차이는 격자 원점 문제가 아니다.
  - 다음 후보는 문단 내부 수식/텍스트 렌더 메트릭, 수식 TAC baseline/bbox 스케일, 문단 세로 정렬(`attr1` bit 20~21, 글꼴 기준) 처리다.

## 승인 상태

- 2026-05-30: Stage26 시작 지시를 받았다.
- 2026-05-30: 19쪽 `문29)` 분배, 미주 가상 문단 문단 모양 회귀, 17쪽 `문30)` 겹침, 10쪽 미주 사이 분배, 수식 속성 기본 탭 빈값 회귀 보정을 구현했다.
- 2026-05-30: 잔여 항목인 수식 속성 `여백/캡션` 값 표시, 개체/수식 속성 대화상자 드래그 이동, 12쪽 `문15)` 시작 전 미주 사이 gap을 추가 보정했다.
- 2026-05-30: 작업지시자가 Stage26 커밋 후 Stage27 문서 생성 순서를 지시했다.
- Stage26은 위 해결 범위로 마감하고, 22쪽 렌더링 차이는 Stage27로 이월한다.
