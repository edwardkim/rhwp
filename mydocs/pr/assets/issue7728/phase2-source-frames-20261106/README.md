# #7728 두 번째 보정 — 저장 물리 프레임과 부동 표의 본문 소속

**부분 보정이다. 6·7쪽 미달과 7쪽의 base 대비 하락이 남아 이슈 종료·PR 제출은 보류한다.**
원본·기존 검사·90% 및 0.1%p 기준은 바꾸지 않았다. 소스 parent는 `2e85891b2d`이며
후보 파일·diff·Native 실행 파일의 SHA와 실제 명령·종료 코드는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase2-source-frames-20261106/validation.json`)에 있다.

## 확인하고 보정한 원인

- 원본의 첫 개체 프레임이 큰 빈 행 안에서 닫히는 경우, 첫 글줄과 남은 물리 공간을 별도
  유닛으로 전달한다. `<신설>` 글줄은 앞 조각만 소유하며 빈 꼬리에 복제하지 않는다.
  빈 꼬리를 글자 있는 문단으로 다시 판정해 안여백을 한 번 더 예약하던 경로도 보정했다.
- 원본 문단 안의 짧은 `0→0` 재시작은 같은 문자 범위·열·폭의 원본 줄과 실제 행 원점으로
  첫 개체 프레임을 인증한다. 한 단 정의는 기존 공용 텍스트 메타데이터 판정을 재사용한다.
  부모의 예약 높이와 자식 부분 표 배치가 같은 줄 컷과 종료 간격을 소비한다.
- 계산된 빈 호스트의 부동 표가 실제 이월될 때, 현재 쪽에 들어가는 호스트 줄과 후속 본문은
  기존 공용 prefill 경로에 남긴다. 저장 줄·음수 offset·어울림 개체의 경로는 유지한다.
  원본 문단42를 임의로 높이 0으로 만들지 않는다.

독립 한컴 2020 재저장본에 문단42의 문자 `x`만 넣으면 그 문자는 8쪽에 인쇄되고 표는
9쪽에 남는다. 문단42·43 제거와 글자 크기 대조도 원본을 보존한 진단용 변형 입력으로만
실행했다. 원본의 페이지 소속 진단은 보정 전 실패에서 보정 후 통과로 바뀌었다.
빈 행 시작과 문단 내부 계속 문구의 소속 진단도 각각 실제 FAIL→PASS다.
새 정식 회귀 검사는 아직 추가하지 않았다.

## 실제 전쪽 비교와 남은 결함

같은 원본 HWP·독립 한컴 Print 18쪽을 Native PNG·96DPI·compat2022·PDFium·2px raw로 비교했다.
[전 단계 전쪽 TSV](before-native.tsv)와 [후보 전쪽 TSV](after-native.tsv)를 보존했다.
표의 세부 수치는 표시용 반올림 TSV가 아니라 validation JSON의 canonical 원 수치다.

| 범위 | 전 단계 Native | 후보 Native |
| --- | ---: | ---: |
| 5쪽 | 54.39305% | 96.74611% |
| 9쪽 | 63.64442% | 97.70433% |
| 6쪽 | 49.85681% | 59.22973% |
| 전 18쪽 최저 | 49.35093% | 48.62869% |
| 90% 미만 | 5·6·7·9쪽 | 6·7쪽 |

7쪽은 fixed devel `170b886f`의 49.35093%보다 **0.72224%p 하락**했다. 허용하거나 기존
미달로 숨기지 않는다. 6쪽의 `(생략)` 행과 7쪽 앞의 제목 소속을 계속 복구해야 한다.
첫 네 행의 실제 합은 왼쪽 448.8px·오른쪽 448.0px로 원본 개체 높이와 정확히 닫히지만,
현재 부모 유닛의 다음 행에는 물리 쪽 경계가 전달되지 않는다. 해당 완결 행 소속 진단은
여전히 FAIL이다. 계산 후보를 넣었다는 사실을 실제 적용·해결 증거로 취급하지 않는다.

- 4쪽: [보정 전 review](before-native-p004-review.png) · [overlay](before-native-p004-overlay.png),
  [보정 후 review](after-native-p004-review.png) · [overlay](after-native-p004-overlay.png).
- 5쪽: [보정 전 review](before-native-p005-review.png) · [overlay](before-native-p005-overlay.png),
  [보정 후 review](after-native-p005-review.png) · [overlay](after-native-p005-overlay.png).
- 9쪽: [보정 전 review](before-native-p009-review.png) · [overlay](before-native-p009-overlay.png),
  [보정 후 review](after-native-p009-review.png) · [overlay](after-native-p009-overlay.png).

## 검증 범위

현재 후보에서 Mac 최소 글꼴 Native Skia 전체 lib **4,039 PASS/0 FAIL/12 ignore**,
누락 그림 PNG **2 PASS**, 직접 PDF **22 PASS**이며 세 종료 코드는 모두 0이다.
로그는 ignored output의 `issue7728-frame-v5-native-native-{lib,png,pdf}.log`에만 보관한다.
명령·소스 hash·로그 hash와 진단의 전후 종료 코드를 validation JSON에 기록했다.

현재 후보 fresh WASM·전체 nextest·영향 대조군 전쪽과 최종 lint/build는 아직 미완료다.
이전 소스의 전체 회귀·WASM 결과를 재사용하지 않는다. #7445 부모나 이 하위 이슈를 닫지 않는다.
