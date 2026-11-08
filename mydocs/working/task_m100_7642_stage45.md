# 이슈 #7642 Stage 45 — 일반 미주 본문 줄의 선행 fit과 간격 소유

Issue: #7642

## 기준과 독립 관측

시작 head는 `c6d8812764fce645ee5a74583078546d11dd5b35`, 정책 base는
`79abd9e49675f4286eaed38197f7e837afb744e7`다. 전체 요청은 52개와 추가 3개,
독립 Print 853쪽이다. 최신 전체 Native/fresh WASM TSV와 전체 회귀는 미완료다.
직전 Native의 다섯 쪽수 불일치 중 두 개는 복구됐고 세 개가 남았다.

구분선 없는 2024년 11월 HWPX 11쪽에서 미주 11의 `△ACD에서 코사인법칙에 의해`
줄이 다음 단 맨 위로 먼저 넘어간다. 독립 Print는 이 줄을 앞 단의 끝에 둔다.
이 한 줄 뒤의 미주 12 제목과 그림이 함께 아래로 밀린다. 현재 측정은 실제
`ColumnContent`를 사용하지만 `advance_for_fit` 실행 뒤의 overflow 검사만 있어
이미 넘긴 줄을 앞 단으로 돌릴 수 없다. 제목에 적용한 선행 실제 fit을 일반
본문 줄에서도 확인해야 한다.

## 그림의 원인 분리

해당 미주 12 그림의 원본은 `BinData/image4.BMP`(1296×916)이며 EMF가 아니다.
HWPX 선언·현재 크기는 가로 16545 HU / 세로 11694 HU이고 자르기는 전체 범위다.
Native SVG의 크기는 220.6×155.92, 독립 Print 이미지의 500 DPI 물리 크기는
220.608×155.904로 같다. 원본 bitmap과 PDF에서 추출한 bitmap을 동일한 물리
프레임에서 대조하면 형태도 일치한다. 실제 Native 캡처의 이미지 프레임과
Print에서 추출한 이미지를 같은 96 DPI 프레임으로 비교한 2px 보조값은 100%다.
이는 그림 비율을 변경할 근거가 없다는 진단이며 문서 시각 통과를 뜻하지 않는다.
전체 문서 좌표에서의 이동은 그대로 결함이다. 단위나 viewBox를 바꿔 이를 숨기지 않는다.

원본·추출 이미지·프레임 진단은 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`
아래 `stage45-graph-source-diagnostic/`에 있다. 프레임 정규화는 원인 분리를 위한
진단이며 전체 페이지 TSV·PR gate에 대입하거나 회귀의 고정 px 선택자로 쓰지 않는다.

## 공통 소비와 묶음 수정

1. 명시적 쪽/단 나눔·`page_break_before`·`keep_with_next`가 없는 한 줄 본문은
   실제 현재 단과 전체 후보의 점유를 먼저 확인한다. 물리적으로 들어가는 줄을
   저장 높이의 추정 누계만으로 넘기지 않는다. 다줄 저장 컷과 별도 원점 계획이
   필요한 float 객체를 이 한 줄 계약으로 바꾸지 않는다.
2. `measure_endnote_column_with_item`이 실제 배치에서 생산한 점유 결과를
   `advance_for_fit` 이전에 소비하고 이동 여부를 후속 높이 계산에도 전달한다.
   파일명·문항 번호·고정 위치·수치 조합으로 조건을 만들지 않는다.
3. 새 미주 제목의 간격 하한은 앞 미주의 실제 마지막 줄 소유에서 결정한다.
   본문/구분선 뒤 첫 미주의 간격과 다른 미주 사이 간격을 구분한다.
   실제 paint와 같은 `starts_note_after_complete_tail`을 사용한다.

## 검증과 남은 범위

관련 수정을 묶은 뒤 fmt·Native를 빌드하고 같은 다섯 원본 및 정상 대조군을
비교한다. 11쪽의 본문 소속·순서·그림 위치와 이후 전체 쪽수를 확인한다.
현재 원본의 문제를 먼저 개선하며 회귀 기대값·baseline을 완화하지 않는다.

수정 방향이 확인되면 요청 전체 Native/fresh WASM TSV와 대표 review를 실행한다.
전쪽 90%와 확대 판독을 충족한 원본에서 #1139 전체 및 관련 회귀를 관계형으로
전환한다. 최신 전체 회귀·필수 lint·정책 검사, PR·CI·머지·후속 처리까지 계속한다.
진단 로그·TSV·JSON·nextrun은 Git에 포함하지 않는다.

## 실행 결과 — 부분 보정 완료

빌드 입력은 계획 커밋 `119691fa7fd5396b94d5d79eb74d18a687b98c1b`와 이 단계의
두 Rust 파일 변경이다. 전체 source SHA-256 목록을 빌드 전에 고정했고 빌드 뒤에도
동일한 입력임을 확인했다. 구현 diff SHA-256은
`5466db9f8754ce4d3c63e7e86e213af19deb5d8da100fd38b3cb854f24136b98`다.

| 실행 | 결과 |
| --- | --- |
| `cargo build --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --bin rhwp` | exit 0, 356.80초 |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --locked --target-dir /home/tsjang/rhwp/target/pr-review -- -D warnings` | exit 0 |
| 기존 쪽수 불일치 다섯 원본의 실제 렌더 쪽수 | 네 원본 22/22, 9/8/7 HWPX는 22/21로 미충족 |
| 구분선 없는 2024 HWPX 10·11쪽 Native Sweep | 최저 95.48355%, 11쪽 65.07700% → 95.48355% |
| 9/8/7 HWPX 10쪽 Native Sweep | 77.29898%, `re_review_required` |
| 2022년 11월 HWPX 정상 대조군 4쪽 | 98.74440%, 21/21쪽 |

보정한 11쪽의 코사인법칙 본문은 앞 단의 마지막 줄에 남고, 뒤 수식은 다음 단에
소속된다. 원본 스타일의 명시적 나눔·묶음 속성도 대조했다. 새로 산출한 review의
그림과 후속 수식에는 미세한 세로 위치 차이가 남아 직접 확대 판독과 원점 분석이
필요하다. 이 차이는 95%라는 점수로 면제하지 않는다.

빌드·lint·경계 분석·대표 review의 재현 기록은 앞서 기재한 ignored 진단 경로의
`endnote-single-line-fit-native-provenance.json`,
`endnote-single-line-fit-native-lint.json`,
`endnote-single-line-fit-boundary-analysis/summary.json`,
`endnote-single-line-fit-focused-review-results.json`에 보존했다. 새 review는
`endnote-single-line-fit-focused-review/all-17/all-17/review/review_011.png`다.

**최종 판정: 부분 보정 완료, 전체 검증 미완료.** 다섯 쪽수 불일치 중 네 건을
해결했지만 전체 55문서·853쪽의 최신 Native/fresh WASM TSV와 확대 판독, 관계형
회귀 변경, 전체 회귀 및 필수 lint 묶음은 아직 완료되지 않았다. 새 PR은 만들지
않는다. 다음 단계에서 남은 9/8/7 HWPX의 단·줄 소유와 그림 원점을 함께 분석한다.
