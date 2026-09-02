# Stage 228 - 수식 host의 저장 line geometry 앵커

## 목적

Stage 226의 HWPX 수식 host 다음 앵커 보정이 `text_footnote_tail_overpagination.hwpx`를
최신 HWP 2020 MCP 정본 242쪽 대신 241쪽으로 압축하는 회귀를 제거한다.

## 회귀 추적

- 전체 integration test에서 `issue_1733_hwpx_matches_hancom_pdf_page_count`가
  `left: 241`, `right: 242`로 실패했다. HWP 원본은 계속 242쪽이다.
- `git bisect`로 `0bb18defd`(정상)와 `57cb9ca52`(실패) 사이를 검사한 결과,
  최초 회귀는 Stage 226의 `57cb9ca52`다.
- #1733의 수식 host `pi=974`, `pi=977`은 각 control의 선언 높이와 대응
  LineSeg 높이가 모두 일치한다. 다음 source VPOS는 일반 본문 간격이므로
  `current_height`를 되감으면 흐름을 한 쪽 과도하게 압축한다.
- issue2006의 MCP 2020 정본 140쪽을 지키는 `pi=328`, `pi=329`는 다중 수식
  control 중 앞 control의 선언 높이와 같은 위치의 LineSeg 높이가 다르다. 이 경우에만
  실제 수식 조판이 저장 line geometry보다 커져 다음 앵커가 정확한 흐름 끝이 된다.

## 수정 계약

- HWPX 저장 조판, 단일 컬럼, 수식만 가진 host와 같은 물리 본문 안의 양수 다음 VPOS라는
  기존 조건을 유지한다.
- control 수와 LineSeg 수가 같고, synthetic이 아닌 대응 pair 중 하나의 높이가 다를 때만
  다음 source VPOS로 `current_height`를 되돌린다.
- 높이가 일치하는 수식 host는 다음 VPOS를 일반 흐름 간격으로 유지한다.
- 문단 번호, 표 식별자, 폰트명, 페이지 번호, 고정 pixel allowance를 사용하지 않는다.

## MCP 2020 근거와 검증

- `pdf/issue1733/text_footnote_tail_overpagination-{hwp,hwpx}-2020-20260814.pdf`는
  HWP 2020 `PrintToPDFEx` 산출 `PDF 1.7`, 각 242쪽이다.
- `pdf/issue2006/1790387_prep_final_report-hwp2020-20260814.pdf`는 같은 경로의
  140쪽 정본이다.
- scratch 검증:
  `cargo test --profile release-test --test issue_1733 --test issue_2006_1790387_prep_pagination_pin -- --nocapture`
  결과는 3건 통과다.
- 다음 단계에서 #3820 고정 회귀와 전체 integration test를 다시 실행한다.
