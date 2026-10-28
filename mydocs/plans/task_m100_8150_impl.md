# #8150 — Kitty graphics 구현 절편

Issue: [#8150](https://github.com/edwardkim/rhwp/issues/8150)

[수행 계획](task_m100_8150.md)의 CLI·프로토콜·범위를 유지한다. 초기 base는
`2e087a5b4bc1515ad877e8020b30114639a59392`이며 준비한 계획 문서 commit을 이어받았다.

## 1단계 — CLI와 전송 계약

완료: [1단계 보고서](../working/task_m100_8150_stage1.md). 구현 전 명령 부재 FAIL과
구현 후 Native 8건·일반 빌드 3건·기존 Sixel 7건 PASS를 확인했다.

- Native 전용 `renderer::kitty`에 `encode_kitty_png`와 `write_kitty_png`를 둔다.
  전자는 기존 흰색 합성과 Skia PNG 인코더를 사용하고, 후자는 `Write`에 최대 4,096바이트
  Base64 청크를 보낸다. 첫 APC는 `a=T,f=100,t=d,q=2`, 나머지는 `m`/`q`만 가지며 마지막은 `m=0`이다.
- `src/cli/outputs/kitty.rs`는 기존 Sixel의 로딩·옵션·쪽·외부 그림 정책을 따르되
  Kitty PNG를 완료한 뒤 stdout 또는 파일에 기록한다. 문서 파일명·내용을 제어 데이터에 넣지 않는다.
- 공개 명령·help·capabilities를 등록한다. Native feature가 없는 명령은 exit 2다.
- 기존 `tests/cases/` 자동 suite에서 새 CLI help/capabilities 계약의 수정 전 실패를 먼저 확인한다.
  그 뒤 청크 3,072바이트 전후·정확한 배수·패딩·빈 입력·중간 writer 오류·PNG 픽셀·CLI 옵션을 확인한다.
  합성 픽셀을 사용하고 새 실물 fixture·쪽수/좌표 golden은 추가하지 않는다.
- 자동 수집과 Native 실행 경로를 조사하고 소유 분류를 정렬한다. 불필요한 CI 개별 호출은 추가하지 않는다.
- 집중 테스트·변경 파일 fmt·공백을 확인한 뒤 `task_m100_8150_stage1.md`와 소스를 같은 commit에 기록한다.
  광범위 PR 사전 검증이나 실제 kitty 표시 완료로 보고하지 않는다.

## 2단계 — 실제 파일과 독립 복원

완료: [2단계 보고서](../working/task_m100_8150_stage2.md). 전쪽 11쪽의 360/720px 복원 22건,
PNG/Sixel 무회귀, kitty 로컬·SSH 표시와 사용자 Windows 11 WezTerm 직접 SSH의 한글 표시를 확인했다.

- 다음 절편 진행 승인 뒤 공개 문서의 전쪽 360/720px 출력과 독립 APC/Base64/PNG 복원을 대조한다.
  PNG/Sixel의 이전 출력과 같은 조건을 비교하고 원문·producer·font SHA를 기록한다.
- kitty의 직접 표시·저장 파일 재표시·SSH를 각각 실제로 확인하고 terminal 버전·환경·프롬프트 위치를 기록한다.
  사용할 SSH 호스트·접근 권한은 실제 설정을 확인하며 임의로 원격 프로그램을 설치하지 않는다.
- CLI·개별 Kitty 매뉴얼을 작성하고 `task_m100_8150_stage2.md`에 직접 관측과 미실행 범위를 구분한다.

## 3단계 — 제출 전 게이트와 보고

완료: [3단계 보고서](../working/task_m100_8150_stage3.md),
[최종 결과보고](../report/task_m100_8150_report.md). CLI 범위의 전체 회귀·lint·Native 출력 보존과
Sixel/Kitty 폰트 옵션 사용 문서를 확인했다. 원격 작업은 수행하지 않았다.

- 다음 절편 진행 승인 뒤 현행 local_validation 4.3의 CLI 범위에 해당하는 필수 lint·전체 회귀와
  Native 출력 계약을 완료한다. Kitty 모듈은 `native-skia`와 비 WASM target으로 제한되며,
  공통 조판·paint·글꼴·기존 PNG/Sixel producer를 변경하지 않는다. 새 소비자는 Native CLI뿐이다.
  2026-10-22 사용자의 CLI 범위 지적을 반영해 WASM/Studio 실행을 필수 범위로 적었던 표현을 정정한다.
  Rust 공통 lint 묶음의 WASM32 Clippy는 수행하고, WASM 재빌드·Studio 실행·WASM Visual Sweep은
  호출 경로상 비해당으로 구분한다. Native 독립 복원·기존 출력 보존과 직접 표시 검증은 유지한다.
- CLI 명령 매뉴얼·개별 사용 가이드·문서 지도·생성형 에이전트 명령 장을 실제 help/capabilities와 대조한다.
- 단계별 결과와 최종 보고를 커밋한다. PR 준비 지시가 있으면 실제 필수 검증·본문·생성 명령을 모두 준비한다.
- remote push·Open PR 생성은 별도 사용자 승인 뒤 수행한다. 원격 code CI가 녹색인 뒤 self-review와
  필요한 오늘할일을 trailing docs-only commit으로 기록한다. merge·이슈 close도 별도 승인 범위를 따른다.

공용 `target/pr-review`를 순차 사용한다. 최초 전송 계약의 실패 실행과 구현 후 성공 실행은
`output/issue8150/`에 기록한다. 작업 중간 산출물은 종료 정리 때 불필요한 보존본을 남기지 않는다.
