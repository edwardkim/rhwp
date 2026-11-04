# PR #8319 — Native Skia의 PDF 표 글자 가시성 검사 보정

## 관측과 선행 분석

제출 head `f528dba6d974df25aa2b28891e0a1ad24da47624`의
[Native Skia job](https://github.com/edwardkim/rhwp/actions/runs/40478847335/job/122104043583)에서
`issue_7950_pdf_paint::inferred_table_cannot_cover_its_labels_with_retained_positional_paint`가
실패했다. 같은 소스의 로컬 검사는 4 PASS였지만, DejaVu만 설치한 CI 조건을 bubblewrap의
읽기 전용 글꼴 마운트로 재현하자 같은 왼쪽 아래 셀에서 3 PASS / 1 FAIL이었다.
호스트의 설치 글꼴·fontconfig 설정을 변경하지 않았다.

원 해결은 [PR #7953](https://github.com/edwardkim/rhwp/pull/7953), merge
`b7a5388034ee66f58a7e1d5363974e83b6374f57`이다. 원래 검사와
[stage7](task_m100_7949_stage7.md)의 원본 PDF·표 승격·paint plane 근거를 대조했다.
목적은 positional 배경이 표 글자를 가리는 실제 결함을 검출하는 것이다. 글꼴의 절대 획 농도는
그 계약이 아니다. 원 PDF는 검은 글자와 흰색 또는 파란색 셀 배경을 독립적으로 정의한다.

정상·최소 글꼴 환경의 저장 HWPX와 render tree는 같았다. 최소 환경의 글자는 실제로 보이며,
흰색 셀의 가장 어두운 값은 95였다. 기존 `R/G/B < 80` 조건은 이를 글자 없음으로 오판했다.
같은 셀의 배경보다 어두운 글자 픽셀은 100개였다. 현재 Native와 독립 Poppler의 한 쪽 raw는
정상·최소 환경 모두 100%이며, 이 수치와 별도로 H1/H2/B1/B2의 표시를 review·overlay에서 판독했다.

## 보정 범위와 독립 기대값

제품 renderer·글꼴 선택·표 추론·저장 속성·좌표를 바꾸지 않는다. 실패한 원 회귀가 사용하는
공개 PDF 세 개만 Native/fresh WASM과 고정 base에서 다시 확인한다. 다른 실제 HWP로 범위를
확장하지 않는다. 원본의 셀 내부 영역·글자/배경·한 쪽 소속·일반 표 추론 조건은 유지한다.

검은 글자가 원본 배경에 합성되면 배경의 0이 아닌 RGB 성분은 모두 감소하고, 0인 성분은
0으로 유지된다. 이 원본 색상 관계로 가시 잉크를 검사하며 임의 밝기 80을 다른 상수로
완화하지 않는다. 같은 배경만 남은 영역과 투명 픽셀은 글자 잉크로 세지 않는다. 최소 글꼴과
정상 글꼴 양쪽에서 실제 CLI→저장 HWPX→PNG 검사를 실행하고, CI의 Native 명령 목록 전체를
순차로 확인한다.

이전 로컬 전체 default-feature nextest와 Skia lib 통과는 해당 실행의 사실로 보존한다.
Native 전용 CLI 가시성 검사가 그 실행에 포함됐다고 쓰지 않는다. 누락된 필수 Native 검사를
보완하고 수정 뒤 종료 코드·같은 원본의 시각 결과·새 head CI를 확인한 뒤 병합한다.

원문 로그와 환경 재현·픽셀 측정·전체 TSV는 ignored
`output/pr-review/semanticist21-20261019/`의 `ci8319-*4795*`~`pdf-paint-scope-4801*`에 있다.
## 검사 보정과 실제 확인 결과

`tests/cases/issue_7950_pdf_paint.rs`에서 `<80` 밝기 핀을 제거하고, 검은 글자가 원본의
흰색·파란색 배경을 어둡게 하는 관계로 검사했다. 원본 셀 내부, 배경 보존, 한 쪽 소속과
일반 표 추론 검사는 유지했다. 제품 source와 기존 baseline·golden은 변경하지 않았다.

- 같은 공개 PDF 3개를 고정 base/후보의 Native/fresh WASM으로 다시 실행했다.
  12개 대조 모두 전체 한 쪽·raw 100%이며, base 대비 하락은 없다.
  후보의 6개 review·standalone overlay를 실제 열어 네 셀의 글자·배경·일반 표를 판독했다.
- 보정한 실제 CLI→저장 HWPX→PNG 검사: 정상 환경 **4 PASS**, 최소 글꼴 환경 **4 PASS**.
- 같은 새 검사 실행 파일의 실제 PNG에서 셀 글자를 원본 배경으로 가린 음성 대조는
  `visible label missing`으로 **FAIL(exit 101)**이었다. 가림 검출 계약을 유지했다.
- 최소 환경 재검증의 첫 실행은 prepare 전의 이전 suite 실행 파일을 가리켰다.
  현재 manifest가 선택한 `regression_suite_015`의 새 실행 파일로 바로잡아 위 결과를 확인했다.
  이전 실행의 실패를 새 검사 실패로 해석하지 않는다.
- 사용자 요청에 따라 여기까지 확인한 보정을 추가 커밋한다. CI Native 명령 목록 전체와
  테스트 수정의 필수 Rust lint는 계속 실행 중이며, 새 head push·CI 완료로 기록하지 않는다.

[원문 TSV와 대표 PNG](../pr/assets/s21_20261027/ci_pdf_paint/README.md)를 정식 증적으로 보존했다.
실행 로그는 ignored `output/pr-review/semanticist21-20261019/logs/ci-native-fix-4806.log`,
`native-pdf-paint-{normal,minimal}-4806.log`, `ci-pdf-paint-occlusion-negative-4807.log`다.
이 테스트 전용 보정 뒤 전체 제품·Studio 검증을 새 실행한 것처럼 표시하지 않는다.
