# PR #9420 검토 · 문단 안 인라인 표와 앞뒤 글자의 쪽 소속

hyunmyungJaneLee의 원 기여 `6485652352171c06446d5ce3ba2540675ce46250`은 저장 줄이 없는 문서에서 양수 세로 오프셋의 표가 종이 아래로 밀리는 회귀를 다룬다. 메인테이너는 표 앞과 뒤에 보이는 문자가 있는 입력까지 검토했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9420) · [원 이슈](https://github.com/edwardkim/rhwp/issues/8463)

같은 원본에서 표 뒤의 공백 하나를 `X`로 바꾸면 한컴 인쇄는 1쪽이다. 원 기여와 이전 `devel`은 표 높이를 두 번 반영해 2쪽을 만들었다. 보완본은 재조판 시 문단의 문자·개체 순서에서 표를 소유한 줄을 찾고, 그 줄에 반영한 표 높이를 배치 단계에서 사용한다. 표 앞 글자, 뒤 글자, 공백과 양수·0 오프셋을 대조했다. [수정 전 두 쪽과 인쇄본](../hyunmyungjanelee_20261126_review_impl.md)

양수 오프셋의 앞 글자·뒤 글자 대조는 보완본 Native에서 한컴과 같은 1쪽이 됐다. HWP 및 HWPX로 저장한 대조본 네 개를 한컴에서 직접 인쇄하니 각각 원본 인쇄와 픽셀이 일치했다. [재인쇄 결과](../assets/hyunmyungjanelee-20261126/final/desktop-pixels.tsv) · [통합 검증](../assets/hyunmyungjanelee-20261126/final/README.md)

전체 회귀에서 누름틀 시작 줄 다음의 두 표가 겹치는 중간 보완본을 발견했다. 글자가 없는 개체 문단에서는 이미 배치한 누름틀 줄을 다시 방출하지 않도록 보완했다. 한컴 인쇄본에서 두 표의 경계 사이에 보이는 간격을 되살렸고 활성 회귀의 기대값을 유지했다. [한컴 인쇄·devel·보완 전후의 표 경계](../assets/hyunmyungjanelee-20261126/final/field-over19-comparison.png)
