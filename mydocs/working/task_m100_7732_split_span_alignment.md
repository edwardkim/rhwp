# #7732 후속: 병합 셀의 선택 조각 세로 정렬

## 범위와 확인 결과

PR #9083 병합 `f8234f6f90cb25d9601c9d5d762e0a565aea19e4` 뒤 사용자가37쪽의
`정책실현`을 지적했다. 원본50쪽과 독립 한컴2020 Print는 보존한다. 기존 원본 두 경로
전50쪽 최저98.75%와 해당37쪽98.75%는 수용 자격의 과거 관측이며 이 정렬 결함을 면제하지 않는다.

독립 PDF에서 같은 행 `31`과 `정책실현`의 기준선은 모두742.9480590820312pt
(990.597412109375 CSSpx)다. 현재 숫자는990.6000000000004px로 맞지만 병합 라벨은
981.266666666667px로9.330745px 위에 있다. 글리프 윤곽만의 차이가 아니다.
독립 선 위치 대비 현재 마지막 행의 위/아래 선 차이는 각각+0.951/-1.167 CSSpx다.
라벨 정렬과 선의 미세한 래스터 차이를 구분해 재검증한다.

## 원인 계층과 공통 소비

원본 row45/col2는Center·rowspan3이고 `정책실현`/`재정조달` 두 문단을 서로 다른 저장
쪽 프레임으로 가진다. 현재 페이지가 수용한 유닛 컷은(0,1), 내용 높이는13.333333px,
셀 높이는35.76px·안여백 합3.76px다. 옆의 한 줄 숫자도 같은 상자/내용 높이지만
`table_partial::layout_partial_table_cells`는 블록 컷의 병합 셀만Top으로 강제한다.

`stored_span_cut_owns_alignment`의 원본 슬롯 정렬 계약은 이미 존재하지만 그 소비를
높이로 고른straddle 컷에만 연결했다. 실제 BlockCut의 동일한 선택 범위는 읽지 않았다.
보정은 측정·줄 선택에서 수용한 동일cut_units를 먼저 확정하고, 원본 프레임의 정렬 소유도
그 범위를 읽게 한다. 문서ID·쪽수·문구별 예외나 고정 위치 덧셈을 추가하지 않는다.
일반 임의 컷·원본 재시작 중간·개체/중첩 셀 등 기존 helper가 거절하는 계약은 유지한다.

원 해결 이력은 저장 프레임/병합 정렬의 `265039cb583459956226313a1df82aea1788e686`,
후속 통합 `84764fa21c96cd5f8a0e158658e4162a4fb3ce9e` 및 #9083의 공통 유닛/paint 보정을
대조했다. 관련 소유 메서드와 실제 최종원점의 소비를 바꾸며 범용 렌더러를 새로 만들지 않는다.

## 회귀와 검증 상태

새 `issue_7732_split_span_frame_alignment`는 `tests/cases/`에만 추가했다.
37쪽이 양쪽98% 이상이고 독립 PDF 직접 판독을 마쳤으므로 Print 기준선±2 CSSpx 고정형과
같은 실제 셀/한 줄 숫자의 기준선·문단 소유 관계형을 명확히 구분한다. 보정 전 진단 실행은
실제0PASS/2FAIL이다. 설치 글꼴은 테스트의 전제가 아니다.

[독립 값·원본/PDF hash·보정 전 실행](../pr/assets/issue7732_split_span_20261115/before-analysis.json).
제품 보정 뒤 정식 Cargo 수집·집중/전체 회귀·Native/fresh WASM 전50쪽·정상 대조군과 보호 문서를
검증한다. 과거 PR의 실행을 후속 코드의 새 실행으로 바꾸어 쓰지 않는다. 후속 PR은 사용자 승인 범위다.

## 2026-11-15T13:25:12+09:00 — 공통 선택 컷 연결과 Native 집중 결과

실제 `cut_units` 계산을 정렬 소유 판정 앞으로 옮겨 RowCut/BlockCut의 표현과 관계없이
같은 수용 유닛 범위를 읽게 했다. 기존 원본 슬롯/현재 프로그램/저장 프레임 인증 helper를
재사용하며 수치나 문서별 예외는 추가하지 않았다.

Native37쪽98.75228%→99.29420%,38쪽99.84367% 유지다. 선택 라벨은 옆의31과
같은 기준선으로 옮겨졌고 review/overlay를 직접 판독했다. 최소 글꼴 진단2PASS·0FAIL이다.
정식 Cargo 수집과 전체 회귀·Native/fresh WASM50쪽·보호 검증은 아직 실행 전이다.

