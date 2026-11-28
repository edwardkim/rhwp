# #9506 시작점만 있는 PDF 경로의 가져오기 오류

[이슈 #9506](https://github.com/edwardkim/rhwp/issues/9506). 검증 기준 `upstream/devel`은 `274ae9d4c83154e06da049c90eed6904a8b2ec89`이다.

원본의 17쪽은 Form XObject 안에 `m S` 세 개를 갖는다. 각각 선분 없이 시작점만 있는 열린 경로다. 추출기가 넘긴 경로를 [공통 경로 변환기](../../src/parser/pdf/path_paint.rs)가 래스터로 만들면서 `invalid PDF path`를 반환한다. [#7953](https://github.com/edwardkim/rhwp/pull/7953)에서 추가한 경로 변환과 [#8120](https://github.com/edwardkim/rhwp/pull/8120)의 IR 복원을 확인했다. [ISO 32000-1 §8.5.3.2](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf)는 시작점만 있는 열린 서브패스가 아무것도 그리지 않도록 정한다.

1. 유한한 `MoveTo`만 있는 경로는 IR 개체를 생성하지 않고 다음 경로로 진행한다. 실제 선분·닫힘이 있는 경로는 기존 래스터화와 클립 검증을 사용한다.
2. 시작점만 있는 원본 합성 PDF와 해당 획을 `n`으로 끝낸 대조본을 같은 조건으로 변환한다. 양쪽 2쪽에서 일반 선·채움·둥근 끝의 길이 0 획을 확인하고 HWP·HWPX를 저장해 다시 연다.
3. 로컬 원본 167쪽을 CLI와 새 WASM으로 가져온다. Native 집중 회귀와 서식 검사를 실행해 변경 전 실패와 변경 후 성공을 기록한다.
