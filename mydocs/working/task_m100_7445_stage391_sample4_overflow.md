# #7445 첫 전체 회귀 실패: sample4 본문 넘침

## 사전 분석

- 전체 회귀 36개 실패 중 첫 항목인 body-overflow partition 10을 먼저 처리한다.
- 대상은 `samples/hwp3-sample4-hwp5.hwp`, 시험지가 아닌 리눅스 어셈블리 36쪽 문서다. 같은 분할에서 이 문서만 4 → 6건으로 증가했다.
- 기준 SHA `21ed64071f81f35f37618717693bd30f727392c9`와 현재 head의 원본 SHA는 동일하다. 초기 base와 현재 코드의 전체 페이지 출력, 실제 넘침 쪽과 전후 문단 소유를 대조한다.
- 아직 검사 기대값이나 baseline을 변경하지 않는다. 현재 수정으로 생긴 회귀는 구현을 수정한다. 기준의 미달 여부는 실제 base 출력으로 입증한다.

## 독립 기준 재산출

- 원본 저장 제품: 한컴 2018, engine 2020을 명시했다. 사용자 지정 환경 파일로 start → status → download를 완료했다. 인증 정보는 기록하지 않는다.
- job `0e37f630-7383-4cb7-85e3-f6d3ab5e1514`: success, 한컴 `11.0.0.9136`, `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`.
- 전체 36쪽, 1,029,723바이트. PDF SHA-256 `3be0bbbe782f2cb3caa45a2ecec98e419103f6f04416050c3ba1f79a25ccc34d`.
- ignored output의 stage391-sample4-overflow/print에 저장했다. 비교 후 기존 기준 PDF 교체와 커밋을 진행한다.

## 비교 계획과 진행 상태

- 동일 입력·독립 Print PDF·compat 2022·96dpi·로컬 Fonts 공급으로 base와 현재 Native 전체 TSV를 먼저 산출한다.
- base 바이너리는 stage356의 정확한 21ed 빌드 증거와 SHA-256을 다시 확인하여 재사용한다. 현재 바이너리는 새 전체 회귀에서 빌드한 코드 후보다.
- 미달 또는 구조 차이 쪽만 review PNG를 생성하여 직접 판독한다. Native만으로 fresh WASM 통과를 주장하지 않는다.
- 다음 판단과 실제 명령·출력은 ignored output의 stage391-sample4-overflow에 기록한다.

## 실제 결과

- Native 전체 36쪽: base 미달 29쪽 / 최저 46.74204%, 현재 미달 27쪽 / 최저 46.77389%. 35쪽은 83.00905% → 88.04609%, 36쪽은 49.80808% → 49.74763%다. fresh WASM은 이번 입력에서 아직 재실행하지 않았다.
- 같은 2px baseline 조건의 증가 2건은 35쪽 맺음말 끝 두 줄이다. 새 Print PDF는 두 줄을 36쪽 첫머리에 둔다. 원본 문단 1242의 저장 줄도 63900 → 0 → 1500으로 재시작한다. 현재는 모두 35쪽에 그려 참고문헌의 36쪽 원점도 앞당겨진다.
- 직접 PNG에서 코드 블록의 회색 배경이 줄간격 구간에 빠지고, 그림 4-5 캡션의 소속 위치도 다른 것을 확인했다. 현재 35쪽 점수가 높아졌다는 이유로 이 결함을 면제하지 않는다.
- `internal_vpos_page_break_line`은 HWP3 변환본에서 음수 되감김만 수용한다. 0 되감김을 일괄 수용하면 변환기의 로컬 원점도 쪽 경계로 잘못 해석할 수 있으므로, 실제 저장 조각과 현재 흐름의 일치를 소비 경로에서 확인하는 방향으로 검토한다. 아직 생산 코드를 수정하지 않았다.
- 독립 Print PDF로 기존 `pdf/hwp3-sample4-hwp5-hwp-2020.pdf`를 교체했다. 이전 PDF는 ignored output에 보존했다. 요약 증거 `sample4-overflow-print-diagnosis.json`과 35·36쪽 review PNG는 completed 증적 경로에 보존한다.
- 현재 단계는 진단 완료이며 수정·최종 검증 완료가 아니다. 전체 36개 회귀 실패와 PR 제출 보류는 계속 유지한다.
