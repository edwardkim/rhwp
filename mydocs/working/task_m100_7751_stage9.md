# #7751 통합 회귀: 저장 문단 끝의 원점 유효성 — 9단계

시작 HEAD `d06b3e03c`. 기준 `upstream/devel`은
`fe5725dc384689b45d60ef8ea4c83138bc2701d8`이다.
8단계의 WMF crop·PNG 색조 검사 보정을 커밋한 뒤 시작한다.

## 구현 전 독립 근거

전체 Rust의 `text_overlap_baseline::text_overlaps_do_not_grow_partition_4`가
`hwpspec.hwp`의 신규1건을 검출했다. 같은 원본의 기준 Native 이상 탐지는0건,
8단계 Native는1건이다. 현재57쪽에서 구역2·문단346 제목 “그 외의 컨트롤”과
문단347 표의 “의미”가 겹친다. 해당 원본의 기존 문제 추적 #7772가 있지만,
이번 신규 겹침은 이관하지 않고 구현을 고친다.

- 기준 출력: `pdf/hwpspec-2024.pdf`178쪽. 대응 내용은 PDF54쪽이다.
  `pdftotext -f 54 -l 54 -bbox-layout`에서 제목497.676pt,
  표 머리글525.087pt로 앞뒤 순서와 비겹침이 독립적으로 확인된다.
- 현재 제목의 실제 글줄687.587–702.920px, 다음 표 머리글697.340–709.340px.
  이상 탐지의 실제 텍스트 교차는9.073×5.580px다.
- 원본 제목 LineSeg는vpos42254/lh1150/ls688HU, 다음 host vpos44592HU다.
  수동 생성 입력이 아닌 저장 HWP를 그대로 사용했다. 앞 표의 물리 점유가 달라지면
  기존 절대 저장 축을 제목 끝으로 계속 사용할 수 있는지 별도로 확인해야 한다.

증거는 ignored `output/pr-review/pr7494-20261016/`의
`hwpspec-{base,head}-anomaly.json`, `hwpspec-source-p{345,346,347}.log`,
`logs/hwpspec-head-flow.log`, `hwpspec-pdf54-bbox.html`에 있다.
기준 전체179쪽/현재181쪽이며 독립178쪽과 일치하지 않는다. 이번 진단을
문서 전체 피델리티 완료로 보고하거나 신규 실물 px 회귀의 근거로 사용하지 않는다.

## 생산·소비 경로와 원인

1. `float_placement::stored_paragraph_successor_flow`는 원본 LineSeg와 다음 host의
   저장 간격 등식을 확인해 `StoredParagraphSuccessorFlow`를 만든다.
2. `typeset/paragraph::stored_successor_flow`가 현재 단의 base로 이를 투영한다.
   현재 `project`는 문단 시작이 저장 **내용 하단**을 넘지 않는지만 확인한다.
   현재 시작이 이미 저장 글줄 시작을 지났어도 짧은 문단이면 잘못 수용한다.
3. `place_fitted_paragraph`는 그 끝으로 흐름을 예약하고
   `stored_paragraph_flow_ends`에 기록한다.
4. `layout_column`은 실제 배치 뒤 이 끝으로 문단을 닫는다. 진단에서 문단346은
   실제 글줄 하단702.920px보다 위인686.067px로 닫혔다. 다음 표가 그 끝을
   소비해 앞 제목과 교차한다.

끝을 실제 bbox로 clamp해 가리지 않는다. 저장 축의 수용 조건에 해당 문단의
원본 시작(앞 간격 포함)을 추가해, 실제 시작이 이미 원본 시작보다 전진한 경우에는
그 저장 끝을 사용하지 않고 기존 순차 측정·배치가 같은 흐름을 계속 소유하게 한다.
정상 저장 축과 column/lazy base의 계약은 유지한다. 특정 문서·쪽·상수 분기는 없다.

## 검증 계획

- 생산 결과의 정상 원점/후속 간격, 잘못 이동한 시작, base 없는 경우를 구분한다.
- 기존 실제 겹침 partition의 수정 전FAIL을 보존하고 수정 후 실행한다.
  baseline 증가·검사 제외로 새 겹침을 숨기지 않는다.
- 편람의 기존 완료 경계·앞 표 종료/다음 제목/다음 표 관계, 원본 대조군을 재실행한다.
- Native/fresh WASM 영향 페이지의 동일 입력·독립 PDF를 확인한다. 기존 전체
  hwpspec 미달은 그대로 기록하며 현재 신규 회귀의 제거와 전체 일치를 구분한다.
- 전체383쪽 검증·최종 lint와 필수 Skia/Studio 검증은 최종 코드에서 판정한다.
  224·226쪽 devel 대비 하락과 나머지 기존 실패의 처리는 별도로 남아 있다.

## 8단계의 후속 완료 결과

Native383쪽도 산출 완료: 최저90쪽92.53354%,90% 미만/누락0쪽.
WASM은92.45371%로 양쪽383/383이다. Decimal 비교의 남은 보류는
Native224쪽−0.01876%p,226쪽−0.01823%p 및 WASM226쪽−0.03393%p다.
`stage8-all-pages-base-head-comparison.json`에 실제 결과를 보존했다.
WMF3/완료 경계5/HWPX 저장2/Skia placeholder2/direct PDF14 모두 PASS다.
초기 잘못 지정한 `png_backend_placeholder`는 없는 case여서 exit1이었으며 통과로
세지 않는다. 정식 `issue_2225_missing_picture_placeholder`2건을 별도 실행해 확인했다.

