# PR #8475 — 세로쓰기 칸의 합성 줄 소유권 보정

## 범위와 독립 근거

원 [PR #8475](https://github.com/edwardkim/rhwp/pull/8475)의 새 동작은 세로쓰기 칸이
있는 표를 쪽 경계에서 나누지 않는 것이다. 원 코드 `9169400766`과 증거 커밋
`c01f286aca`를 구분한다. 원 PR도 세로쓰기 표현의 기존 차이를 공개했다. 통합 후보의
직접 판독에서는 표의 한 쪽 소속뿐 아니라 같은 주장 칸의 글자 보존을 함께 확인한다.

입력은 `samples/issue8470/vertical_fit.hwpx`, 기준은 기존 한컴 Print
`pdf/issue8470/vertical_fit-2020.pdf`다. 원본이나 기준 PDF를 재생성하지 않는다.
원본은 공개된 생성기의 HWPX이며 실제 한컴 Print와 대조한다. 합성 메타데이터만으로
정상 한컴 저장 줄의 수용 조건을 완화하지 않는다.

`Contents/section0.xml`의 세로쓰기 칸은 `textDirection="VERTICAL"`, 5행 0열,
`rowSpan="3"`, `cellSz` 3000×5547HU이고 텍스트는 `임용전경력`이다. 저장 LineSeg가
없다. `Contents/header.xml`의 charPr 0은 1000HU·장평 100%, paraPr 0의 문단 여백은
0이다. 한컴 Print 5쪽에서는 다섯 글자가 하나의 세로줄로 모두 보인다.

## 실제 관측과 소비 경로

2026-11-05 직접 연 Native/fresh WASM review와 별도 overlay에서 표 V6 행1~30은
5쪽에 모두 남지만 병합 칸은 두 세로줄로 보이며 마지막 `력`이 보이지 않는다.
Native 98.75%·WASM 98.72%라는 페이지 점수만으로 이 차이를 면제하지 않는다.
실제 파일·해시는 `mydocs/pr/assets/planet6897_20261103/direct-reading-current.json`의
PR 8475 기록에 있다. 기존 PNG의 producer `4b149b6cdf`와 새 측정 제품 `c183a1ac61`을
구분한다.

새 Native 제품의 5쪽 render tree에서 병합 칸은 약 40×74px다. 다섯 TextRun이 모두
트리에 있지만 글줄이 `임용`·`전경`·`력` 세 열로 나뉘고 마지막 열은 칸 왼쪽 밖에 있다.
이 진단 트리는 Print PNG와 별도 프로필의 진단이므로 직접 PNG 판독을 대체하지 않는다.

추적한 경로는 다음과 같다.

1. `DocumentCore`의 셀 LineSeg 재조판은 `cell_inner_width`로 합성 줄을 발행한다
   (`src/document_core/commands/document.rs`의 셀 내부 문단 reflow).
2. `layout_vertical_cell_text`는 원본을 다시 compose한 뒤 칸 높이를 줄 예산으로 전달한다
   (`src/renderer/layout/table_cell_content.rs`). 일반 표와 부분 표가 이 함수를 공유한다.
3. `recompose_cell_lines_in_frame_with_storage`는 편집되지 않은 합성 다줄 LineSeg를
   먼저 반환해 보존한다 (`src/renderer/composer/stored_cell_reflow.inc.rs`).
4. 높이 축으로 재분할하기 전에 이 반환이 실행되면, 가로 축으로 만든 합성 줄이 그대로
   세로 열이 된다. 실제 배치는 그 열의 폭을 오른쪽에서 왼쪽으로 누적하고 셀 viewport가
   바깥 열을 잘라 마지막 글자가 숨는다.

이 경로는 코드에서 확인한 원인 가설이다. 수정 전 정식 관계 검사로 재현한 뒤 확정한다.
현재 실행 중인 전체 회귀의 소스를 바꾸지 않으며, 종료 후 보정과 영향 검증을 진행한다.

실제 `dump --section 0 --para 45`에서도 원본에 없던 합성 줄의 문자 시작이 0·2·4,
각 줄 폭이 2716HU임을 확인했다. 이는 셀 가로 폭 3000HU에서 좌우 여백을 뺀 격자의
폭이며 칸 높이 5547HU 축이 아니다. 원본 XML에는 LineSeg가 없다. `vertical_fit`과
`vertical_oversize`의 현재 5쪽 출력 트리에 같은 단일 세로줄·셀 포함 관계를 적용했을 때
둘 다 실패했다. 이 관계 진단을 새 Rust 함수의 실행 결과로 표시하지 않는다.

기존 장시간 검증의 소스는 고정하고 ignored output의 별도 detached worktree
`worktree-vertical-cell`에 새 이름 `issue_8470_vertical_cell_content_axis.rs`를 준비했다.
공유 검증 target은 그대로 사용한다. 현재 Cargo·Native 집중 묶음 종료 뒤 실행하도록
의존성을 걸었으며, 실제 수정 전 실패를 확인하기 전에는 보정 성공으로 표시하지 않는다.
`logs/vertical-axis-before-*`에 명령·종료 코드와 예상 실패 여부를 구분한다.

## 보정·검증 계약

원본의 유효한 세로쓰기 저장 줄은 보존한다. 가로 축에서 생성된 내부 합성 캐시는 원본
저장 줄로 승격하지 않고 실제 칸 높이 축에서 재구성한다. 일반/부분 표와 Native/WASM이
같은 줄 구성 결과를 소비하게 한다. 문서 ID나 글자별 고정 px, 출력 숨김으로 우회하지 않는다.

`vertical_fit`·`vertical_oversize`의 다섯 글자 수·순서·동일 세로줄·셀 내부 포함과
기존 표의 쪽 소속을 함께 검사한다. 정상 가로쓰기 대조군과 원본 저장 LineSeg 경로도
보존한다. 모든 검사는 최소 글꼴에서 동작해야 한다. 수정 전 실패와 수정 후 통과를 구분하고,
기존 Print의 Native/fresh WASM 전쪽 90% 이상·확대 판독 뒤 회귀를 확정한다.

아직 구현·재검증을 마치지 않았으므로 최종 수용·PR 제출은 보류한다.

## 측정 소비 지점의 추가 확인

`height_measurer.rs`의 두 세로쓰기 측정 분기는 `max(LineSeg.segment_width)`를
세로 점유로 읽는다. 잘못된 가로 폭 캐시를 남긴 채 배치에서만 새 열을 만들면 측정과
실제 배치가 같은 근거를 소비하지 않는다. 따라서 배치 helper만 바꾸는 것으로 끝내지 않는다.

`DocumentCore`의 로드·명시적 재조판·중첩 빈 셀 재조판에서 공유할 작은 글줄 프레임
소유자를 만들고, 가로쓰기에는 기존 셀 폭·좌우 여백 계약, 세로쓰기에는 셀 높이·상하 여백
계약을 적용한다. 세로 축을 알 수 없으면 가로 폭으로 합성해 원본인 것처럼 남기지 않는다.
실제 배치는 확정된 칸 높이로 내부 합성 캐시를 재구성하고 유효한 원본 저장 줄은 보존한다.

수정 전 재현용 detached worktree `worktree-vertical-cell`의 소스는 그대로 두었다.
별도 `worktree-vertical-cell-fix`에서 보정 코드를 작성 중이며, 두 폴더의 Cargo는 공유
`target/pr-review`에서 기존 검증 종료 뒤 순차 실행한다. 큰 자료는 원본 경로에 그대로 두고
이 작업 소유의 symlink만 사용하며 symlink를 저장소 자원으로 커밋하지 않는다.

## 실제 보정과 집중 회귀 결과 — 2026-11-05

앞 절의 작성 중 상태에서 보정 소스 `37e5bba3007614d9304c3dbf863503a8c7340405`를
별도 worktree에 커밋했다. 현재 위치는
`/home/tsjang/rhwp-worktrees/review-pr8475-vertical-axis-fix`다. 기존 통합 소스의 장시간
측정은 고정 제품으로 계속하며 ROOT의 Rust 소스를 이 보정으로 덮어쓰지 않았다.

- 생산: `document/cell_line_frame.rs`를 로드·중첩 셀·명시적 재조판의 세 경로가 공유한다.
  가로쓰기는 기존 셀 폭·좌우 여백 계산을 유지하고, 세로쓰기는 원본 셀 높이·상하 여백으로
  글 진행 축을 만든다. 높이를 알 수 없을 때 가로 폭으로 대신하지 않는다.
- 소비: `composer/vertical_cell_lines.rs`가 내부 합성 줄을 최종 칸 높이에서 구성한다.
  일반 표·부분 표의 기존 `layout_vertical_cell_text`가 이를 함께 소비한다. 유효한 원본
  저장 줄과 별도 개체·어울림 흐름이 소유한 줄은 이 합성 캐시 제거 대상으로 삼지 않는다.
- 관계 검사: 다섯 글자의 동일 세로줄·셀 내부 포함·순서·누락·중복을 확인하고, 모델에
  남기는 합성 줄도 같은 단일 세로줄을 나타내는지 검사한다. 절대 px나 특정 글꼴 폭으로
  기대값을 맞추지 않는다.

[실제 수정 전후 실행 원장](../pr/assets/planet6897_20261103/pr_8475/vertical-axis-focused-37e5bba300.md)에
검출 함수의 동일 SHA와 각 로그 SHA를 기록했다. 동일 검출 assertion으로 수정 전
0 PASS·2 FAIL, 수정 후 2 PASS·0 FAIL이다. 기존 표 나눔 4개와 정상 저장 세로줄 1개도
통과했다. 모두 공개 글꼴 5개만 노출한 최소 글꼴 환경과 기본 컴파일 옵션을 사용했다.

검증 폴더의 `llms.txt` 누락과 안쪽 Cargo workspace 참조 오류는 환경 실패로 구분했다.
후보를 복사한 뒤 baseline을 먼저 컴파일한 탓에, 공유 target의 상대 경로 dep-info와 파일
mtime 판정으로 이전 라이브러리를 Fresh로 재사용한 초기 실행도 후보 결과에서 제외했다.
새 helper가 실제 컴파일 의존성에 포함됨을 확인한 뒤 기본 옵션으로 재컴파일해 위 결과를
얻었다. 진단용 추가 cfg 실행을 기본 옵션의 통과 증거로 쓰지 않는다.

기존 저장 줄 정상 대조군 `samples/issue6029/3200477_icao_procedure.hwpx`와 기존 한컴
Print PDF 1쪽의 보정 전 측정은 Native 97.97%·fresh WASM 98.75%다. 이 기준도
보정 후 다시 대조한다. 원본 바이트를 훑은 보수적 영향 상위 집합은 현재 80쌍 중
PR #8475의 3쌍이며 보호 범위 17쌍 중에는 없다. 이 소스 분석을 출력 측정으로 표현하지 않는다.

아직 보정 후 Native/fresh WASM의 실제 전쪽·확대 판독과 필수 전체 검증을 완료하지 않았다.
집중 검사 7개 통과만으로 최종 수용이나 PR 준비 완료로 판정하지 않는다.

## 보정 후 Native 전쪽·직접 판독 — 2026-11-05

보정 소스 `37e5bba300`의 기본 옵션 Native 제품으로 6입력·29쪽을 다시 측정했다.
`vertical_fit`·`vertical_oversize`는 각각 5쪽 모두 raw 100.00%다. 실제 review와 별도
overlay를 대조해 마지막 글자 복원·동일 세로줄·셀 내부 포함·앞뒤 행·표 종료를 확인했다.
나머지 27쪽은 직전 후보의 Native PNG와 바이트가 같고 두 문제 쪽만 바뀌었다.
실물 16쪽 최저는 94.92448%, 저장 줄 정상 대조 1쪽은 97.97401%로 유지됐다.

[보정 후 Native 전쪽·판독 증거](../pr/assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/native/README.md)에
실제 입력·PDF·제품 SHA·TSV·PNG와 관측을 연결했다. 실물 10쪽의 유효한 저장 세로줄
간격과 정상 대조의 연결 화살표 등 기존 차이도 별도로 적었다. 이들은 직전 후보와 같은
출력이며, 저장 줄이 없는 축약 입력의 누락 복원을 전체 기존 표현 차이의 해결로 쓰지 않는다.

최적화 WASM 제품 빌드가 실제 종료한 뒤 공유 Cargo target의 필수 전체 검증을 순차
실행하도록 연결했다. 이 절은 Native의 완료 기록이며 fresh WASM·전체 Rust 통과나
통합 브랜치 적용 완료를 선행 기록하지 않는다.

## 보정 후 fresh WASM 전쪽·판독 — 2026-11-05

최적화한 동일 보정 source의 fresh WASM 6입력·29쪽도 완료했다. 두 축약 입력의 각
5쪽은 모두 100.00%이며, 실물 16쪽 최저 96.96022%, 저장 줄 정상 대조 98.74661%다.
두 문제 쪽만 개선되고 나머지 27쪽의 PNG는 c183 WASM과 바이트가 동일하다.
Native/WASM 각각 두 문제 쪽·실물 10/11쪽·정상 대조 1쪽의 review와 별도 overlay를
직접 확인했다. 기존 저장 세로줄 간격·기호 차이와 합성 줄 누락 복원을 구분했다.

[전쪽·직접 판독·완료 조건](../pr/assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/README.md)에
결과를 연결했다. 보정 source의 전체 Rust·필수 편람 및 ROOT의 기존 PR 보호 WASM은
아직 진행 중이며 최종 수용·통합 적용·PR 제출 완료로 표시하지 않는다.

## 공간 부족 복구와 필수 편람 완료 — 2026-11-05

첫 필수 편람 실행은 전체 구간 측정 뒤 ENOSPC로 집계·보고서 저장에 실패했다. doctest의
동시 링크도 Bus error로 실패했으며 둘 다 통과로 기록하지 않았다. 실행 중인 Cargo·Rust가
없는 것을 확인한 뒤 이 검토 소유 증분 캐시 4개만 정리해 약 7.3GB를 확보했다.
같은 코드의 doctest를 동시 링크 1개로 재실행해 8 PASS·3 IGNORE·exit 0을 확인했다.
기대값·컴파일 옵션·Cargo 8잡·글꼴 환경을 바꾸지 않았다.

동일 제품·입력·PDF·글꼴의 완료 구간을 canonical resume으로 재사용하고 실제 편람
게이트 집계를 다시 실행해 383쪽·최저 99.96%·기준 하락 0쪽·fallback 0건을 확인했다.
복구는 약 64초에 끝났다. [전쪽 TSV·명령·SHA와 복구 근거](../pr/assets/planet6897_20261103/pr_8475/vertical-axis-37e5bba300/handbook-validation.md)에
결과를 보존했다. 전체 Rust nextest·Skia 묶음은 실패 단계 뒤에서 이어 실행 중이며
최종 통과로 표시하지 않는다.

## 2026-11-05 최종 통합 검증과 회귀 측정 주석 보완

최신 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`의 전체 최소 글꼴 nextest 12,583 PASS·Skia lib 4,040 PASS·CI Native 21개·doctest 8 PASS를 포함한 필수 실행 41개가 모두 exit 0으로 종료됐다.
`tests/cases/issue_8470_vertical_cell_content_axis.rs`의 개발 중 측정 주석을 실제 Native/fresh WASM 두 입력 전 5쪽 100.00%와 직접 판독 결과로 갱신했다. 검사·helper 본문의 SHA를 대조해 실행 코드가 같음을 확인했다.
독립 rustfmt와 unit-tier·manifest 정책을 다시 확인했다. 주석 바이트 수에 따라 갱신한 파생 harness의 source module 다중집합은 같으며 생성 파일을 제출하지 않았다. 주석 때문에 전체 회귀나 시각 산출을 반복하지 않는다.
[실제 실행 41개·출처](../pr/assets/planet6897_20261103/rebase8609/rust-regression-validation.md)와 [실제 Studio 소비 화면](../pr/assets/planet6897_20261103/rebase8609/studio-consumers.md)을 연결한다. 새 base 제품과의 기능 보존 대조 후 최종 판정한다.
