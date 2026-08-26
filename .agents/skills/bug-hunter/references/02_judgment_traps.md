# 02 — 판정 함정 4종

playbook "판정 함정" 장의 실행 계약이다. 여정보다 먼저 읽는다.
번호를 바꾸거나 다섯 번째 함정을 권위로 올리지 않는다. P05 이후는
실행 함정이지 이 4종을 대체하지 않는다.

## 1. "오라클이 통과했다"는 무손실의 증거가 아니다

각 오라클은 **자기가 보는 축에서만** 참이다. `--verify` 는 IR 을
대조한다. IR 에 없는 XML 래퍼·ZIP 엔트리·`<hp:switch>` 는 영원히
안 잡힌다.

`--verify` · `--verify-pages` 가 4/4 통과한 문서에서 `tabItem` 이
절반으로 사라진 사례가 #3551 이다. 잡아낸 것은 ZIP 엔트리 바이트
대조였고, `header.xml` 이 서로 다른 세 문서에서 **같은 바이트 수
(6,737B)** 만큼 줄어든 것이 신호였다.

엔트리 대조는 **개수가 아니라 이름 집합**이다. 총량은 추가와 소실이
서로를 지운다. `samples/한셀OLE.hwpx` 는 `Preview/PrvImage.png` 가
생기며 `BinData/ole1.ole` 소실을 상쇄해 12→12 였다 (#3557).

코퍼스 사실: samples/ HWPX 77건에서 `--verify` 는 74/74(100%)
통과, 구조까지 무손실은 4/74(5.4%). 다만 검출 94.6% 를 그대로
"손실" 이라고 쓰면 함정 3을 밟는다. `hp:fwSpace` 등은 정규화였고
확정 데이터 손실은 엔트리 소실 4문서였다.

정지: **F09**. 왕복 여정은 IR 통과와 무관하게 이름 집합·엔트리별
크기·태그 개수를 대조한다.

## 2. 상신 전에 devel 을 먼저 확인한다

2026-07-29 전수 재검증에서 열린 이슈 53건 중 17건이 이미 고쳐져
있었다. 이슈가 안 닫힌 것뿐이다.

과거 PR 이 CLOSED 여도 반려가 아닐 수 있다. base 충돌로 닫힌 뒤
리베이스본이 메인테이너 배치 PR 에 cherry-pick 된 사례
(#2927/#2943 → #3205).

정지: **F14**. 파일:라인으로 현재 devel 에서 생존을 확인한다.
커밋 메시지 검색만으로 판단하지 않는다.

## 3. 표본 1건을 전체로 일반화하지 않는다

"한컴은 margin 에 halving 을 하지 않는다"고 단정했다가 정정했다
(#3368). 근거는 한 파일의 21쌍이었고, 한컴 저장본 262개 전량은
반감 관계 16,248쌍이었다.

정지: **F15**. 포맷 계약은 N건 중 M건과 반례 수. 없으면 가설.

## 4. 가설은 구현해서 기각한다 — 음성 결과도 결과다

#3518(페이지 수 64→65)에서 provenance 가드를 원인으로 지목했으나
구현해 보니 증상이 그대로였다. `--verify` 로 IR 차이를 뽑자
`char_shapes` 시작이 전부 −2 였다.

정지: **F16**. 증상이 아니라 IR 차이를 먼저. `--verify-pages` 가
exit 4 로 끊기면 `--verify` 로 우회. 고치지 못한 가설은 이슈에 남긴다.

## 관련 픽스처

- `fixtures/pitfalls.json` P01–P04 가 이 4종
- `fixtures/envelopes/verify_pass_zip_loss.json`
- 예제: [16_oracle_pass_not_lossless.md](../examples/16_oracle_pass_not_lossless.md),
  [17_check_devel_first.md](../examples/17_check_devel_first.md),
  [18_dont_generalize.md](../examples/18_dont_generalize.md),
  [19_reject_hypothesis.md](../examples/19_reject_hypothesis.md)
