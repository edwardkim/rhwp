# 단계 405: 다른 작업이 보정하는 CGMP의 렌더링 회귀 제거

## 사전 분석과 범위

사용자는 CGMP가 다른 작업의 보정 대상임을 확인하고 이 브랜치의 회귀를 제거하도록
지시했다. 기존 [#7445 이관 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5886337869)을
유지하며 새 이슈·중복 댓글을 만들지 않는다. 이 브랜치의 CGMP 자체 보정 계획을 철회했다.

대상은 `samples/issue6035/cgmp_evaluation_table.hwpx` 48쪽이다. 같은 issue 번호의
`2804253_cosmetics_gmp_checklist.hwpx` 11쪽과 전용 검사는 별도 입력이므로 변경하지 않았다.
원본 HWPX·독립 PDF·기존 비교 증거·다른 작업의 checkout은 보존했다.

## 독립 기대값 재검토

기존 두 함수는 물 공급 제목과 첫 항목이 반드시 같은 쪽에 있어야 한다고 요구했다.
고정 `upstream/devel`의 제목은 36쪽이지만 독립 한컴 Print PDF의 제목은 **35쪽**, 첫 항목은
**36쪽**이다. 현재 후보도 제목35쪽·항목36쪽이다. base/current의 차이만 보고 새 회귀로
추정했던 판단을 정정한다. 잘못된 같은 쪽 전제를 관계형이라는 이유만으로 유지하지 않는다.

| 비교 | 전체 쪽수 / PDF | 35쪽 | 36쪽 |
| --- | --- | ---: | ---: |
| 고정 devel `21ed64071` Native | 48 / 48 | 97.39482% | 47.66633% |
| 현재 후보 Native | 48 / 48 | 98.32878% | 85.09829% |

직접 review PNG와 PDF 추출 텍스트를 확인했다. 현재 Native 전48쪽 TSV도 완료했지만
11쪽이 90% 미만이고 최저0%다. fresh WASM은 실행하지 않았으며 CGMP 피델리티 개선 완료나
원래 출력의 수용을 주장하지 않는다. 이 진단은 다른 작업을 위한 증거로만 보존한다.

## 변경과 확인

- `issue_6035_cell_split_empty_band.rs`의 전용 함수 **2개**를 제거하고 원래 파일을 자동 수집되지 않는 `.rs.txt` 자산으로 보존했다.
- 같은 입력을 본문 넘침·글자 겹침·용지 밖 원장의 수집 대상에서 제외했다. 본문 넘침2·글자 겹침1의 기존 TSV 행도 제거하여 수집 범위와 일치시켰다. 다른 문서의 기대값은 변경하지 않았다.
- 셀 넘침은 이미 #7445 보류 대상이었다. 사용자 지시에 따라 이 CGMP 입력은 렌더링 실행도 제외했다. 다른 보류 입력의 실행 범위는 변경하지 않았다.
- 생산 코드 변경·새 테스트 추가·원본 및 PDF 삭제는 없다.
- manifest 준비 후 고정 base를 지정한 check와 fmt가 통과했다. 남은 규제분석·low-slack·교육과정 관련 기존 함수는 **7 PASS / 0 FAIL / 1084 skip**, 11.263초, exit0이다. locked nextest release-test, threads8로 실행했다.
- 원장 전체와 전체 nextest는 아직 다시 실행하지 않았다. 이 집중 결과를 전체 통과로 보고하지 않는다.

[해시·명령 결과·범위 기록](../pr/assets/issue7445/completed-20261009/cgmp-fidelity/validation.json),
[현재35쪽](../pr/assets/issue7445/completed-20261009/cgmp-fidelity/cgmp-current-p035-review.png),
[현재36쪽](../pr/assets/issue7445/completed-20261009/cgmp-fidelity/cgmp-current-p036-review.png).
로그는 ignored `output/pr-review/7445-completed-20261009/stage405-existing-transfers-remaining/`,
전쪽 TSV는 같은 output의 `stage406-cgmp-full-fidelity/native/cgmp/silhouette.tsv`에 있다.

CGMP의 해당 회귀 제거를 완료했다. 나머지 차단 사유는 범위를 구분하여 계속 처리한다.
