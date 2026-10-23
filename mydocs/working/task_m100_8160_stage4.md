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