## 9단계 구현과 선행 진단

공통 frame에 원본 `첫 LineSeg vpos−문단 앞 간격`의 흐름 시작을 보존했다.
`project`는 현재 시작이 이 저장 시작보다 실제로 전진했으면 저장 끝을 거절한다.
1HU 이하 차이만 단위 변환 반올림으로 수용한다. 끝을 bbox에 clamp하지 않았고
같은 판정을 조판의 예약과 paint의 저장 끝 기록에 사용한다.
정상 단/lazy base 투영, 이동한 시작, 미해결 base의 source-side 수치 검사3건을
시도했으나 기존 source test 총량/신규 cfg(test) 모듈 금지 정책에 걸려 철회했다.
`logs/stage9-unit-tier-check.log`의 실패를 통과로 세지 않는다. 새 수치 검사·모듈은
제품에서 제거했고 실제 기존 겹침 partition의 수정 전FAIL/수정 후 실행으로 검증한다.
원래 함수에서 제거한 것은 비활성 cfg(test) 선언뿐이다. Native/WASM의 비검사
생산 함수 바이트 동일성은 `stage9-cfg-test-removal-equivalence.json`에 보존했다.

Native CLI 빌드 exit0,4분53초. 고정 실행 파일의 원본 `hwpspec.hwp` 전쪽
이상 탐지에서 text-overlap1→0, 전체 overlap5→0, overflow0을 확인했다.
전체181쪽은 변하지 않았다. 기존178쪽 독립 PDF와의 전체 피델리티 완료는 아니다.
정식 기존 text-overlap partition의 실행은 아직 대기 중이다.
편람3/130/224/226/241/290/297/375쪽 Native는 모두90% 이상이며 원래 영향 쪽
검증을 먼저 했다. 독립한컴 PDF와 같은 입력·5개 글꼴 경로를 유지했다.

복학원서는 새 Native에서도1쪽80.43535%로 base와 동일하다. 열린 #7756의
동일 원본·미해결 범위와 실제 실패 overlay 함수1건을 연결해 assets에 본문과
review/overlay를 보존했다. 정상 SVG 함수·공용 입력/PDF/golden은 유지했다.
최종 fresh WASM 확인과 유지 검사 실행은 아직 대기 중이다.

HWP3 암호 문서의 배경 불투명도 검사는 첫 쪽만 검사한다. 기존 PDF의 SHA는
#3486 조사 기록과 같고 실제 소스 BMP3개와 PDF 이미지가 대응함을 확인했다.
8단계의 동일 WASM에 공개 fixture 비밀번호만 전달하는 인증 adapter를 사용했고
WASM 바이트와 원본 입력은 바꾸지 않았다. 관련1쪽 raw는 Native91.77401%,
fresh WASM92.70803%다. review에서 배경의 실제 색·불투명도를 대조했다.
독립 PDF JPEG와 원본 BMP를 대조한 encoded RGB 모델 MAE1.89806은
옛 linear SVG 모델12.52267보다 작았다(`hwp3-page-background-pdf-color-models.json`).
따라서 기존 filter ID 요구를 실제 PNG의 원본 크기·불투명 alpha·원색 대비
밝아진 픽셀 보존 검사로 바꾼다. 기존 raw 속성(-15,50)과 crypto 계약도 유지한다.
제목·글꼴·차례의 알려진 차이는 그대로이며24쪽 전체 개선으로 보고하지 않는다.

## 정식 회귀 결과

`stage9-focused-command.json`의 manifest 동적 해석으로5개 suite를 선택했다.
PNG payload3, 편람 완료 경계5, HWPX 저장2, HWP3 계약11,
복학원서 유지 SVG1, 임대차 유지3, 겹침 partition1을 포함한 실제 결과는
`logs/stage9-focused-tests.log`의26PASS/0FAIL이다. 세부 함수 수는 로그의
실제 목록을 기준으로 하며 이 합계보다 넓은 범위가 실행됐다고 하지 않는다.
이 실행은 source-side 검사3건을 철회한 상태의 최종 생산 코드에서 컴파일했다.
기존 전체10467 실행에서 실패한9건은4건을8단계에서 보정했고, 이번 단계에서
HWP3/신규 겹침2건을 보정하고 복학원서1/임대차2건을 등록 이슈로 이관했다.

임대차 이관은 #7955를 생성해 #7445의 하위 이슈와 본문 표, 담당자를 실제로
등록하고 API로 재확인했다. 현재94개 연결/OPEN93개/추가 여유6개다.
테스트·전용 상수2건만 원문과 함께 assets에 보존했고 정상3함수와 공용 입력/PDF는
유지했다. 새 px 기대값으로 바꾸지 않았다. 복학원서까지 이번 활성 검사 감소는3건이다.

9단계 fresh WASM 빌드 exit0,278.65초이며 실제 WASM과 wrapper를 동결했다.
삭제한 cfg(test) 모듈을 제외한 생산 코드 동일성을 별도 증명했다.
전쪽 Native/fresh WASM383쪽 재검증과 최종 lint는 현재 진행/대기 중이다.
