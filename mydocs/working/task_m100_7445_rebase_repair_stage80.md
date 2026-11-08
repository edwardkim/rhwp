# Stage80: sample16 독립 그림 행 회귀 통합과 원격 공통 수정 조사

Issue: #7445. 부모 HEAD `2df2e53ced6a6fd9113f6e48651e429c6209e7ea`, base
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 사용자 지시에 따라
`ubuntu-ted2`의 `task/8160-sample16-raw100` HEAD `871b470437f39479ad8ba97506d328bd8f3921e4`를
읽기 전용으로 조사했다. 원격 source·branch·작업 트리는 변경하지 않았다.

## 회귀 제거와 보호 범위

원격에서 삭제한 독립 case는 `verified_stored_table_picture_rows.rs` 하나다. 원격은 이를
단순 폐기하지 않고 전쪽 case의 helper로 합쳐 세 저장본에서 실행한다. 현재 브랜치에는
그 전쪽 case와 새 기준 PDF가 아직 없어, 같은 독립 파일을 제거하고 기존
`issue_7642_sample16_2022_paragraph_relations`의 표 관계 검사로 통합했다.

기존 15개 assertion의 식을 helper 이름·공백만 정규화해 대조했으며 모두 동일하다.
원본 표·그림·표의 세 컨트롤과 TAC 속성, 단일 소유·내용·문자 순서·누락·중복,
앞뒤 비겹침·쪽 가로 포함·셀 주소를 유지했다. HWPX 원본과 Print 모드도 그대로다.
새 문서나 px 핀을 추가하지 않았다. 원격에서 새로 검증한 98% 좌표 fixture를 이 브랜치의
통과 근거로 복사하지 않았다. 기존 BCP 두 줄·기준선과 쪽 테두리 검사도 보존했다.

## 실제 검증

- suite `--prepare` 후 삭제한 case가 manifest에서 빠지고 이동한 helper가
  `regression_suite_005`의 기존 표 관계 검사에서 실행됨을 확인했다.
- Native Skia focused nextest: **3 PASS / 0 FAIL**, 실행 2.200초, exit 0.
  259개는 같은 harness 안의 filter 제외이며 실행 0건이나 fixture 누락을 통과로 세지 않았다.
- `rustfmt --check`, `git diff --check`, suite manifest `--check` 모두 exit 0.
- 원본 및 이전 case의 바이트·SHA와 assertion 대조, 실제 실행 로그는 ignored output의
  `rebase-d9fd85ed-20261023/ubuntu-ted2-8160-assessment/test-removal/`에 보존했다.

## 원격 구현의 현재 적용성

원격 동결 Native source `803046519d82d20c71d85937b96a8eefabb200d0`를 Linux에서 실행하고,
기존 C352/C354/C356 입력과 현재 Mac의 같은 PDF PNG·canonical 계산으로 재측정했다.
동결 바이너리 SHA는 `52e232fba7c9348f875abcd4dde19cb77cb77530c73225ec7a561b9be3f852da`다.

| 기존 대조군 | 현재 후보 | 원격 동결 Native |
| --- | ---: | ---: |
| C352 | 100.00000% | 91.58691% |
| C354 | 94.57727% | 100.00000% |
| C356 | 42.89522% | 33.33810% |

C354의 JPEG 전달 공통 수정은 직접 도움이 되지만 원격 브랜치 전체 적용으로 C352 개선을
잃거나 C356 미달을 해결했다고 주장할 수 없다. C354 review에서 세 격자·색상 strip을
확인했으며 실제 비교의 엄격 픽셀도 100%다. C356은 옅은 선이 보이더라도 raw 미달이며
사용자가 거절한 예외를 적용하지 않았다. 원격 엔진이나 fresh WASM을 이 브랜치에 통합하지 않았다.

조사 source·producer·전달 입력 hash·PNG·TSV·review/overlay는 같은 ignored assessment
디렉터리에 보존했다. 원격 sample16의 384쪽 결과는 그 작업의 증거이며 이 브랜치 수용이 아니다.
현재 전체 50조건 검증과 C354/C356 해결은 미완료이며 PR 제출을 계속 보류한다.
