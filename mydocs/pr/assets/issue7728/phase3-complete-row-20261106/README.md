# #7728 세 번째 보정 — 닫힌 중첩 행 프레임의 실제 컷 소속

**부분 보정이다. 7쪽이 74.64159%여서 이슈 종료·PR 제출은 계속 보류한다.**
기존 원본·검사·문턱을 유지했다. parent `5f425c8a51` 위 후보의 세 소스 파일·diff·Native
실행 파일 hash, 실제 검증 명령·종료 코드는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase3-complete-row-20261106/validation.json`)에 기록했다.

## 원인과 공통 소비 경로

안쪽 두 표의 원본 개체 높이 33,660HU·33,600HU는 실제 앞 네 행의 합
448.8px·448.0px와 정확히 닫힌다. 원시 최소 행 높이만 더하면 이 경계를 놓친다.
측정한 행을 각각 정수로 역변환한 뒤 더하는 후보도 한 행에서 1HU를 절삭해 실패했다.
이를 누적 물리 높이와 원본 선언의 같은 좌표계 비교로 보정했다. 임의 공차를 추가하지 않는다.

이 경계는 독립 개체가 없는 원본 앞 행 전체와 병합 범위·재조판 출처를 확인한 뒤에만 쓴다.
부모 유닛의 일반 행·세분 행이 같은 경계를 전달하고, 기존 재귀 소속 조회가 원래 자식 컷까지
추적해 최종 수용 스캐너에 전달한다. 단순 `hard_break` 표식이나 로컬 문단 원점만으로 쪽을
끊지 않는다. 새 일반 스캐너나 문서별 쪽수·행 번호 예외를 추가하지 않았다.

보정 전 5쪽은 두 열의 다음 `(생략)` 행까지 소비했다. 보정 후 해당 행은 독립 Print처럼
6쪽 첫 행으로 넘어간다. 실제 render tree 소속 진단은 FAIL→PASS이며, 5쪽 종료 컷은
`[21,34]→[20,33]`으로 바뀌었다. 최종 스캐너에서 같은 경계의 `strict=false→true`도 확인했다.
유닛에 표식을 넣었다는 사실만으로 완료를 주장하지 않고 실제 페이지 결과로 검증했다.

## 전쪽 Native 결과

같은 원본과 정식 한컴 2020 Print 18쪽을 Native PNG·96DPI·compat2022·PDFium·2px raw로
비교했다. [보정 전 TSV](before-native.tsv)와 [보정 후 TSV](after-native.tsv)를 보존했다.
세부 원 수치는 표시용 TSV가 아니라 validation JSON의 canonical 측정값이다.

| 범위 | 전 단계 Native | 현재 후보 Native |
| --- | ---: | ---: |
| 6쪽 | 59.22973% | 94.62153% |
| 7쪽 | 48.62869% | 74.64159% |
| 전 18쪽 최저 | 48.62869% | 74.64159% |
| 90% 미만 | 6·7쪽 | 7쪽 |

전체 18쪽 산출·누락 0을 확인했다. fixed devel `170b886f` 대비 0.1%p 초과 하락은 없다.
이 사실은 7쪽의 90% 미달을 면제하지 않는다. 5쪽은 95.35356%, 9쪽은 97.70433%다.

- 6쪽: [보정 전 review](before-native-p006-review.png) · [overlay](before-native-p006-overlay.png),
  [보정 후 review](after-native-p006-review.png) · [overlay](after-native-p006-overlay.png).
- 7쪽: [보정 전 review](before-native-p007-review.png) · [overlay](before-native-p007-overlay.png),
  [보정 후 review](after-native-p007-review.png) · [overlay](after-native-p007-overlay.png).

6·7쪽을 직접 판독했다. 7쪽의 앞 제목과 행 소속은 돌아왔지만 열 경계와 줄바꿈이 다르다.
원본 안쪽 표의 폭은 23,254HU·23,537HU(약 310.05px·313.83px)이고 독립 PDF 괘선도 이에
가깝지만 Native render tree에서는 둘 다 296.2px다. 원본 overlay 조회의 배율은 둘 다 1.0이다.
너비가 달라지는 실제 생산·소비 지점은 아직 미확정이며 글꼴 예외로 분류하지 않는다.

## 검증과 남은 범위

이 후보 소스에서 Mac 최소 글꼴 Native Skia 전체 lib **4,039 PASS/0 FAIL/12 ignore**,
누락 그림 PNG **2 PASS**, 직접 PDF **22 PASS**를 확인했다. 세 종료 코드는 모두 0이다.
로그는 ignored output의 `issue7728-frame-v8-native-native-{lib,png,pdf}.log`에만 두고,
명령·소스·로그 hash는 validation JSON에 보존했다.

현재 후보 fresh WASM·전체 nextest·영향 대조군·최종 범위 검증은 아직 미완료다.
전쪽 Native/fresh WASM 90% 조건을 충족하기 전 정식 새 회귀를 추가하지 않는다.
#7728과 #7445는 닫지 않으며 PR·push도 진행하지 않았다.
