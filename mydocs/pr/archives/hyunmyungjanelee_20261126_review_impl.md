# #9416·#9418·#9420 조판 검토와 보완 범위

## 원 기여를 검토한 시점의 판정

아래는 메인테이너 보완 전에 확인한 결과다. 보완본의 수용 판정과 필수 검증 결과는 [세 PR의 최종 검토](hyunmyungjanelee_20261126_review.md)에 기록했다.

세 원 PR의 병합을 보류한다. 최신 `devel` `cbebdbeeca060efe670cb8702355e4d79cb7e70e`에 세 원 head를 순서대로 병합한 로컬 검토 커밋은 `3f09b6d71d0fd66498edf1039cbcc876a32c4b89`다. 기여자의 제출 원본 22개를 한컴 데스크톱에서 다시 인쇄했다. 원본과 같은 버전·PDF 프린터를 사용했고 총 25쪽이 제출된 기준 PDF와 96 DPI에서 픽셀 단위로 일치했다. [원본별 입력·인쇄본·픽셀 해시](../assets/hyunmyungjanelee-20261126/reference-reprints.tsv)

| 원 PR | 재현한 경계 | 보완 범위 |
| --- | --- | --- |
| [#9416](https://github.com/edwardkim/rhwp/pull/9416) | 같은 Windows 글꼴의 이름만 `굴림`과 `Gulim`으로 달리한 두 한컴 인쇄본은 같다. Native 후보의 일치율은 각각 63.37%와 99.54%다. | 워드 호환 줄 높이가 같은 원 글꼴의 두 이름에서 같은 수직 메트릭을 사용하도록 한다. 공개 표 안 문단의 잔여 차이도 실제 글리프·위치로 분리해 재검증한다. |
| [#9418](https://github.com/edwardkim/rhwp/pull/9418) | 원본 두 쪽은 Native 100.00%다. 그림을 120 HU 옮겨 양옆에 글이 들어가는 경우, 한컴은 같은 줄을 양옆으로 나누고 후보는 문단을 그림 아래로 내린다. 이 대조본은 현재 devel과 후보의 픽셀이 같다. | 줄의 시작 높이가 같은 양쪽 조각을 한 물리 줄로 묶어 칸 높이와 실제 배치에 함께 사용한다. 한쪽만 열리는 원본과 양쪽이 모두 좁은 대조군도 확인한다. |
| [#9420](https://github.com/edwardkim/rhwp/pull/9420) | 표 뒤 공백 하나를 `X`로 바꾸면 한컴 인쇄는 표와 글자를 1쪽에 놓는다. 양수 세로 오프셋에서 현재 devel과 후보는 표를 글자 아래로 밀어 2쪽을 만든다. 같은 파일의 오프셋을 0으로 바꾼 한컴 인쇄본은 양수 오프셋 인쇄본과 전쪽이 같다. | 문단의 문자·표 순서로 표 앞뒤 줄의 소유를 판단해 행 높이 계상과 실제 배치가 이를 공유하도록 한다. 양수·0 오프셋과 공백·가시 문자의 네 조합에서 표와 뒤 글자의 순서를 확인한다. |

세 PR의 원 head에는 GitHub 필수 검사 실패가 없고, 각 원 PR의 요청 리뷰어는 `edwardkim`이다. #9414의 Rust CodeQL 실패는 해당 원 PR에 따로 알렸다. [#9414의 댓글](https://github.com/edwardkim/rhwp/pull/9414#issuecomment-7178890959)

## 독립 대조

- #9416은 원본 `word_hcr_p100.hwpx`의 `Contents/header.xml`에서 글꼴 이름만 변경했다. Windows `gulim.ttc`의 첫 번째 face에 두 이름이 등록돼 있다. PDF에 실제 사용된 13개 글리프의 윤곽과 전진 폭을 모두 확인했다. 같은 파일의 `hhea` 높이는 1,024/1,024 em이다. 후보의 줄 높이 함수는 한글 이름에 1.4924, 영문 이름에 1.3을 적용한다. [글꼴과 PDF의 대조](../assets/hyunmyungjanelee-20261126/font-check.tsv) · [한글 이름 확대](../assets/hyunmyungjanelee-20261126/word_gulim_ko-native.png) · [영문 이름 확대](../assets/hyunmyungjanelee-20261126/word_gulim_en-native.png)
- #9418은 `partial_table_following_text.hwpx`의 그림 가로 오프셋만 1,650 HU에서 1,530 HU로 변경했다. 96 DPI에서 그림 위치의 차이는 1.6px다. 한컴은 첫 줄을 왼쪽의 `F O`와 오른쪽의 `L L`로 나눈다. 두 조각의 기준선은 모두 128.04pt다. 후보의 그림 아래 경계와 뒤 글줄 위 경계는 모두 226.6px다. [원본 확대](../assets/hyunmyungjanelee-20261126/partial_table_following_text-native.png) · [양쪽 대조본 확대](../assets/hyunmyungjanelee-20261126/picture_both_sides-native.png)
- #9420은 공개 HWP의 세 번째 본문 문단에서 표 뒤 UTF-16 공백을 `X`로 바꿨다. 한컴은 오프셋 3,000 HU와 0 HU에서 모두 제목 다음에 표를 그리고 그 아래에 `X`를 놓는다. Native 후보의 3,000 HU 대조본은 표의 위 경계가 983.7px다. 0 HU 대조본은 109.5px에서 표가 시작되고 뒤 글자는 965.9px에 있다. [표와 뒤 글자 확대](../assets/hyunmyungjanelee-20261126/tac_visible_suffix-native.png) · [한컴 텍스트 좌표](../assets/hyunmyungjanelee-20261126/reference-text.tsv)

공개 원본에서 만든 대조본은 한컴 2020 또는 2024의 PDF 프린터로 각각 인쇄했다. [입력·출력 SHA-256과 한컴 버전](../assets/hyunmyungjanelee-20261126/reference-controls.tsv)

## #9416 양방향 저장

HWP 5 표 55의 호환 대상과 표 56의 글자·문단·구역·개체·필드 설정을 함께 추적한다. 한컴 2024에서 굴림 원본을 HWP로 저장하면 대상은 2이고, 다섯 설정은 `000009e3 / 00cdf8ff / 0000004f / 000007ec / 00000000`이다. 이 파일과 HWPX 원본을 데스크톱에서 인쇄한 픽셀은 같다. 기존 rhwp의 HWP 저장은 두 레코드를 0으로 만들어 마지막 본문 줄이 59.34pt 올라갔다. HWP에서 만든 HWPX 헤더 역시 호환 대상을 현재 한글로 고정했다.

헤더의 `compatibleDocument`와 HWP의 두 레코드를 연결하고, 동일한 문서 설정을 조판과 저장에서 사용한다. 한컴에서 저장한 두 형식을 각각 출발점으로 삼아 변환·재열기·재인쇄와 반복 저장을 확인한다. 현재 한글, 한글 2007 및 Word의 개별 설정을 바꾼 대조본도 확인한다.

## #9420과 기존 누름틀 줄의 소유

전체 회귀에서 #8522의 `field_over19.hwpx`에 표 두 개가 9.7px 겹쳤다. 이 문서는 폭을 넘긴 첫 표를 누름틀 시작 줄 다음 줄에 놓는다. 독립 한컴 2020 인쇄와 비교한 Native 첫 쪽은 기준 `cbebdbeeca`와 세 원 기여의 통합 `3f09b6d71d`에서 각각 100.00%, 중간 보완본 `d034f6602e`에서 53.37%였다. 폭이 맞는 대조군은 세 소스 모두 100.00%였다. [원 해결 PR의 통합 검토와 독립 인쇄](pr_8522_review.md)

개체만 담은 문단에서는 `composed_empty_host_tac_line_offset`이 제어 슬롯 소속 줄의 원점을 사용하고, `tac_reconcile`이 그 앞 누름틀 줄을 전진에 반영한다. 중간 보완본의 `place_table_with_text`가 이 줄을 일반 본문 조각으로 한 번 더 방출했다. 양수 오프셋 표 앞에 실제 문단 텍스트를 배치하는 분기에서만 소유 줄까지 텍스트를 방출하도록 고쳤다. 누름틀·실제 앞뒤 글자·공백, 2020·2024 인쇄와 저장본을 함께 대조한다.
