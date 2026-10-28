# 단계 38: 첫 표 조각의 원본 개체 앵커 소유 복구

## 원인과 원 해결 근거

- parent `028385943d51e3ef7096a194cc16feceb08504c1`, base `2e087a5b4bc1515ad877e8020b30114639a59392`.
- #7752의 accepted cut/fragment row 원장은 유지한다. `issue2004_cell_image_stack.hwp`는 base 8쪽인데 통합 후보가 빈 4쪽을 추가해 9쪽이었다. 기존 #4068 두 함수가 실제로 실패했다.
- `from_stored_empty_host_after_line`은 직전 글줄·저장 앵커·실제 흐름이 닫히는 표 상자를 만들지만, 폭0의 원본 개체 앵커도 `Exclusion`으로 발행했다. 새 첫 조각 경로는 이를 별도 글줄로 예약했다. table_top은 44.5733 대신 72.5733px, 남은 높이는 900.3 대신 868.5px가 되어 첫 내용 871.9px를 다음 쪽으로 밀었다.
- 앞 줄과 앵커의 원점 일치·오프셋0 근거는 [기존 보정326](task_m100_7445_stage316_completed_pr.md)을 대조했다. 이 근거를 제거하거나 문서별 수치 예외를 만들지 않는다.
- 공통 `object_only_saved_table_anchor`로 소유를 분류해 원본 폭0 단일 개체 앵커는 `EmptyHost`를 발행한다. 실제 공백/텍스트 줄은 기존 `Exclusion`을 유지한다. producer → `for_first_fragment` → host line 예약 → emit/paint가 같은 분류를 소비한다. 별도 위치 clamp나 컷 수 조정은 없다.
- 기존 `RHWP_TABLE_DRIFT` 조건부 진단을 첫 조각의 source/placement/줄 소유까지 보강했다. `.log`는 ignored output에만 보존한다.

## 실제 검증

- 보정 전 #4068 2 FAIL → 후보 2 PASS. 정상 #6950 문단 끝·#7330 공백 호스트·#7418 본문/분할 행 및 단계37 그림/PUA를 포함해 45 PASS/0 FAIL, 테스트 0.846초. 필터 제외 1506건을 전체 검증 통과로 쓰지 않는다.
- 이전 4eee 전체 회귀에서 실패한 정확한147함수를 같은 Native 후보에서 재검사했다. 91 PASS/56 FAIL, 테스트59.620초, 컴파일 포함198.215초, exit100이다. [남은 함수](../pr/assets/issue7445/completed-20261009/stage38-empty-anchor-20261022/remaining-failures.json)를 보존하며 전체 회귀 재실행을 대체하지 않는다.
- Cargo jobs8, test threads8, 공유 target 순차 실행. 현재 manifest로 실제69모듈/26binaries와147함수 선택을 확인했다. 검사·기대값을 삭제하거나 완화하지 않았다.

## 동일 환경 Native 전쪽 대조

96DPI/Print, 원본 세대별 compat2022/2024, 같은713개 글꼴·PDFium smooth_rgb_v1·2px raw 실루엣 기준이다. 각 source와 독립 PDF, frozen base/후보 executable hash는 [producer](../pr/assets/issue7445/completed-20261009/stage38-empty-anchor-20261022/producer.json)에 고정했다.

| 문서 | base/후보/PDF 쪽수 | base 최저 | 후보 최저 | 후보90% 미만 | 0.1%p 이상 하락 |
| --- | --- | ---: | ---: | ---: | --- |
| 이미지 스택 | 8/8/8 | 97.42222% | 96.42147% | 0 | 1·2·3쪽 |
| 규제분석 | 17/17/17 | 73.73353% | 99.26826% | 0 | 0 |

- 이미지 스택의 4~8쪽은 원래 source 소유를 복구했고 각각 base 대비 -0.00190/0/-0.00002/+0.00002/-0.00066%p다. 1~3쪽은 -0.47930/-0.31094/-2.08377%p로 별도 보정이 남아 있다. 높은 4~8쪽 점수로 상쇄하지 않는다.
- 3쪽 제목의 실제 원문 U+2024 한점 리더는 base에서12px, 후보에서24px로 측정된다. 글꼴·크기·장평·원본 자간은 같고 뒤 텍스트가 밀린다. source metric/결측 폴백의 공통 경로를 다음 단계에서 검증한다. 아직 해결로 쓰지 않는다.
- 4쪽 원본 그림과 글의 소유·순서·끝, 규제분석2쪽 본문·중첩 표·쪽번호를 review와 overlay로 직접 대조했다. 가는 괘선의 래스터/두께·이미지 재구성 차이를 완전 일치라고 쓰지 않는다.
- [이미지4쪽 review](../pr/assets/issue7445/completed-20261009/stage38-empty-anchor-20261022/image-stack-004-review.png) / [overlay](../pr/assets/issue7445/completed-20261009/stage38-empty-anchor-20261022/image-stack-004-overlay.png), [규제분석2쪽 review](../pr/assets/issue7445/completed-20261009/stage38-empty-anchor-20261022/regulatory17-002-review.png) / [overlay](../pr/assets/issue7445/completed-20261009/stage38-empty-anchor-20261022/regulatory17-002-overlay.png).

## 남은 제출 게이트

새 소스의 전체 회귀, Native/fresh WASM 전쪽 및 실제 소비 경로 검증은 계속 필요하다. 이번 후보의 fresh WASM 수용이나 PR 준비 완료를 주장하지 않는다. 편람의24쪽 하락과 다른 필수 회귀도 유지한다. 새 픽셀 핀·지침 변경·원격 push/PR/댓글 게시를 하지 않았다.

원본 로그·TSV·PNG·전후 source/producer는 ignored `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-7752-20261022/`의 `repair-focused-v4`, `retry-failed-native-v4`, `empty-anchor-native-visual`, `image-stack-border-owner-diagnostic`, `image-stack-page3-text-programs`에 보존한다.
