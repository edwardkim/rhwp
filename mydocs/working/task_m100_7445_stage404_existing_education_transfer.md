# 단계 404: 이미 이관된 교육과정 문서의 잘못된 쪽수 검사 제거

## 사전 분석

- Issue: #7445. [기존 이관 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-6063814441)과 [같은 고정 base의 후속 검토](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-6091651108)를 조회했다. 새 이슈나 중복 댓글을 만들지 않는다.
- 같은 원본 `samples/task2287/1342000_edu_curriculum_map.hwp`의 SHA-256은 `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e`다. `upstream/devel` 고정 SHA `21ed64071f81f35f37618717693bd30f727392c9`의 원본과 바이트 동일하다.
- 기존 두 함수는 부분 후퇴 또는 줄 소유 자체를 확인하지 않고 전체 **413쪽**만 요구한다. 독립 한컴 Print는 **415쪽**이다. 413을 현재 출력으로 바꾸는 방식으로 통과시키지 않는다.
- 기존 댓글의 다른 후보 결과를 이 브랜치의 결과로 사용하지 않는다. 직접 실행한 같은 원본·`pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf`·96DPI·compat2022·명시 글꼴 조건을 고정했다.

## 직접 비교와 판정

| 실행 | 전체 쪽수 / 독립 PDF | 26쪽 관용 실루엣 | 결과 |
| --- | --- | ---: | --- |
| 고정 upstream/devel Native | 413 / 415 | 62.95597% | 기존 미달 |
| 이 브랜치 Native, 생산 코드 `47e8e570c` | 416 / 415 | 68.79519% | 잔여 피델리티 보류 |

양쪽 실행은 실제 exit 1로 종료했다. 표 행·본문 줄·하단 외곽의 차이를 review PNG에서
대조했다. 선택 한 쪽의 미달 근거이며 전415쪽 시각 비교나 fresh WASM 완료로 확대하지 않는다.
현재416쪽 출력을 정확한 것으로 승인하지 않는다. 기존413쪽 핀의 오류와 현재 출력의 잔여
쪽 소속 문제를 구분한다.

## 변경과 확인

- `issue_6761_stored_vpos_rewind_page_break::partial_retreat_inside_a_page_does_not_break`를 제거하고 이 함수만 쓰던 상수도 제거했다. 같은 파일의 시장구조 되감김·본문 소속 검사는 유지했다.
- `issue_7226_rowspan_only_row_cut::the_fix_does_not_add_a_page`를 제거했다. 내용 보존·이어받기·셀 포함과 예산 변화의 기존 검사는 유지했다.
- 두 변경 전 파일은 자동 수집되지 않는 `.rs.txt` 자산으로 보존했다. 원본 HWP와 독립 PDF는 그대로 남겼다. 생산 코드·baseline 기대값·공차·다른 검사 함수는 변경하지 않았다.
- 제거 전 두 함수는 최신 후보 개별 실행에서 실제 FAIL이었다. 제거 후 남은 관련 함수는 **3 PASS / 0 FAIL / 183 skip**, 11.079초, exit 0이다.
- 처음 집중 명령의 suite12/13은 해당 모듈 등록이 없어 exit4/0tests였다. 이를 성공으로 기록하지 않고 현재 registry의 suite001에서 위3건을 다시 실행했다. fmt의 끝 빈줄은 정리하고 최종 확인했다.

[입력·기준·빌드·검사 해시 및 결과](../pr/assets/issue7445/completed-20261009/education-page-pins/validation.json),
[고정 devel 26쪽 review](../pr/assets/issue7445/completed-20261009/education-page-pins/base-native-p026-review.png),
[현재 26쪽 review](../pr/assets/issue7445/completed-20261009/education-page-pins/current-native-p026-review.png).

로그·중간 JSON은 ignored `output/pr-review/7445-completed-20261009/stage404-existing-education-transfer/`에 보존한다.
기존34개 실패 모두를 처리한 것으로 보고하지 않는다. 그 밖의 이미 이관된 입력도 같은 원본·미해결
증상·기대값을 확인해 단계적으로 처리한다. 정확한 기존 계약과 현재 수정의 실제 회귀를
문서가 미달이라는 이유만으로 제거하지 않는다.

기존 #7445 이관 범위의 잘못된413쪽 검사 두 개 제거와 남은 관련 검사 실행을 완료했다.
