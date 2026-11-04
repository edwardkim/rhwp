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
