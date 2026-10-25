# PR #8193 리뷰 — fix(checkout): Windows CRLF·심볼릭 링크 체크아웃에서만 실패하던 로컬 검사를 고친다

## 최종 판정

**메인터너 보정 후 수용 가능 — 이 PR의 통합 검토 범위 로컬 검증 완료.**

원격 push·통합 PR 생성·CI·merge는 아직 수행하지 않았습니다. 사용자 승인 후 진행하며, 원 contributor head와 검증한 통합 head를 구분합니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [8193](https://github.com/edwardkim/rhwp/pull/8193) / planet6897 / devel |
| 원 contributor head | `9b3791da320ba66bbd0532fd69bdd1103ae281b2` |
| 통합 검토 branch | `review/planet6897-20261024` |
| 검증 source / base | `673c1d5f7cdcb5573b793ba2d2abaaedd2602134` / `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a` |
| 현재 제출 후보 | `673c1d5f7cdcb5573b793ba2d2abaaedd2602134` / runtime `38ad63a903` |
| 적용 순서 | 현재 누적 history의 10번째 PR. 원 PR별 적용 SHA는 아래 표 참조 |
| 종료 지시 / 참조 | 원 본문 Closes [] / Refs []; 실제 종료 판정 전에는 실행하지 않음 |
| 원 PR 상태 조회 | 2026-10-25T13:01:34.007187+00:00 / OPEN / MERGEABLE |
| 원 PR reviewer | jangster77 |

## 변경과 검토 범위

Windows CRLF·심볼릭 링크 체크아웃에 따른 검사 실패를 보정합니다.

### 원 기여와 메인터너 보정

Mac의 현재 계약 검사와 원 기여자의 Windows checkout 증거를 구분합니다. 이 Mac에서 Windows 전체 실행을 했다고 보고하지 않습니다.

| 원 기능 commit | 통합 branch의 적용 commit |
| --- | --- |
| `9b3791da320ba66bbd0532fd69bdd1103ae281b2` | `7dce905fc109323fa140ac336adb0f927cc2f983` |

아래 적용 commit은 이 통합 branch의 실제 history입니다. 원 PR source branch는 변경하지 않았습니다.

### 검토할 실제 소비 경로와 회귀 원본

- `rhwp-studio/tests/canvaskit-sfnt-face.test.ts`

LF checkout 계약과 심볼릭 링크/CRLF에 영향받는 테스트의 입력 해석을 보정합니다. CanvasKit SFNT face 테스트는 메트릭·글리프 바이트를 보존합니다. 새 조판 규칙을 추가하는 변경은 아닙니다.

## 검증 입력과 결과

- code head `673c1d5f7cdcb5573b793ba2d2abaaedd2602134`, base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`.
- 공통 통합 검증: 렌더러 집중 회귀 16/16 PASS, MCP 계약 4/4 PASS입니다. MCP 기존 실패는 제외한 sample16의 64쪽 기대값이 남은 오류였고 선택 fixture의 선언된 24쪽 기대값과 연결했습니다. 암호 stdin·비노출 검사는 유지했습니다.
- 최종 전체 회귀:      Summary [1920.075s] 11481 tests run: 11481 passed (46 slow), 45 skipped
- 필수 gate 상태 `complete`. [공통 검증 원장](../assets/integration-planet6897/final-validation-20261025/README.md).
- sample16 계열과 편람 HWP/HWPX는 사용자 지시로 제외하고 기존 검사 삭제를 유지합니다. 현재 판정 범위를 다른 #7445 항목으로 확대하지 않습니다.

## 변경 내용과 대응하는 증적

### 대응하는 checkout 계약 검사

`rhwp-studio/tests/canvaskit-sfnt-face.test.ts`의 정확한 TTC face 선택·standalone face0·SFNT 정렬·잘못된 디렉터리·VTT 제거·DSIG 보존·maxp·레거시 cmap 검사가 Mac frontend unit 로그에서 모두 통과했습니다. 해당 source와 fixture는 현재 head에서 동일합니다. `.gitattributes`의 LF 강제와 fixtureBuffer의 실제 바이트 읽기를 검토했으며 Windows 전체 실행 결과를 새로 생성한 것으로 표현하지 않습니다.

LF checkout 계약과 심볼릭 링크/CRLF에 영향받는 테스트의 입력 해석을 보정합니다. CanvasKit SFNT face 테스트는 메트릭·글리프 바이트를 보존합니다. 새 조판 규칙을 추가하는 변경은 아닙니다.

이 변경은 화면 배치를 입증할 대상이 아니므로 다른 PR의 review PNG를 붙이지 않습니다. 정식 테스트의 입력·assertion·종료 결과가 증거입니다.

## 원격 검토와 후속 처리

통합 PR 생성 전 최신 devel/source head와 제출 바이트를 확인합니다. 게시 후 exact-head CI와 raw 이미지·TSV 다운로드 해시를 검증합니다. 통합이 실제 merge된 뒤에만 merge SHA를 연결해 원 PR의 완료 범위와 comment/close 여부를 처리합니다. 원 contributor의 기여·메인터너 보정·남은 차이를 구분하여 한국어 존댓말로 설명합니다.
