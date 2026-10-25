# #8153: 가운데 정렬 fit과 마지막 가시 글줄의 점유 계약

## 선행 분석과 원인

같은 원본·독립 2024 Print를 사용한 고정 `7620cc75` Native 전 766쪽에서 HWP 320쪽,
HWPX 317쪽이 raw 100%·미대응 0픽셀이다. 이전 기호 보정 producer `5bcb654516` 대비
미대응 증가 목록은 두 형식 46쪽의 각 17픽셀이다. 전쪽 100%나 제출 준비 완료가 아니다.

HWP 5.0 revision 1.3의 문단 모양 속성 1은 정렬을 bit 2~4, 공백 최소값 0~75%를
bit 9~15로 구분한다. 같은 공백 최소값이 모든 정렬에서 같은 실제 paint 배정을 뜻한다고
사양에 없는 해석을 추가하지 않는다. 사양의 값·원본 LineSeg와 독립 Print에서 구분한
줄 수용·실제 fit 계약을 따른다. 공식 원문 PDF와 입력은 수정하지 않는다.

- 46쪽의 실제 원본은 paraPr860, CENTER, condense70, charPr342다. 독립 Print의
  한글 전진은 약 9.44~9.60px, 내부 공백은 약 4.64~4.80px다. 기존 정상 출력은 이
  구간과 대응하지만 `7e8e5e9e02`에서 넓힌 공백 전용 계획은 내부 공백에만 압축을
  배정한다. 원래 가운데 정렬의 공통 glyph fit을 보존해야 한다. 문서 ID·원문 숫자·
  특정 문자 조합으로 조건을 좁히거나 별도 보정 상수를 넣지 않는다.
- 373쪽은 `5bcb654516`과 `7620cc75`의 전체 Native RGB가 같다. paraPr1381의
  JUSTIFY·condense0 마지막 줄이며 원래 세 글자모양과 말미 공백이 있다. 원 KoPub
  Batang Light의 한글 hmtx는 936/1000em이다. 실제 원점은 독립 PDF에서 추출했고,
  같은 후보의 SVG 소비 경로에서도 누적 압축이 확인됐다. 가시 내용은 약 294.5067px로
  실제 가용 폭 약 295.25px 안에 들어간다. 양쪽 정렬 측정은 말미 공백을 제외해 맞다고
  판정한 뒤, 일반 fallback fit이 공백을 다시 세어 약 0.1579167px씩 글자를 압축한다.

두 원본의 해당 쪽과 같은 증상을 검사하는 기존 #8153 검사는 아직 없다. 기존 148·327쪽의
음수 Justify 압축, 179쪽의 가운데 정렬 terminal tracking과 200쪽의 LEFT 공백 압축은
실제 독립 근거와 76건 통과 기록을 갖고 있으므로 그대로 보호한다. 정답인 검사를 낮추지 않는다.

## 공통 생산·소비 경로와 보정 범위

46쪽은 `원본 정렬·condense → 공백 압축 계획 선택 → 같은 glyph fit·정렬 폭 → 실제
positions와 다음 run pen → Native/Canvas`를 확인한다. CENTER를 공백 전용 계획으로
확장한 가정을 제거하고 기존 가운데 정렬의 공통 fit을 유지한다. LEFT의 확인된 공백
예산을 되돌리거나 RIGHT의 현재 계약을 이 작업에서 추측으로 확장하지 않는다.

373쪽은 `원본 run·말미 공백 소유 → 기존 가시 줄 측정 → 비분배 fallback의 fit 폭 →
실제 positions`에서 같은 점유를 소비하게 한다. 기존 음수 Justify의 분배 계약도 보호한다. 원문 공백과
저장 char 범위·positions를 삭제하는 방식으로 해결하지 않는다. 별도의 x 보정이나
backend별 폭 계산을 넣지 않는다.

말미의 혼합 글자모양은 각 run의 원래 메트릭으로 측정한다. 개체 앞 공백은 내부 공백이고,
밑줄·장식이 있는 공백과 원자적 표시 필드의 공백을 일반 비가시 말미로 간주하지 않는다.
선두 공백·필드·겹침 글자·탭의 원래 소유와 보호 계약을 확인한다. 정말 가시 내용이 넘치는
148·327쪽의 기존 Justify 압축과 soft-wrap 구분 공백·장식선은 반례로 실행한다.

## 실제 재현과 검증

초안은 ignored 작업 자료에만 준비했으며, 표준 WASM producer가 공용 target 사용을 마친 뒤
고정 `7620cc75` rlib로 두 원본 각각의 실제 FAIL을 먼저 확보한다. 46쪽 검사는 동일한
초안을 고정 `5bcb654516`에서도 실행해 기존 정상 동작을 확인한다. 준비한 초안이나 SVG
소비자 진단을 정식 회귀 통과로 세지 않는다.

확인 뒤 정식 파일은 `tests/cases/issue_8153_centered_condense_fit.rs`와
`tests/cases/issue_8153_justified_visible_last_line.rs`로 등록한다. 같은 글줄의 원문·세부 run
순서·셀/문단 소유를 보호하고, 독립 Print의 실제 글자·공백 전진 구간으로 검사한다.
구현의 계산값이나 helper 반환값을 기대값으로 재인용하지 않는다.

기존 76건, 새 네 검사, TAC 앞 공백·가운데/오른쪽 말미 공백·합성 soft-wrap의 정상 대조를
먼저 실행한다. 영향 목록에 46쪽을 더해 두 형식 Native/fresh WASM을 직접 비교한다.
현재 producer 전쪽 TSV와 정상 출력의 해시를 보존하고, 새 producer 전 766쪽씩도 같은
원본·PDF·DPI·글꼴로 검증한다. 새 증가가 있으면 전쪽 100%·미대응 0 목표까지 공통
원인을 계속 보정한다. 최종 Rust·lint·Skia·Studio·표준 fresh WASM 필수 검증은 별도로
완료해야 한다.

작업 자료는 `output/pr-review/issue8153-handbook-raw100-20261022/`의
`remaining-raw100-diagnostic/p046-current-regression/`,
`remaining-raw100-diagnostic/p373-source7620-line/`,
`handbook-final-visible-fit-candidate/`, `handbook-centered-condense-regression/`에 있다.
기존 기준 PDF를 재사용하며 private 글꼴·프로그램 분석 자료는 output에만 둔다.
사용자 지시대로 upstream/devel은 동기화하지 않는다. 모든 1,532조건의 raw 100%·미대응
0과 필수 테스트를 완료한 뒤 PR 준비까지만 하고 대기하며 원격 게시·push·PR 생성은 하지 않는다.
