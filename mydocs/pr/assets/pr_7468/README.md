# PR #7468 통합 보정 시각 증적 — 2026-10-08

최종 16 PNG는 사용자가 첨부한 함초롬바탕 regular/bold를 실제 공급한 Native와 fresh 최적화 WASM의 full sweep 결과입니다. Native source `82aeff6091b6827b4f913d4483ba0a354f06f5ca`, WASM source `282d5d77f6f7167a70047170dbad1a10bcce6ba5`입니다. 이후 source 차이는 test/fixture와 import 줄바꿈입니다. [입력·독립 Print 출처](../../../../tests/fixtures/pr7468/README.md), [검토·실행 명령·font SHA-256·실제 선택 trace와 제한](../../archives/pr_7468_review.md#사용자-제공-함초롬바탕-적용-최종-비교)을 함께 확인합니다.

한컴 2020 Print method 0 / one-up, 인쇄 profile, 96dpi로 전체 4쪽을 두 backend에서 비교했습니다. 전 4쪽 관용 실루엣은 각각 100.00000%, 미달/누락 없음, 예외 없이 90% gate PASS입니다. 엄격한 내용 픽셀 일치는 18.45444–38.23178%이며 실제 글꼴을 공급한 뒤에도 약 1px 획/기준선 raster 잔차가 남습니다. 관용 결과를 완전한 픽셀 일치로 해석하지 않습니다.

16개 모두 직접 판독하여 C/D 줄 배치·빈 줄 삭제 및 Line 041/042의 실제 저장 쪽 경계와 끝 Line 070을 확인했습니다. 새 회귀/fixture 추가 전에 이미 통과했던 초기 gate와 최종 font 재실행을 검토 기록에서 구분합니다.

| 입력·쪽 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| edited p1 | [review](edited-p001-native-review.png) / [overlay](edited-p001-native-overlay.png) | [review](edited-p001-wasm-review.png) / [overlay](edited-p001-wasm-overlay.png) |
| deleted p1 | [review](deleted-p001-native-review.png) / [overlay](deleted-p001-native-overlay.png) | [review](deleted-p001-wasm-review.png) / [overlay](deleted-p001-wasm-overlay.png) |
| stored p1 | [review](stored-p001-native-review.png) / [overlay](stored-p001-native-overlay.png) | [review](stored-p001-wasm-review.png) / [overlay](stored-p001-wasm-overlay.png) |
| stored p2 | [review](stored-p002-native-review.png) / [overlay](stored-p002-native-overlay.png) | [review](stored-p002-wasm-review.png) / [overlay](stored-p002-wasm-overlay.png) |

로그·manifest·중간 JSON·TSV는 ignored `output/pr-review/pr7464-pr7468/`에만 보존합니다.
상세 provenance와 환경의 localhost 폰트 전송 adapter는 검토 기록에 설명했습니다.
새 전쪽 gate를 통과한 정상 대조군과 구분하여 sample16의 기존 미달은 #7445에 남아 있습니다.

## 초기 Native TSV 원본 다운로드

[Native 전 4쪽 TSV ZIP](https://drive.google.com/file/d/1cLI-GC8HA7lu97GfQV5eLaB2JRlfCtsd/view?usp=drivesdk) — 1,592 bytes, SHA-256 `b60c678372c879ec293f81eda00132bc03408bc136f587e7683c29a4238a3177`. GitHub App의 Gist 쓰기가 403으로 거절되어 Drive에 원본 ZIP을 첨부하고 업로드 성공 및 metadata 크기를 확인했습니다. 연결된 Drive 소유자 접근이며 공유 권한을 확대하지 않았습니다. PR 본문에도 이 접근 범위를 명시합니다. TSV는 Git commit에 포함하지 않습니다.

## 최종 Native TSV 원본 다운로드

[함초롬바탕 적용 최종 Native 전 4쪽 TSV ZIP](https://drive.google.com/file/d/10frfxke2w-TAmQGGJjfxEOYpjpb4RxJ9/view?usp=drivesdk) — 2,843 bytes, SHA-256 `f67237b9df86ebde7a46f555d8d5bec3ea73abda67fd15e8571bb466d91ba430`. 정식 PNG-pair 재사용 명령으로 최종 full sweep PNG에서 산출했으며 새 렌더링으로 표시하지 않습니다. ZIP에는 README와 입력별 TSV 3개만 있고 글꼴 파일은 없습니다. 업로드 성공과 metadata 크기 및 owner-only 권한을 확인했으며 공유 범위를 확대하지 않았습니다.
