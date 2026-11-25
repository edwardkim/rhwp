# HWPX 문단 여백의 선언 단위와 저장 캐시 출처

이슈 #7765의 독립 진단 근거다. 렌더러 전쪽 통과나 새 회귀 등록의 증거로 대신하지 않는다.

- [한컴 저장 원시 레코드](hancom-unit-boundaries.json): 원본 HWPX의 SHA·package/head 판본,
  선택한 문단모양 XML, 한컴 2020·2024 저장 HWP의 SHA·job·원시 tag 25 바이트를 담는다.
  HWP 저장은 PDF Print와 다른 작업이다. 중복 자원이 재번호될 수 있어 확인한 앞쪽 ID만 보존한다.
- [단일 XML 차이별 Print](isolated-print-matrix.json): 셀 폭 5곳, 들여쓰기 단위·값 36곳,
  layoutCompatibility 1곳을 각각 분리한 입력의 실제 한컴 2024 Print를 비교한다.
  들여쓰기 변경만 27·53쪽의 내용 소속을 바꾼다.
- [fallback 값만 바꾼 Print의 동일성](default-only-print-equivalence.json): case의 선언 단위에
  맞춰 default 값만 바꾼 진단 사본과 원본 Print가 96DPI 전64쪽 래스터 바이트까지 같다.
  입력 변경 목록과 SHA, 실제 Print job·제품·경로, 쪽별 래스터 SHA를 보존한다.

86712의 패키지 판본은 1.5이고 head만 1.2다. 문단모양 24의 case는 -1233 HWPUNIT,
default는 -2467이다. 두 한컴 제품의 저장 HWP는 -2466이며, fallback의 홀수 표지를
다시 넣으면 CHAR 단위가 된다. 80168 대조군의 실제 패키지는 1.2이고 두 제품 모두 기존
case 저장값을 유지한다. 헤더 판본만으로 단위 변환을 바꾸지 않는다.

파서는 현재 case 단위를 유지하고, 옛 fallback의 가로 여백은 캐시 인증 출처로만 보존한다.
기존 공통 프레임은 이전 문맥으로 모든 저장 줄 경계를 재현한 경우에만 새 줄과 뒤 문단 원점을
함께 발행한다. 원본 파일·공식 Print를 진단 사본으로 교체하거나 캐시를 일괄 삭제하지 않는다.

구현·실제 검증의 진행 상태는 [작업 기록](../../../../working/task_m100_7754_stage2.md)에 있다.
