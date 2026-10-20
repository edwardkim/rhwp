# 이슈 #7642 — 수식 전용 문단의 내어쓰기 단위 중복 제거

이전 단계: [HML 검사 이관](task_m100_7642_stage9.md).
시작 head: `f22e9143d`. 수식 관련 HML 출력·검사는 #7445로 분리한다.
나머지 원본 전수 검증·관계형 회귀·전체 검증·PR 후속 범위를 유지한다.

## 원본과 독립 관측

사용자가 2022년 9월 교육 통합 원본 17쪽의 수식 차이를 지적했다.
Native 페이지 점수는 94.55551%지만 둘째·셋째 식의 등호가 오른쪽으로 크게 밀린다.
같은 물리 영역을 576dpi로 대조해 위치 차이를 확인했다. 점수 통과로 면제하지 않는다.

원본 s0:p242:ci0 미주 27의 note paragraph 11은 네 수식과 명시적 줄바꿈 두 개다.
원본 ParaShape 41의 indent는 -11020이며, 앞 수식 폭 5398과 좌우 바깥여백 56+56의
합 5510에 정확히 두 배다. `resolve_para_style`은 정상 저장 단위를 나눠 indent를
이미 유효 크기로 해소한다. 한컴 Print의 이어지는 두 등호는 첫 줄 오른쪽 식의 등호와
정렬된다. 이 소유·정렬 관계가 기대값이며 화면 px 좌표를 기준값으로 삼지 않는다.

## 원인 경로와 보정

`ParaShape.indent → resolve_para_style의 유효 단위 →`
`height_measurer / typeset::paragraph::format의 줄 가용폭 →`
`compute_equation_only_tac_line_flow → paragraph_layout의 실제 줄 원점`을 추적했다.
세 소비 경로가 수식 전용 문단에만 옛 ×2 보정을 남겨 해소된 들여쓰기를 두 번 소비한다.
HWP3 계보 보상 ×0.5가 이 중복을 가리기도 했으나 유효 스타일의 단위를 바꿀 근거는 아니다.

원본 단위 변환은 style resolver에 남기고 측정·예산·배치 모두 공통
`paragraph_effective_margin_left`를 소비하게 한다. 수식 문자열·문서 ID·숫자 좌표에
따른 분기를 추가하지 않는다. 저장 줄/명시적 개행의 소유는 기존 공통 구성 결과로 유지한다.
일반 본문, 수식/본문 혼재, 셀, 내어쓰기가 없는 문단과 실제 줄바꿈 경계를 대조한다.

## 검증과 완료 조건

먼저 원본 17쪽 같은 영역과 정상 수식 대조군을 Native/fresh WASM으로 비교한다.
영향 원본의 전체 페이지 최저 90% 및 수식·간격 확대 판독을 확인한 뒤 회귀를 변경한다.
해당 회귀는 같은 미주·문단·줄의 수식 소유, 내용 순서와 첫 줄 RHS/이어지는 등호의 정렬을
독립 근거로 검사한다. 현재 고정 좌표의 기대값만 새 값으로 바꾸지 않는다.
이후 원래 36개·다른 실패 원본의 전수 시각 검증과 전체 nextest/lint/정책을 진행한다.
진단 TSV·JSON·로그·nextrun은 ignored output에만 보존한다.

## 적용한 공통 소비 변경

수식 전용 줄에서만 추가하던 `×2` 및 그 보상 배율을 측정·typeset 가용폭·실제 원점에서
제거했다. 일반 줄과 수식 줄은 해소된 스타일의 동일한 공통 내어쓰기를 소비한다.
배율 helper와 전달 필드를 제거해 같은 단위에 다시 배율을 적용할 통로를 줄였다.
현재 변경은 code 4개 파일의 15줄 추가 / 79줄 삭제이며 회귀 기대값은 수정하지 않았다.
Native 재빌드는 `stage10-equation-indent-native-build.log`로 실행 중이고 아직 수정 후
시각 통과나 전체 회귀 완료를 기록하지 않는다.


## 첫 Native 재검증 결과

`c886b29db6a2b766b078dff17535dab9df58b7d6`의 Native 빌드는 exit 0, 14분 56초다.
고정 바이너리 `output/pr-review/issue7642/failed-inputs-20261008/rhwp-equation-indent-once`의
SHA-256은 `cd4d63f94c5c9ab8dc215f033338baee1789df138a4c6084ab3cb9c86fc9eed9`다.
`equation-indent-once-native-provenance.json`에 코드 파일별 해시와 명령을 보존했다.

같은 원본·한컴 Print의 17쪽 2px 실루엣은 94.55551% → **95.33926%**다.
Native와 PDF의 전체 쪽수는 각각 23쪽이며 이 실행은 17쪽만 직접 비교했다.
둘째·셋째 수식의 x 원점은 첫 줄 RHS와 같아져 원본 내어쓰기 정렬을 회복했다.
576dpi의 같은 물리 영역 대조에서도 확인했다. 해당 영역의 닫는 괄호/등호와 수식 내부
간격 차이는 남아 있으므로 확대 판독은 **미충족**이며 시각 개선 완료나 회귀 변경으로
승격하지 않는다. 전체 23쪽 TSV는 별도 실행 중이고 fresh WASM은 아직 미검증이다.

증적은 ignored `output/pr-review/issue7642/equation-regression-scope-20261008/` 아래의
`indent-once-monthly-page17/`, `sep2022-indent-once-equation-zoom/`에 있다.
같은 head의 `cargo clippy --locked --target-dir target/pr-review -- -D warnings`는
exit 0, 1분 32초다. 나머지 필수 lint/전체 nextest는 최종 보정 뒤 다시 확인한다.
정상 대조군 `eq-002.hwp`/`eq-002.hwpx`는 첫 Chrome navigation timeout을 로그에 보존하고
180초 raster timeout으로 별도 경로에서 재검증 중이다. 환경 실패를 출력 결함으로 세지 않는다.


## 전체 페이지 Native 보조값

동일 고정 Native의 전체 23쪽 TSV 산출은 exit 0이다. 누락 쪽은 없고 최저는
20쪽 **90.44588%**이며 모든 쪽이 90% 이상이다. TSV는
`indent-once-monthly-all/sep2022-indent-once-all/silhouette.tsv`에 보존한다.
이는 stage10 바이너리의 결과이며 이후 글자 전진폭/괄호 보정의 결과가 아니다.
실루엣 전용 gate는 `not_evaluated`다. 확대 판독과 fresh WASM이 남아 있으므로
최종 통과·PR 준비 완료로 기록하지 않는다. `eq-002.hwp`와 `eq-002.hwpx`의
Native 대표 PNG는 각각 92.77108%이며 `indent-once-equation-controls-retry/`에 있다.