[집중 결과](../pr/assets/issue7732_split_span_20261115/focused-native-result.json) ·
[37쪽 review](../pr/assets/issue7732_split_span_20261115/after-native/review-037.png) ·
[overlay](../pr/assets/issue7732_split_span_20261115/after-native/overlay-037.png).

## 2026-11-15T14:00:11+09:00 — 정식 집중 회귀와 두 경로 전50쪽

정식 Cargo 자동 수집의 새 검사2건은 최소 글꼴 환경에서 모두 PASS했다(0.125초).
Native와 fresh WASM 각각50쪽을 새로 산출했고, 공식 두 자리 값은 두 경로 모두
37쪽만98.75%→99.29%로 상승했다. 나머지49쪽은 기존 값과 같고 전쪽 최저는47쪽99.23%다.
독립 Print의 다음 문단 `재정조달`은38쪽에서 한 번만 남고37쪽에 중복하지 않는다.
새 WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 대체 빌드이며
root `pkg`와 Studio `public`의 JS/WASM 해시 일치를 확인했다. Docker 최적화 완료가 아니다.

정본 보호 Native18조건1,771쪽은 전부 기존 값과 동일했다. 사용자 지시에 따라 상승한 조건은
최종 전쪽 TSV와 manifest를 함께 갱신하지만, 현재 Native 상승0쪽이므로 기준을 낮추거나
임의로 바꾸지 않는다. 보호 fresh WASM·필수 편람 gate·전체 회귀·실제 소비 경로는 진행 중이다.
최소 글꼴 프로필이 공개 테스트 fixture인 Happiness Sans까지 차단했던 컴파일 실패는
원인과 로그를 보존하고 그 파일 하나를 허용한 뒤 all-target Clippy를 실제 재실행해 PASS했다.
제품 또는 회귀 기대값은 이 실행 환경 오류 때문에 바꾸지 않았다.

[전50쪽 결과와 판독](../pr/assets/issue7732_split_span_20261115/whole-footnote-validation.json).

## 현재 base f8234f6f의 검증 완료와 PR9088 선행 병합

최소 글꼴 전체 회귀는13,567 PASS·0 FAIL·45 skip(테스트890.642초), Native Skia
라이브러리4,034 PASS·12 ignore, 그림2 PASS, 직접 PDF36 PASS다. 필수 lint11단계 모두
최종 exit0을 확인했다. 보호36조건3,542쪽의 상승·하락은 모두0쪽이며 필수 편람383쪽 gate도 PASS했다.
[전체 Rust 실행](../pr/assets/issue7732_split_span_20261115/rust-validation.json)과
[전쪽 보호 결과](../pr/assets/issue7732_split_span_20261115/protected-validation.json)를 보존한다.

실제 설치 SFNT metadata/blob를 공급한 Local Font Access API adapter에서 Studio Canvas2D
36·37·38쪽은99.75605·99.07736·99.57140%다. 실제 OS 열거 성공을 주장하지 않는다.
같은 조건의 base37쪽98.52771%보다 개선했고36·38쪽은 같다. 초기 JS 응답의 Vite 소스맵에서
WASM URL 재작성 한 곳을 복원하면 root JS와 바이트가 같고 초기 WASM 응답도 동결 package와 같다.

추가 SVG 진단은 주의가 필요하다. 준비된 동일 프로그램의36쪽은 base/후보 모두88.253%로
기존 미달이고37쪽은89.97757%→90.23439%,38쪽은91.98205% 유지다. CanvasKit 요청6건은
실제 Canvas2D fallback이므로 CK PASS가 아니다. 같은 프로그램을 준비하지 않은 별도 SVG
진단89.89609/89.445%와 검증 모듈 공급·portable metric 선택 오류도 로그를 보존했다.
이 결과를 숨기거나 문턱을 바꾸지 않는다. [소비 경로 판정](../pr/assets/issue7732_split_span_20261115/consumer-validation-before-rebase.json).

사용자는 PR 제출을 보류하고 #9088 병합 뒤 리베이스하도록 지시했다. #9088의 실제 병합
`e7ee272cfa33a960b0d0d924983d29e6509903e0`을 확인했다. 그 PR은 인쇄/글꼴 소비 경로와
보호 기준30행을 바꾸므로 위 결과를 새 source의 결과로 바꾸어 쓰지 않는다. 현재 기록을
커밋하고 리베이스·영향 확인 후 준비 범위까지만 진행하며 원격 push·PR 생성·댓글·merge는 하지 않는다.
