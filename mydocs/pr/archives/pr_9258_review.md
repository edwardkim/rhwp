# PR #9258 검토 — 한글 97 안내문과 다섯 조판 회귀 복원

[PR #9258](https://github.com/edwardkim/rhwp/pull/9258)은 `jangster77`의 collaborator 자가 검토 PR이다.
비교 기준 제품 소스는 `411a9f944cd6ff202e1f8c5b835ecccbc837486f`다. GitHub의 코드 후보
`b8ca0f9202f9e40a174d67b81732b431ba3fee94`에서 필수 검사를 완료했다.

## 수정과 재현

- 한글 97 안내문 12쪽의 마지막 문단에서 공백별 절삭이 누적돼 `URL`이 둘째 줄로 넘어갔다.
  HWPUNIT의 100분의 1 단위로 한 줄의 압축 예산을 합산한 뒤 정수로 비교한다. 줄바꿈과
  비분배 줄의 공백 배치가 같은 한도를 사용한다. 오른쪽 여백과 공백 압축률을 바꾼 독립
  한컴 2024 Print 6건에서 `URL`의 줄 소속과 폭 경계를 확인했다.
  [구현 근거](../../working/task_m100_7744_stage2.md) ·
  [독립 인쇄 경계](../assets/issues7744_7748_20261120/issue7746/url-fit-boundaries.json).
- 다섯 원본의 보류 회귀를 복원하고 안내문 12·13쪽의 줄 소속과 내용 이어짐 검사를 보강했다.
  12쪽의 첫 줄 끝 `URL`, 13쪽 첫 문장, 15쪽 원고지 항목을 비교했다.
  Native CLI의 Save PDF도 direct 및 SVG 경로에서 확인했다.
  [전쪽 TSV와 대표 비교 이미지](../assets/issues7744_7748_20261120/README.md).
- 같은 원본의 독립 Print 기준으로 총 57쪽씩 Native와 최적화 fresh WASM 전체를 측정했다.
  최저값은 Native 98.29106%, fresh WASM 98.22386%다. 전직시험면제 표는 Print에서 사용한
  한컴바탕 TrueType을 공급한 조건에서 두 경로 모두 6쪽 전체가 100.00000%였다.
  재활용 보도자료는 괄호와 숫자가 보존된 한컴 2020 Print를 기준으로 삼았다.
  한글 97 안내문 1쪽은 설명 줄 시작과 `Enter` 표지 표시 차이가 남아 있다.
  [기준 PDF와 원본 SHA-256](../assets/issues7744_7748_20261120/print-references.json).

## 검증

- 최소 글꼴 집중 57건과 전체 Nextest 13,849건, Native Skia 라이브러리 4,035건과 그림·직접 PDF
  38건이 통과했다. Rust lint, workspace 빌드, 편람 Native 383쪽, 공통 보호 8조건도 통과했다.
  [로컬 실행 영수증](../assets/issues7744_7748_20261120/local-validation.json).
- Ubuntu에서 정본 보호 46조건 4,046쪽을 전수 측정했다. 정본 표시값에 못 미친 77쪽을
  `upstream/devel`과 같은 입력·PDF·글꼴·출력 경로로 다시 비교했다. 해당 77쪽 모두의
  후보·기준 PNG SHA-256이 일치했다. 이 비교 결과를 근거로 작업지시자가 정한 진행 조건을
  확인했다. [페이지별 기준·후보 표시값](../assets/issues7744_7748_20261120/protected-comparison.tsv) ·
  [정본 미달 쪽의 동일 호스트 재검증](../assets/issues7744_7748_20261120/protected-comparison.json).
- `298be1744ca4a73a668c9108c3e755a7a2848d8c`까지 전진한 `upstream/devel`의 Visual Sweep 변경은
  Pillow 실행 정보 기록에 한정된다. 바뀐 도구를 후보의 등록 원본·PDF와 대조해 alpha·재개·쪽수 기준표
  검사 33건이 통과했고, 복원한 중첩 표 17쪽과 전직시험면제 표 6쪽의 독립 기준이 재생성 입력에
  포함됨을 확인했다.
- 정확한 코드 후보에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/44759345868) ·
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44759345705) ·
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44759345682) ·
  [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/44759345784) ·
  [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/44759345773) ·
  [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/44759345331)가 성공했다.

최신 기록 커밋의 필수 검사를 확인한 뒤 병합하고 #7744–#7748을 정리한다.
