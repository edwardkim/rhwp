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

## 2026-11-15 15:10 KST — #9088 병합 뒤 충돌 없는 리베이스

#9088의 실제 병합 `e7ee272cfa33a960b0d0d924983d29e6509903e0`을 fetch한 뒤
`6c635a2941ea481ef37571f7e68f3454bd1bee58`에서 `a9b0d49fd3a42ee3bf8ad4c212c9024d9c537c36`으로
리베이스했다. 충돌0이며 range-diff의 기존4개 패치가 모두 동등하다.
원본·기대값·기준 PDF를 바꾸지 않았다.

#9088은 인쇄 글꼴 프로그램과 fractional coverage 경로를 바꾸고 정본 보호30개 값을 높였다.
따라서 f823 base의 실행을 새 후보 통과로 재사용하지 않는다. 새 base와 후보를 각각 빌드해
각주 원본50쪽 및 유입 #9075/#9088 원본14쪽의 Native/fresh WASM을 같은 조건으로 비교한다.
14쪽 원 검증의9개 공급 프로그램은 SHA가 같은 파일을 내부 검증용 output으로만 확보했다.
현재 시스템/사용자 글꼴은 변경하지 않았으며 해당 프로그램을 Git에 포함하지 않는다.

최소 글꼴 Rust와 보호36조건, 필수 편람 및 실제 소비 경로는 새 source 검증이 완료된 뒤
결과를 갱신한다. 사용자의 최신 요청에 따라 PR 생성·push 없이 준비까지만 진행한 뒤 대기한다.


## 2026-11-15 17:13 KST — 보호 비교 지침 보완과 사용자 글꼴 등록

사용자 요청에 따라 Mac의 보호 문서 비교 지침을 `7017b7a6ba`에 반영했다.
누락된 기준 프로그램 확보, 파일 SHA·TTC face·대체 설정 및 실제 선택 확인,
일반 글꼴 환경의 Visual Sweep과 최소 글꼴 Rust 회귀의 구분을 명시했다.
특정 Linux 호스트에서 복사하라는 표현은 사용자 정정에 따라 넣지 않았다.
같은 Mac의 base에서도 하락한다는 사실을 보호 기준 면제로 쓰지 않는다.
문서 링크 검사는 통과했고 전체 문서 메타데이터의 기존 오류 20건 대비 신규 오류는 0건이다.

사용자가 필요한 글꼴의 사용자 등록을 요청해 비교 공급 파일 33개를 점검했다.
동일 파일 21개는 기존 사용자 폰트에서 재사용했고, 누락된 원본 9파일·12face를
`~/Library/Fonts/rhwp-protected-originals/`에 추가했다. 작은 검증용 서브셋 3개는
명시적 공급을 유지했다. 필요한 파일은 기존 검증 공급본과 SHA를 대조했고,
현재 Linux에서 같은 SHA로 확보한 3개도 다시 확인했다. 기존 파일을 덮어쓰지 않았다.

현재 문서의 캡처가 끝난 뒤 다음 문서 진입을 멈추고 등록했으며, 설치 전 결과는 별도 기록이다.
CoreText 사용자 범위와 독립 프로세스의 실제 파일 descriptor·glyph를 확인한 결과
30파일·36face의 사용자 등록 범위·해시·읽기 검사가 통과했다. Human Myungjo의 임시
PostScript 이름은 전역 이름 열거에 없으므로 실제 파일 descriptor로 확인했다.
동명 버전의 기본 선택이 다른 경우도 있어 설치만으로 보호 결과가 복원됐다고 쓰지 않는다.
[사용자 등록 영수증](../pr/assets/issue7732_split_span_20261115/post-rebase-user-font-registration.json)에
실제 선택 SHA와 등록 결과를 남긴다. 글꼴 바이트는 커밋하지 않는다.

제품 소스는 리베이스된 a9b0d49fd3와 같고 새 지침 커밋에는 제품 변경이 없다.
설치 전 최소 글꼴 전체 Rust 회귀는 13,576 PASS·45 skip, 대상 50쪽 및 유입 14쪽의
Native/fresh WASM 비교는 새 base 대비 하락 0쪽이다. 다만 설치 전 Native 보호 비교에는
Helvetica 1쪽과 nested 4·10·14쪽의 보호 하한 미달이 남았으므로 전체 보호 통과가 아니다.
설치 후에는 같은 프로그램·설정을 공급한 고정 base/head 대조부터 재검증한다.
사용자 지시대로 PR 생성·push는 하지 않는다.


## 2026-11-15 17:44 KST — nested 4쪽 화살표의 실제 Print 대체 확인

설치 후 남은 Native4쪽99.94%를 확대 판독했다. 50개 raw 미대응 픽셀은
`⇩`(U+21E9) 두 개에 모여 있었다. 원본은 굴림체를 요청하지만 실제 Native는
D2Coding을 선택했고, 독립 PDF는 두 문자 모두 Haansoft Batang을 사용했다.
PDF 기준선은671.3862915·708.3059082pt다. 기준 PDF를 글꼴 공급원으로 사용하지 않았다.

기존 비교 환경에 굴림체/GulimChe의 누락 문자 fallback을 Haansoft Batang으로
명시했다. 원본 프로그램·전진폭 소유는 유지하며 제품 코드나 판정 기준을 바꾸지 않았다.
동일 환경의 새 base와 후보에서 전17쪽을 Native/fresh WASM 각각 재측정했다.
4쪽은 네 실행 모두100.00%이고 Native 전17쪽의 보호 하락은0쪽이다.
다만 WASM11쪽은 두 source 모두99.09%/보호99.10%로 남아 전체 보호 통과가 아니다.
Helvetica 대조군은 PDF에서 확인한 Arial 설정으로 base/후보 Native가 모두100.00%다.
[비교 조건·전쪽 TSV·4쪽 review/overlay](../pr/assets/issue7732_split_span_20261115/post-rebase-print-font-conditions/comparison.json)를 보존한다.

