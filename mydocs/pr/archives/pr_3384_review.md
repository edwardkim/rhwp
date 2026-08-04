# PR #3384 검토 기록

## 라우팅·메타데이터

외부 collaborator 통합 검토, HWPX 자산·시각 증적 경로를 적용했다. 작성 시점 참고값:
`kevin9327`의 `pr/task-edit-set-cell` → `devel`, 최신 head `d571e8a7e996`, 보류
comment/review 없음, 검토 branch `review/kevin9327-20260726`. #3391은 이 PR에 포함된 이슈다.

## 변경 검토와 메인터너 보정

`edit set-cell`은 표 번호·행·열 좌표로 값을 기록한다. 기본 동작은 제출 양식의 안내문 파란
이탤릭을 검정 일반 글씨로 바꾸고, `--keep-style`은 기존 모양을 보존한다. 통합 검토에서 큰
row/col이 `u16`으로 wrap되지 않게 입력 상한 검사를 더했고, 기본 스타일도 문서의 다른 검정
스타일을 재사용하지 않고 **대상 셀의 글꼴·크기·장평·자간을 보존한 복제본**을 쓰도록 보정했다.

## 실제 시각 검토

원 공고 양식(좌)과 CLI 작성본(우)을 실제로 열어 비교했다. 체크 선택과 작성 값이 들어가고,
기본 글자가 검정으로 보이며 표의 셀 경계·행 높이·본문 배치가 유지된다.

![K-Startup 원본 양식과 CLI set-cell 작성본](../../report/assets/task_m100_3391/kstartup-form-compare.png)

## 검증·권고

스타일 보정 뒤 `edit_set_cell_contract` 5개와 통합 release-test 전수가 통과했고, 산출 HWP를
재파싱해 색상·기울임·굵기와 글꼴 ID·크기·장평·자간 보존을 확인했다. clippy·fmt·diff check도
통과했다. 세부는 [통합 구현 기록](pr_3345_review_impl.md)에 있다.

**메인터너 보정 후 수용 가능**. #3381·#3391의 실제 close는 통합 PR merge 뒤 확인한다.
