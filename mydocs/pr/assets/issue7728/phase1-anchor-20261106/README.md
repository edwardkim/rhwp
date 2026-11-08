# #7728 첫 보정 — 닫힌 TAC 래퍼의 기본 줄 중복 점유

**부분 보정이며 이슈 완료·PR 수용 증거가 아니다.** 원본·기존 검사·문턱은 유지한다.
source parent는 `98a68a98f8d1b9a18a6d8ee5d092b6f66f1cf5bf`이고 실제 수정 두 파일의
hash·diff hash·frozen Native hash는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase1-anchor-20261106/validation.json`)에 기록했다.

원본 HWP의 문단 46 래퍼 높이 30,001HU는 TAC 자식 29,437HU와 안팎 여백 564HU의 합이다.
기본 글자 줄 30px를 별도 더하던 측정을 같은 원점의 합집합으로 보정해
430.0133px→400.0133px가 됐다. 실제 저장 빈 줄·다음 빈 문단과 독립 가로 프레임은 보존한다.

같은 원본의 독립 한컴 2020 Print PDF 18쪽을 PDFium·96DPI·compat2022·2px raw로 비교했다.
정식 PDF는 `pdf/issue7728/70833-electrical-safety-rule-hancom2020-print-20261106.pdf`이며
SHA-256은 `6cec821ff5897714669602c8707d66f669cfcb1187362c69101a0255c8d17956`이다.
기존 2022 생성 PDF도 보존한다. 변형한 빈 문단 대조 입력을 수용 입력으로 대체하지 않았다.

| 범위 | 보정 전 Native | 후보 Native |
| --- | ---: | ---: |
| 전체 18쪽 최저 | 16.84183% | 49.35093% |
| 9쪽 | 57.11636% | 63.64442% |
| 10쪽 | 16.84183% | 99.57724% |
| 90% 미만 | 5·6·7·9·10쪽 | 5·6·7·9쪽 |

[보정 전 Native 전쪽 TSV](before-native-candidate.tsv)와 [후보 Native 전쪽 TSV](after-native.tsv)를
보존했다. `before-*` 네 TSV는 고정 base 170b/runtime fc378의 이전 코드 자료다.
**새 후보 fresh WASM은 아직 실행하지 않았으며** 이전 WASM 결과를 새 후보 통과로 쓰지 않는다.

- 9쪽: [보정 전 review](before-native-p009-review.png) · [overlay](before-native-p009-overlay.png),
  [보정 후 review](after-native-p009-review.png) · [overlay](after-native-p009-overlay.png).
- 10쪽: [보정 전 review](before-native-p010-review.png) · [overlay](before-native-p010-overlay.png),
  [보정 후 review](after-native-p010-review.png) · [overlay](after-native-p010-overlay.png).

새 소스의 Mac 최소 글꼴 Native Skia 전체 lib는 4,039 PASS/0 FAIL/12 ignore,
누락 그림 PNG는 2 PASS, 직접 PDF는 22 PASS이며 세 종료 코드는 0이다.
로그는 ignored `output/open-issues-oldest-20261105/post-rebase-ece9/rebase-20261106-031441/`
아래 `issue7728-anchor-v1-native-native-{lib,png,pdf}.log`에 보존하고 명령·요약·hash는
validation JSON에 기록했다. 최종 전체 nextest·새 fresh WASM·영향 문서 전쪽 검증은 남아 있다.

남은 큰 빈 행의 계속 컷과 9쪽 후속 위치를 보정한 뒤 최종 소스에서 다시 검증한다.
전체 Native/fresh WASM이 수용 조건을 충족하기 전 정식 신규 회귀를 추가하거나 이슈를 닫지 않는다.