사용자가 요청한 보호 검증용 ZIP은 output 아래에 생성한다. 보호18종·각1771쪽의
글꼴 의존 기록에서 실제 적재 프로그램과 동일 family 선택 후보를 모았고, Native 직접
공급 파일을 합쳐 SHA 중복을 제거했다. OS 전체 글꼴을 묶지 않으며 비공개 바이너리는
커밋하지 않는다. ZIP 생성은 전체 보호 검증의 통과나 PR 제출을 의미하지 않는다.

## 2026-11-15 18:42 KST — WASM 11쪽 하락의 선행 보정

사용자 지시에 따라 전체 WASM 검증보다 nested 11쪽의 99.09%/보호값99.10%
차이를 먼저 해결한다. PDF raster·원본 굴림 윤곽·전진폭 대조로 기준 변환과 글꼴
프로그램 차이를 배제한 뒤, 하단 밑줄 끝의 실제 초과 픽셀을 확인했다.
독립 Print의 끝점은532.6339722pt(710.1786296CSS px)다. 현재 출력은 본문의
마지막 가시 구간 뒤에 분리된 ASCII 공백까지 밑줄을 그어 약716px까지 이어진다.

`LayerBuilder`의 기존 트림 검색은 마지막 비공백 TextRun 안의 공백만 처리하고,
별도 공백 전용 TextRun은 건너뛰었다. 원 도입 `da604b2943`의 공통 발행 경로와
기존 #6117·#6028 계약을 대조했다. 분리된 공백도 같은 soft-wrap 꼬리로 처리하되,
문단 끝·명시적 줄바꿈·공백 전용 입력란은 보존하도록 공통 paint 생성을 보정한다.
문서별 좌표·점수·판정 기준·기준 PDF는 변경하지 않는다.

`issue_6117_layer_decoration_trim`의 관계 검사7건은 보정 전 실제3PASS/4FAIL이다.
첫 실행의 인자 오류와 파생 suite 준비 전0건 실행은 검사 결과로 사용하지 않는다.
실제 실패 로그는 `rebase-9088/p11-softwrap-before-prepared.log`에 보존한다.
보정 후 최소 글꼴 회귀와 새 Native/fresh WASM 11쪽 출력을 순서대로 확인하며,
이 쪽이 보호값을 복원하기 전에는 전체 WASM 완료나 PR 준비 완료를 주장하지 않는다.

보정 후 최소 글꼴 검사7건은 실제7PASS/0FAIL(0.018초)이며, 새 Native·WASM 빌드와
11쪽 실제 출력 검증을 시작했다. [집중 검사 증적](../pr/assets/issue7732_split_span_20261115/soft-wrap-decoration/focused-tests.json)을 보존한다.

## 2026-11-15 18:58 KST — 11쪽 보호값 복원 확인

`2c90fa288b` 새 Native·fresh WASM 빌드에서11쪽을 canonical Visual Sweep으로
실제 재출력했다. Native99.08498→99.08811%(공식99.08→99.09),
fresh WASM99.09499→99.09810%(공식99.09→99.10)로 보호값을 복원했다.
전후 render tree·PDF PNG는 동일하다. review·overlay 및 밑줄 끝 확대를 직접 판독했고,
분리된 공백까지 이어지던 초과 밑줄이 제거됐음을 확인했다.
[보정 증적](../pr/assets/issue7732_split_span_20261115/soft-wrap-decoration/README.md)을
보존한다. 그 뒤 새 소스의 nested 전17쪽과 최소 글꼴 Rust 게이트를 시작했다.
전체 보호 검증과 PR 준비 완료는 아직 아니다.

## 2026-11-15 19:34 KST — 전체 회귀 통과와 실행 환경 대조

최소 글꼴 전체 회귀는13580PASS/0FAIL/45skip,921.437초로 완료했다.
추가 #6028·#6451 밑줄 반례2건과 fmt·Native/WASM Clippy·workspace build·all-target
Clippy·manifest·unit tier도 통과했다. 파생 suite 선택이 달라0건 실행된 첫 보조 호출은
무효로 보존하고 다시 생성한 실제 suite에서2PASS를 확인했다.

잔여 issue7905의1쪽은 같은 글꼴·render tree·PDF에서 Mac GPU99.97%,
소프트웨어99.98%다. CDP로 Mac 기본의 가속·Graphite 활성과 소프트웨어 프로파일의
비활성을 확인했다. 원 기준 호스트의 현재 기본 headless도 소프트웨어였으며,
그 조회에서는 문서를 출력하지 않았다. 모든 제품 검증은 Mac에서 수행한다.
같은 소프트웨어 조건의 고정 base도99.98%를 확인했고, 후보는 전6쪽 보호 하락0이다.
점수·PDF·글꼴·판정 기준은 변경하지 않으며 이 조건으로 전체 보호 검증을 계속한다.
원 GPU 결과와 실행 조건은 자산에 별도로 보존한다.

사용자는 모든 테스트 완료 뒤 PR·CI·병합·후속 처리까지 승인했다. 필수 결과가
남아 있는 동안 제출하지 않으며, 이전 준비 후 대기 지시는 이 승인으로 갱신됐다.
