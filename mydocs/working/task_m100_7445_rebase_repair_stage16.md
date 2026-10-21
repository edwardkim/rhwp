# 단계16: 원본0 크기 텍스트의 공백 전진 보존

## 원 PR과 현재 실패

`issue1948_cross_para_fieldend.hwpx`는 원 PR#1963/커밋0a0822c3b1,
merge2e1c6929ac에서 교차 문단 fieldEnd의8유닛 갭을 말미 표가 가로채던 HWPX 저장 순서를
보호하려고 추가했다. 원 이슈·PR 본문/댓글·실제 커밋을 확인했다. 현재 실패는 그 저장 순서와
별개인 실제 조판 회귀이며 serializer와 원본 fieldEnd/표 앵커는 변경하지 않는다.

고정4a07 Native는offCanvas0/전1쪽98.03372%,현재 후보는offCanvas1/Native95.79258%,
fresh WASM95.81623%였다. 원본 문단10의글자모양14는`base_size=0`이고, 개인정보 안내문
텍스트168자는 원문에 남되 그림·글리프 전진을 만들지 않아야 한다. 그러나 실제0크기 run의
폭이214.428746px로 늘고 앞 본문까지 압축됐다. 원본은 보존했고 기존 anomaly 검사를 완화하지 않았다.

## 원인과 공통 보정

로컬 커밋35f392439d8의`reflow_space_font_size`는 첨자 뒤 공백을 본문 em으로 유지하면서
`font_size<=0`이면12를 반환했다. 미해소 기본 스타일과 원본에서 명시적으로 해소한0을 혼동했다.
공통 `TextStyle::effective_font_size()`로 바꿔 해소된0은0,미해소 기본값은12를 보존한다.
첨자의 공백을0.7배로 줄이지 않는 기존 본문 em 계약도 유지한다.

이 helper는 `char_width_decision`의 반각 공백과 `hancom_regenerated_space_width`의 재조판
공백 양쪽에서 소비된다. 수정 전0크기 공백이 만드는 폭을 줄 채움과 최종 run 측정이 함께
소비해 앞 본문을 압축했고, 같은 run 상자가 진단의 심층 가로 합집합에도 들어갔다.
수정 후 공통 측정에서0을 보존해 실제 본문 원점과 진단이 함께 복구된다. 출력 좌표 clamp나
안내문 삭제, 문서/페이지/문구 조건은 추가하지 않았다.

## 같은 원본 전쪽 결과

기존 유효한 독립 한컴 Print `pdf/hwpx/issue1948_cross_para_fieldend-hwpx-2020.pdf`를
재사용했다. 실제 Creator는Hwp2022이며 파일명과 생성 제품을 구분한다. 원본/PDF를 교체하지 않았다.
canonical192DPI/compat2022/PDFium smooth_rgb_v1/raw2px90%로 각 backend 전1쪽을 다시 산출했다.

| 경로 | 수정 전 | 수정 후 | 독립 쪽수 |
|---|---:|---:|---:|
| Native |95.79258%|98.73123%|1/1|
| fresh WASM |95.81623%|98.73890%|1/1|

수정 후 Native는fixed base보다0.69751%p 높고offCanvas는1→0이다. 같은 쪽의 Native 수정 전후와
fresh WASM review/overlay를 직접 판독했다. 붙임2 본문과숫자·끝 문구의 원점이 복구됐고,
원본0크기 안내문은 모델에 한 번 남는다. 위 본문의 미세한 글자/간격과 소방서 마크의 잉크 차이는
남으며 완전 픽셀 일치라고 주장하지 않는다.

## 실제 회귀와 소비 경로 확인

- 새 정식 case `issue_7445_zero_size_stored_text`의2검사는 명시적0 크기의 글자·일반/비분리 공백
  전진0,미해소 기본값의 정상 전진,16px 첨자에서 본문 반각8px 공백을 함께 검사한다.
- 실물 검사는 원본0크기 스타일·1쪽 소유·원문 scalar 내용 보존·0폭 안내문 한 번 소유와
  실제 본문의 보존을 검사한다. 해소된0과 기본 스타일을 구분하지 않으면 공개 측정과 실물 양쪽이 실패한다.
- 전쪽 Native/fresh WASM 양쪽98% 이상·직접 판독 후, 원문 문단10/char7의 실제 붙임 숫자2를
  독립 Print 원점x131.1289333333/기준선679.417492CSSpx±2로 보호했다. 고정 영역으로 대상을
  선택하지 않는다. 강화한 최종 검사도 frozen540에서2FAIL→frozen552에서2PASS다.
  수정 전 숫자x125.9697859는 올바른 독립 원점에서5.159px 어긋나 실제로 실패했다.
- 정식 현재 manifest 묶음은32실행31PASS/1FAIL,1697선택 제외,실행46.791초/전체323.056초다.
  off_canvas partition15와 이번2검사·캡션/원본 수식/쪽번호 대조군은PASS다.
  남은1FAIL은 `issue_1105::task1105_sample16_hwp5_2022_server_requirements_page_matches_hancom`이다.
- px를 강화한 최종 source의 정식2검사 재실행도2PASS/258선택 제외/exit0이다.
  앞 묶음과 중복되는 검사 수를 합산하지 않는다. 최초 standalone의 존재하지 않는
  `measure_text` 호출 오류는 보존하고 실제 컴파일·실행한 결과만 최종 근거로 쓴다.
- Native552 build139.656초/exit0, fresh WASM554 build215.518초/exit0이며 source hash가 같다.
  Mac locked wrapper의web/pkg/no-opt 로컬 대체 빌드다. Docker 최적화 통과가 아니다.
  root pkg·Studio public hash와 실제 Studio7736 JS/WASMGET200/캐시 미사용 응답을 확인했다.
  첫 중복 JS 응답 body 관측 오류를 보존하고 깨끗한 브라우저 재시도에서errors0/초기화true다.
  이 초기화 검사는actual36의 문서 paint/글꼴/CanvasKit 수용이 아니다.

[전쪽 TSV·수정 전후 review/overlay·원본/PDF/빌드 출처·검증 정본](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/zero-size-space/verification.md)

[정식 묶음 로그](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair555-zero-size-focused-gates/nextest.log)

[최종 px 검사 로그](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair559-zero-size-px-gates/nextest.log)

## 계속 보정할 범위

Science 원본 전4쪽을같은96DPI로 재검증했으며 최저93.11518/93.04014%,3쪽94.73771/94.73462%로
직전 결과와 같다. Native3쪽의fixed base95.06581% 대비0.32810%p 하락은 여전히 보류 사유다.
`#1105`의올바른 Heart Bit/구성은 경계, 저장 issue265의 캡션 번호 차이,
최종 전체 회귀·범위 게이트·전체 시각/actual 검증도 남아 있으므로 전체 PR 완료가 아니다.
임시 진단·로그는 output에만 보존하고 PR은 계속 보류한다.
