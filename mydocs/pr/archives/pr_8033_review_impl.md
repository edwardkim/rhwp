# PR #8033 구현·검증 기록

[개별 리뷰](pr_8033_review.md)의 실제 소비 경로와 원 커밋 대응을 따른다.
원 contributor head는 `8a02d977b6e72e9a8117a15a0f786e0cc0cfb598`, 검증 코드·테스트 source는
`04c92da78165ccbf63b467855591df183655a98d`, 고정 devel은 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`다.

## 보정과 검증

TAC fit은 format이 이미 포함한 tac_outer_margin_v_px를 중복 계상하지 않고 최종 소유 줄·실측 표 높이를 소비한다. 현재 tac_fit::prepare의 단일 소유 frame, 저장 frame, 편집 후 성장 floor를 함께 확인했다. 한 번의 여백으로 들어가는 입력과 같은 식에서 실제로 넘치는 입력을 각각 1쪽·2쪽으로 검사해 무조건 fit하는 완화를 막는다.

tests/cases/tac_table_no_lineseg_outer_margin_fit.rs의 fits/overflows 경계. 실제 저장 frame과 no-LineSeg 재조판은 서로 다른 분기이며 한쪽 성공으로 다른 분기를 대체하지 않는다.

공통 연산의 정상 대조군과 실패 경계를 개별 리뷰의 실제 검사에 연결한다. 이미 정확한
독립 기대값은 유지하며, 잘못된 고정 위치·과거 쪽수 핀은 원본·독립 Print에 따라 재검토한다.
실행 결과와 명령, 현재 입력의 전체 Native/fresh WASM TSV·직접 판독은
[최종 제출 증거](../assets/s21_20261027/README.md)의 해당 조건을 따른다.

## 현재 단계와 후속 처리

메인터너 보정 후 수용 가능 — fix(typeset): 저장 줄 없는 글자처럼 취급 표의 바깥 여백을 쪽 맞춤에서 한 번만 센다 범위의 로컬 검증 완료.

통합 PR의 실제 code head CI·merge는 로컬 검증과 별개다. 최신 지시에 따라
code head CI 성공 뒤 같은 PR의 self-review·오늘할일 trailing commit을 추가하며, 완료 시 실제 PR·CI·merge SHA와
원 PR별 comment·close 결과를 기록한다. 번호나 merge 결과를 미리 만들지 않는다.
삭제한 sample16·편람의 일반 회귀와 파생 참조를 되살리지 않는다. 명시적으로 지정한
#8188 네 변환 입력의 기능 보존 검증은 별도 예외 범위로 구분한다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
