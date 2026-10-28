---
kind: snapshot
status: active
canonical: mydocs/pr/pr_7238_review_impl.md
last_verified: 2026-09-17
---

# PR #7238 체리픽 충돌 조정

## 분석

#7230과 #7238이 같은 #7218의 CLI 계약 파일 두 곳을 수정했다. 원본 source를 rewrite하지 않고 최신 devel 위 누적 branch에서 테스트 충돌을 해소한다. production text_editing.rs는 자동 병합되었지만 별도 helper 의미가 남아 있어 "충돌 없음"을 "동작 정합"으로 처리하지 않는다.

## 수정·검증

- insert_page_break_contract.rs: #7230의 중간 분할·봉투 검사를 유지.
- issue_7218_page_break_at_paragraph_start.rs: CLI 3개를 유지하고 #7238 core 4개를 core_contract 모듈로 보존.
- source의 header/footer 생성 helper 수정 유지. 코드 head `3127bcce945b00bf7c767df2fb9da1f67ebb3633`.
- 충돌 조정 포함 focused 27/27, fmt PASS. CLI/core 의미 차이는 실제 CLI로 재확인했고 해결 완료로 판정하지 않음.

## 결과보고·다음 보정 조건

[#7238 검토](archives/pr_7238_review.md) 및 [#7230 검토](archives/pr_7230_review.md)의 보류 조건을 따른다. 이번 회차는 테스트 충돌만 조정했으며 production 동작 보정은 하지 않았다. 단일 저장 속성 계약과 사용자 명령 의미를 정한 뒤 실제 제품 경계 입력 검증을 추가한다. 전체 회귀·원격 CI·push 완료를 주장하지 않는다.

## 메인터너 보정 — 속성 설정과 편집 명령의 계약 구분

### 분석

실제 CLI 첫 문단의 성공 봉투와 저장 `pageBreak=0` 불일치, 다른 break 축 덮어쓰기와 synthesized 플래그를 확인했다. 반면 Studio Ctrl+Enter의 분할을 없애는 근거로 저장 속성 통계를 사용할 수는 없다. Windows 10 PowerShell→Python COM `11, 0, 0, 9136`에서 `First\r\nSecond` 입력 첫 문단 시작의 `BreakPage`를 실행했다. 실제 커서는 `(0,0,16)`(첫 문단 제어 슬롯 포함)에서 `(0,1,0)`으로 이동했고, 2문단/1쪽이 빈 선행 문단을 포함한 3문단/2쪽으로 바뀌었다. 추가 사례들은 기존 Hwp 프로세스가 있어 실행하지 않았다. 이 1건을 전체 경계의 한컴 검증으로 확대하지 않는다.

### 수정·검증

- `insert_page_break_native`는 base의 기존 분할 구현을 복원한다. Studio 명령 의미를 새로 바꾸지 않는다.
- CLI/MCP의 `mark_page_break_at_paragraph_start_native`에 `raw_break_type |= 0x04`, synthesized 해제, reflow/vpos 재계산을 모은다. Section/다단/단 비트를 보존하며 동일 명시적 속성은 멱등이다.
- 실제 CLI 첫 문단의 XML pageBreak/secPr 검사, HWPX/HWP5 저장·재열기, 다른 break 축, synthesized, Studio 분할/반복 회귀를 추가했다. 핵심 새 테스트 4개는 보정 전 4/4 FAIL, 보정 후 PASS. #7218 11개와 관련 focused 20개, 총 31/31 PASS(exit 0). 첫 focused 실행은 1개가 nextest LEAK로 표시됐으므로 통과 수만으로 해당 경고를 숨기지 않는다. 최종 전체 nextest에서는 해당 검사가 PASS(0.021초), LEAK 없이 완료됐으며 전체 9,990 PASS/51 skipped(exit 0)를 확인했다.
- npm HwpCtrl `BreakPage`는 `breakAtCursor` → `break_at_cursor`라는 별도 경로다. 앞선 review의 모든 HwpCtrl이 insert_page_break_native를 소비한다는 설명은 정정 대상이다. 이 경로는 이번 보정으로 변경하지 않는다.

### 결과보고

위 수정과 focused 결과를 사용자에게 보고한 뒤 이 회차를 커밋한다. 최종 Native/fresh WASM 제품 경계·Visual Sweep과 전체 검증은 이 코드 commit 기준으로 진행한다. 아직 전체 회귀·최종 승인·remote push 완료가 아니다.

## 실제 저장본 재검증

보정된 CLI로 커밋 원본 `outline_headings.hwpx --para 3 --offset 0`을 다시 저장했다. 기존 after fixture와 달라진 ZIP member는 `Contents/section0.xml` 하나이며, 대상 제목과 뒤 본문의 LineSeg만 재배치된다. 문단 수·텍스트·pageBreak는 같다. 이 정확한 결과로 기존 `samples/issue7218/outline_headings_pagebreak_after_fix.hwpx`를 갱신하며 다른 이름의 복제 입력을 추가하지 않는다.

`rhwp info --json`의 product는 hancom-office-2020. MCP engine 2020 start/status/download(2026-09-17, job `83ad9df7-756d-4b4b-800e-b6fc442282fd`, 9초 succeeded)로 해당 결과를 다시 PDF 변환했다. 2쪽 모두 기존 커밋 PDF와 144 DPI raster byte가 동일했다. 따라서 기존 기준 PDF를 그대로 재사용하며 신규 PDF 복제는 커밋하지 않는다. Native 실제 저장본 sweep에서도 2쪽/문단 텍스트 보존과 빈 개요 항목 제거를 확인했다. 남은 개요 번호 표시(1.Second 대 2. Second)는 기존 renderer 차이로 구분한다. fresh WASM 재캡처는 최종 검증에 포함한다.
