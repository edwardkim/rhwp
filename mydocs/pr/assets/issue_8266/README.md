# 기존 혼합 TAC 배치 결함 #8266

[이슈 #8266](https://github.com/edwardkim/rhwp/issues/8266)의 독립 Print 증거다.
일반 이동 `before.hwpx`·`after.hwpx`는 한컴에서 열리지만, 저장 줄 없는 혼합 도형·
글상자가 RHWP에서 앞뒤 글줄과 겹친다. **두 파일을 손상 문서로 분류하지 않는다.**
별도로 손상 경고를 받은 HWP 두 파일은 사용자 지시에 따라 삭제했으며 복원하지 않는다.

고정 devel `238de6de41a208648f4ade6b494fe433a81c1fce`와 현재 생산 코드에서
Native/fresh WASM raw는 각각 정확히 같다. before는 60.18966%/59.46767%, after는
47.66593%/47.17905%다. 이번 변경의 회귀로 보고하거나 정상 렌더링 증거로 사용하지 않는다.
원문 TSV 8개는 이슈 본문에 header·전쪽 행을 그대로 보존했다.

원본 HWPX는 기존 [이동 표본](../../../../tests/fixtures/inline-control-move/)에 있다.
[독립 Print PDF](../../../../pdf/issue_8266/)는 같은 원본을 MCP engine 2020의 한컴
11.0.0.9136에서 Print한 전체 1쪽이며, [provenance](provenance.json)에 입력/PDF 해시·
job ID·실제 Print 계약을 기록했다. 인증 정보와 중간 실행 로그는 커밋하지 않는다.

[이동 모델 계약](../../../../tests/cases/valid_inline_move_fixture_contract.rs)은 이 두
HWPX의 본문·개체·논리 캐럿 보존만 검사한다. 미달 상태를 정상 쪽수/좌표 baseline으로
고정하지 않는다. Native/fresh WASM 99%대인 차트 HWP/HWPX 네 파일의 쪽수·데이터 검사는
유지한다. 실제 배치 문제는 이슈에서 공통 줄 구성·측정·배치의 같은 frame으로 보정하고,
전쪽 90% 이상·직접 판독을 마친 뒤 관계 회귀를 복원한다.

## 제거 전 검사와 복원 경계

제거 전 검사 모듈 전체와 공유 helper를 `.rs.txt`로 보존했다.
[검사 원문](valid_inline_move_fixture_contract.rs.txt)과
[공유 helper](inline_control_move_contract.rs.txt)는 source commit
`5e71af95aaaa4e18555813a5c94abaf528486086`의 바이트 그대로이며, Git blob·SHA-256·
이전/보존 경로는 provenance에 있다. `.rs.txt`는 자동 Rust 검사 수집 대상이 아니다.
검사 함수는 제거하지 않았고, 일반 HWPX 두 파일을 정상 조판으로 고정하던
`page_count() == 1` assertion만 제거했다. 공유 모델·저장 계약과 차트 네 파일은 유지한다.
손상 HWP 두 파일의 보관본을 만들거나 복원하지 않았다.
