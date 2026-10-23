# #8060 공유 원본의 body corpus 참조 보관

기존에 전쪽 피델리티 검증 범위에서 이관한 `71352_lpg_regulatory_impact.hwp`가
본문 넘침 자동 수집에는 남아 있었다. 사용자 지시의 이미 이관한 범위 제외를 이
정확한 `body` 참조에 적용한다. 공유 원본과 PDF, 정상 9·42쪽 경계 검사 2건,
`off_canvas`와 `text_overlap` 축은 그대로 유지한다. 결함 해결이나 시각 통과가 아니다.

고정 base에는 이 입력과 전용 #8060 검사 파일이 없다. 입력 도입은
`576a86a138a05a215c020352e0c2a7072e4e051f`이며, 이번 통합에 들어온 뒤 이미 이관한
원본이 자동 body 수집에 남은 경우다. base에 등록된 전용 회귀를 새로 이관했다고
보고하지 않는다. base CLI에 같은 입력을 공급한 비교는 엔진의 기존 미달·신호를
확인한 별도 증거이며 base 파일 등록 증거로 대신하지 않는다.

## 현재 base 대조와 기존 시각 근거

고정 `upstream/devel`은 `2e3b4f880bd11d35398e6ea0100846611f13a72d`다.
현재 Native 관측은 `9fc704592`와 `shape-host-baseline-and-header-text` producer이며,
원본과 비교 조건을 맞춘 두 `layout-anomaly --overflow-tolerance 2 --json`의 전체 결과는
바이트 내용까지 같다. 48쪽 중 본문 아래 넘침은 11쪽 TextLine 한 건(11.68px)이다.
원장의 8건은 오른쪽 괘선 초과도 포함하므로 본문 아래 8건으로 보고하지 않는다.
이 현재 관측은 이번 코드가 만든 새 본문 증가가 아니라는 증거다.

이전 전쪽 Native/fresh WASM 미달과 사용자 승인 이관의 전체 TSV는
[기존 원문·독립 PDF·base/후보 근거](../../issue7445/deferred-20261020/added-issue8060-71352_lpg_regulatory_impact-hwp/README.md)에 있다.
그때 runtime base는 `efc52afb738bd408ac821c4e156101b8d26e5cbd`, 후보는 `a30dacfd0`이며,
Native 최저 31.18975%, WASM 최저 31.28414%다. 해당 기록을 현재 head의 전쪽 실행으로
바꾸어 쓰지 않는다. `efc52`와 고정 `2e3b4`의 runtime 변경은 native clipboard 경로뿐이며
HWP 조판 입력이 동일하다는 기존 대조 기록과 현재 anomaly 전체 일치를 함께 사용한다.

현재 열린 #8060의 본문과 댓글도 다시 확인했다. 원 해결 PR #8059의 저장 되감김
9·10쪽 및 42·43쪽 경계와 이번 11쪽 body 관측을 구분하며 정상 해결 검사를 제거하지
않는다. 11쪽의 정확한 관측은 이번 PR 증적을 공개한 뒤 기존 이슈에 후속 기록으로 연결한다.

## 제거 전 원문 보관

- [전체 body 모듈·모든 helper](body_overflow_baseline.rs.txt)
- [이관 목록 소비 helper](deferred_visual_regression_scope.rs.txt)
- [변경 전 전체 이관 목록](deferred_visual_regression_inputs.tsv.txt)
- [source commit·blob·이전 경로·SHA](preservation.json)
- [현재 base/후보의 동일 본문 신호](issue8060-current-base-corpus-body-equality.json)

사본은 모두 변경 전 바이트 그대로이며 `.rs.txt`를 자동 실행 suite로 등록하지 않는다.
새 baseline 숫자를 추가하거나 정상 기대값을 완화하지 않는다. body baseline TSV에는
이 원본의 행이 없었고, 자동 수집 입력 목록에서 이 원본의 body 참조 하나만 분리한다.
테스트 함수 수는 줄지 않는다. 입력 수 감소는 정확히 한 건이다.

## 복원 조건

같은 원본과 독립 Print의 전체 쪽수·내용 소유가 맞고 Native/fresh WASM 전체 raw가
각각 90% 이상이며 직접 확대 판독을 마친 뒤 목록의 정확한 body 행을 제거해 복원한다.
공유 원본·PDF는 원래 경로에 있고, 정상 경계 검사는 현재 활성 파일에서 계속 실행한다.


## 실행 확인

보존 사본만 추가한 시점과 body 목록 한 행을 추가한 뒤 모두 canonical manifest
`--check --base-ref 2e3b4f880`를 통과했다. 전후 1,646 sources·7,140 static test attrs·
48 integration targets가 같다. 정식 함수 제거는 0건이고 body 자동 수집 입력만 1건 줄었다.
같은 기존 전체 body 모듈의 실제 `collect_samples`를 사용한 진단은 수정 전 해당 원본이
남아 있어 의도한 FAIL, 수정 후 제외와 다른 축 보존을 확인해 PASS다. 정상 #8060 경계
2건도 기대값 변경 없이 통과했다. 진단 사본은 ignored output에만 있고 정식 suite에
중복 등록하지 않았다. 로그와 정확한 명령·producer는
`output/pr-review/planet6897-20261018/issue8060-*-results.json`에 있다.
