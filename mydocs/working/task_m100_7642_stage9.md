# 이슈 #7642 — 수식 관련 HML 회귀를 #7445로 이관

이전 단계: [원본 크기 0과 떠 있는 수식](task_m100_7642_stage8.md).

## 사용자 지시와 변경 범위

사용자가 수식 관련 HML 회귀 검사를 제거하고 나머지를 먼저 전수 검증하도록 지시했다.
새 이슈를 만들지 않고 열려 있는 #7445에 원본·독립 Print·현재 차이·복구 조건을 연결한다.
이관은 출력 개선 완료나 PASS가 아니다.

- HML parser의 수식 전용 검사 8개, serializer의 수식 전용 검사 4개,
  WASM의 수식 경고 보존 검사 1개를 제거한다.
- 복합 HML 저장 상태 검사의 수식 경로를 분리하고 일반 HML/비-HML DTO 검사는 유지한다.
- 원본 HML·한컴 HWP/HWPX 저장본·수동 2024 Print 및 2020 MCP Print는 그대로 보존한다.
- 누락 RATIO·RELSIZE·ShadeColor의 일반 저장 계약 검사와 다른 HML 검사는 유지한다.
- 제거 전 검사 본문·이름·source SHA는 이관 자산에 보존하며 자동 test 수집에서 제외한다.

## 검증 순서와 후속 범위

현재 Native 빌드가 끝난 뒤 테스트 소스를 변경한다. 테스트 목록을 새로 준비하고
HML의 남은 일반 계약 검사·lint·정책 비교를 실행한다. 전체 nextest를 이후 전수 실행하며
제거한 검사를 통과 개수로 세지 않는다. 입력 37개 중 HML 1개는 #7445로 이관하고
나머지 36개의 전체 페이지 Native/fresh WASM·직접 확대 판독을 계속한다.
기존 원본 manifest는 당시 범위로 보존하고 새 활성 manifest는 ignored output에 둔다.

원래의 다른 실패 입력·시험지·#1139 전체·회귀 관계형 전환·rebase·최종 lint·PR/CI/merge와
후속 처리 범위는 유지한다. 관련 원본의 최저 90%와 확대 판독 전에 회귀 기대값을 바꾸지 않는다.
오늘할일과 번호 기반 PR review는 code head CI 성공 뒤 trailing commit으로 작성한다.

## 이관 실제 적용과 정책 확인

수식 전용 parser 8개·serializer 4개·WASM 1개를 제거했고 복합 DTO의 일반 경로는
`test_hml_save_state_is_one_canonical_dto_for_hml_and_non_hml`로 유지했다.
제거 전 검사 본문은 `mydocs/pr/assets/issue7445/hml-equations-20261008/removed-tests.rs.txt`에
보존했다. 원본·PDF 및 일반 저장 계약 검사에는 변경이 없다.

[#7445 이관 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-6056803377)을 게시하고
API로 본문 일치·작성자·BOM 없음을 확인했다. 이슈는 OPEN이다. 원본 HML의 Native/fresh WASM
현재 진단은 각각 1쪽·6.22797%이며 이관 자산 README의 binary/source 해시로 구분한다.

`--prepare` 후 manifest 및 source unit tier의 정책 검사는
base `f0e7228f6dd2ea1437724e53ad640c40c56d204b`를 고정해 모두 exit 0이다.
Native Clippy는 exit 0(1분 14초), WASM Clippy도 exit 0이다. 남은 HML 일반 계약 검사는 전체 nextest에서 확인한다.
전체 회귀·전쪽 시각·최종 PR 게이트를 완료했다고 표시하지 않는다.

직전 단계의 Native 위치 보정 빌드는 exit 0(14분 14초)이며 세 위치 대조군은 각각
7.09220→100%, 0→100%, 0→100%다. 수식 글리프의 확대 판독 문제와 HML 파생 진단은
#7445로 이관한다. 공통 개체 위치 보정의 fresh WASM 및 다른 입력 무회귀는 아직 확인해야 한다.

## 전체 회귀의 컴파일 차단과 자체 보정

첫 전체 nextest는 실행 전에 exit 101로 종료됐다. 외부 crate의 `TextStyle`을 사용하는
`issue_4969_shaping_glyph_lowering`에서 새 공통 `ratio_draw_params`가 `pub(crate)`여서
E0624가 발생했다. E0282 두 건은 같은 접근 실패 뒤의 연쇄 오류다.
장평 계산을 검사 쪽에 복제하지 않고 기존 helper를 공개해 같은 계산을 소비하게 했다.
계산과 실제 출력은 바꾸지 않으며 해당 target 및 전체 회귀를 다시 실행한다.
첫 실행은 테스트 PASS/FAIL 결과가 아니므로 전체 회귀 완료로 세지 않는다.

접근 범위 수정은 `99accaec4`로 커밋했다. 차단 target의 재검사는
19 PASS / 0 FAIL / 1 skip, nextest exit 0이다. 전체 회귀는 이 결과로 대체하지 않는다.
새 사용자 지적에서 수식 줄의 큰 들여쓰기 오차가 확인되어, 영향 페이지 보정을 먼저 수행한
뒤 전체 nextest를 다시 실행한다. 첫 전체 빌드 실패와 이번 target 통과를 구분한다.
