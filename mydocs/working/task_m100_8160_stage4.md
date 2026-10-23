# #8160 단계 4 — 전 페이지 회귀와 기존 기대값 정정

## 착수 조건과 권한

- 브랜치: `task/8160-sample16-raw100`. 선행 증거 commit `d311cb14b`.
- source `803046519`의 Native/fresh 표준 WASM 384/384쪽 raw 100%, 양방향 정수 미대응 0을
  먼저 확정했다. 증거는 `mydocs/pr/assets/issue_8160/candidate33-all-backends.json`이다.
- 구현·로컬 단계별 커밋·필수 검증은 계속한다. 원격 push·PR 생성은 결과를 정리해 사용자
  승인을 받은 뒤 진행한다.

## 기존 검사 감사와 정정 순서

1. BCP: 기존 `issue_7642_sample16_2022_paragraph_relations`의 한 줄 기대값은 원본 두
   LineSeg(0/64)·새 Hancom 2020 Print 두 줄과 다르다. 예외는 이미 `f0f434b8f`에서 제거했다.
   기존 함수를 두 줄의 내용·문단 소유·후속 문단·비겹침으로 정정한다. 별도 중복 검사를 만들지 않는다.
2. 글자 폭: 기존 세 단위 검사는 `fixed_glyph_advances=false`인 default를 쓰면서 명시적인
   잉크 폭 맞춤을 기대한다. 명시적인 true 사례의 기존 음수 자간·배분 간격 보호를 유지하고,
   요청하지 않은 일반 잉크 보존은 별도 의미 검사로 확인한다.
3. 전 페이지: 새 독립 Print에서 내용과 기준선 좌표를 직접 추출한다. 실제 원본의 문단·셀
   경로는 주소로 연결하며 현재 출력의 bbox를 독립 기대값으로 복사하지 않는다.
   각 문서 64쪽 모두를 내용·소유·순서·중복·좌표의 실제 실행 대상으로 등록한다.
4. `verified_stored_table_picture_rows`의 HWPX 18쪽 보호는 전 페이지 검사와 병합해
   원본 표·그림·표 세 컨트롤의 소유·순서·셀 주소 계약을 그대로 보존한다.
5. `verified_sample16_2024_print_boundaries` 및 단 첫 여백 검사는 입력이 구형
   `hwp3-sample16-hwp5.hwp`다. 이번 별도 `-2024.hwp`와 바이트가 달라 중복으로 삭제하지 않는다.
   parser/save-version 계약과 일반 corpus baseline도 다른 의미를 보호하므로 유지한다.

## 검증 상태

기존 잘못된 기대값의 실제 실패와 정정 뒤 통과, 새 검사별 독립 기대값·수정 전 결함 검출,
최종 lint·전체 nextest·Native Skia·브라우저 검증 결과를 실행 후 기록한다.

## BCP 기대값 정정의 실제 실행

동일한 동결 Native rlib `803046519`에 기존/정정 후 정식 case source를 각각 `rustc --test`로
링크했다. 기존 소스는 BCP `len=2`에 기대값 1로 실패(exit 101, 다른 표 검사는 PASS)했다.
정정 후에는 두 검사 모두 PASS(exit 0, 2.59초)다. 실행 환경은 Linux이며 같은 원본·Print와
전쪽 384개 통과 뒤에 검사만 정정했다. 이 focused 진단은 이후 정식 generated suite·nextest
검증을 대신하지 않는다.

BCP 함수는 원본 LineSeg 문자 경계 `[0, 64]`, 두 줄의 전체 내용 순서·누락·중복, 두 번째 줄
`립`, 각 런의 본문 소유와 후속 문단의 내용·비겹침을 보호한다. 2022 Print 모드도 명시한다.
기존 잘못된 한 줄 함수를 교체해 같은 증상을 검사하는 중복 함수를 추가하지 않았다.

## 전 페이지 관계 검사와 시각 회귀 통합

세 문서 192쪽의 독립 문구 572곳을 새 Print에서 선정했다. 실제 PDF 기준선 ±2px,
쪽의 단일 원문 소유·내용·누락·중복, 원본 문자 범위와 순서·가로 포함을 보호한다.
글상자와 표의 로컬 문단 번호를 본문 주소로 혼동하지 않으며 원본 컨테이너 경로를 다시
해소한다. 같은 주소·문자 시작을 다른 쪽에 중복 배치해도 실패한다. 첫 실행은 동결
Native rlib로 세 검사 모두 PASS, 192쪽, 2.04초다. 정식 nextest 실행은 별도로 남아 있다.

기존 `verified_stored_table_picture_rows`를 공유 helper로 옮겨 새 세 검사에서 실행한다.
기존 소유·내용·셀 주소·앞뒤 비겹침 assertion은 보존하며 HWPX 외 두 HWP에도 적용한다.
구형 HWP를 사용하는 기존 2024 경계·단 첫 여백 검사와 parser/save-version/corpus 보호는
다른 입력 또는 의미 계약이므로 유지한다.

전쪽 실제 paint는 새 `verify_sample16_raw100.py`가 384개 PNG를 정수 비교한다.
384/384 PASS(exit 0)를 다시 확인했으며, 수정 전 candidate31 WASM은 2022 18쪽 15/6,
35쪽 0/67을 검출해 382/384 FAIL(exit 1)했다. 같은 Native33은 대조군이다.
이 결과는 실제 구버전 출력의 결함 검출이며 환경·컴파일 실패를 결함 검출로 세지 않는다.

기존 lib test 빌드에서는 이번 스타일 확장의 `isolated_jamo_font` 초기화가 여섯 곳에
누락돼 컴파일 실패했다. 해당 단위 입력은 선택 글꼴이 없는 대조군이므로 None을 명시해
보완했다. 이 실패를 글자 폭 기대값의 실제 FAIL로 보고하지 않는다. 기존 세 기대값의
런타임 재실행을 진행 중이며 이후 명시 요청/일반 잉크 계약을 정정한다.

## 글자 폭 기대값 정정과 source-side 개수 정책

초기화 누락을 보완한 실제 기존 lib 검사에서는 3개 중 1 PASS·2 FAIL을 확인했다.
실패는 default `fixed_glyph_advances=false`의 결과 None과 과거 기대값 Some(0.5)/Some(1.0)의
불일치다. 세 기존 검사에 명시적인 폭 맞춤 요청 true를 넣어 음수 자간 보호와 양수 배분
간격 제외·음수 간격 보존의 기존 의미를 유지했다.

일반 잉크 보존의 추가 4조건×ASCII pin 2종도 실제 lib harness에서 PASS했다.
중간 후보의 4개 검사 모두 PASS(exit 0, 빌드 9분 31초, 실행 0.053초)다.
source-side 테스트 총량 증가 금지 정책에 따라 새 `#[test]`는 남기지 않고 해당 내용을
기존 폭 맞춤 검사가 호출하는 helper로 통합했다. 정책 재실행은 4,101개·296개 모듈로
exit 0이다. 최종 3개 함수 구성의 정식 실행은 최신 devel 반영 뒤 전체 nextest에서 확인한다.
중간 4개 harness 결과를 최종 3개 구성의 실행 결과로 표기하지 않는다.

CanvasKit 준비 보정까지 포함한 Studio 전체 Node 재실행은 exit 0, 2,363개 중
2,361 PASS·0 FAIL·2 SKIP, 92.369초다. SKIP은 기존 Node 전용 pkg-node 부재의 두 왕복
검사이며 실제 fresh web WASM 전쪽 출력과 구분한다.
