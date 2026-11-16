# PR #9012 리뷰 — 딸기 굵기 전진과 미해결 연결 그림 인쇄

## 최종 판정

**승인 — #8957 해결 범위와 정본 보호 18문서 Native/fresh WASM 3,542쪽의 로컬 검증을 완료했다.**
정본의 공식 표시값 하락은 0쪽이며 전체 최저는 양쪽 98.09%다. 드라이버의 비트맵 내부 이동·획 확장을
공통 조판에 반영하지 않았고, 해결된 4.26pt 정렬과 기존 개선을 유지한다.

이는 collaborator self-review 판정이다. reviewer를 지정하거나 GitHub의 자기 PR 승인 제한을
우회하지 않는다. 현재 검토 head의 CI는 성공했으며, 이 기록을 추가하는 trailing head의 실제
required check·fast-pass·mergeability 확인과 일반 병합은 후속 조건이다. 작업지시자는 전체 검증과
하락 보정, self-review·최신 head CI 확인 후 #9012 병합을 명시 승인했다.

## 접수와 고정 기준

| 항목 | 값 |
| --- | --- |
| PR·작성자 | [#9012](https://github.com/edwardkim/rhwp/pull/9012), `postmelee` |
| source | `postmelee/rhwp:task/8957-print-frame-outline`, base `devel` |
| 검토·시각 실행 head | `71c002495603a34af43f238dd2c98354d942a589` |
| 코드·테스트 필수 검증 head | `87a4acee56c4d39c290777a9b6c4bd6f3edbbe57`; 이후 71c0024956까지 문서만 변경 |
| 구현 비교 base | `80b1c354c29b7a02fb2b3c14a274da7f5b653ada` |
| 검토 중 최신 devel | `6217d3cbedc9aba707812e5d4e5ea2a1ed53845a` |
| 최신 base 병합 시뮬레이션 | 종료 코드 0, tree `aa402d2ca788e1c410d6455e47729445644fdada`; 기록 커밋 뒤 다시 검사 |
| 관련 이슈 | [#8957](https://github.com/edwardkim/rhwp/issues/8957) 해결·닫기 대상, [#7774](https://github.com/edwardkim/rhwp/issues/7774) 참조만 유지 |
| 기록 시점 | 2026-11-13 KST, Open·non-Draft, reviewer 없음, MERGEABLE/CLEAN |

검토 중 base가 전진했지만 위 다섯 제품 수정 파일에는 구현 비교 base 이후 변경이 없었다.
문서 기록 때문에 source를 merge/rebase하지 않고, 검증된 head 위에 single-parent 문서 커밋을 잇는다.
작성자 커밋과 기여 정보는 그대로 유지한다. 기록 push 직전 원격 base/head를 다시 조회하고,
최신 병합 트리의 공백·링크·다른 PR의 오늘할일 보존을 검사한다.

## 원인과 실제 소비 경로

딸기 HFT의 보통/굵게 독립 한컴 2024 Print에서 12/18/24pt의 꾸밈 전진을 확인했다.
선언 family와 실제 CFF 공급 프로그램이 서로 다른 목록을 사용해 굵기 전진을 빠뜨린 결함이다.
`legacy_font_advances::has_verified_hft_bold_tracking_family`의 공통 목록을 선언 검사와
`NativePuaFonts`의 실제 프로그램 검사 양쪽에서 소비한다. `style_resolver.rs`의 원본 자격 →
`layout/text_measurement.rs`의 실제 glyph 공급·faux-bold 확인 → 공통 전진 결과 →
가운데 정렬 배치·paint로 이어진다. 실제 Bold, 미확인 family, TTF, 공백은 기존 계약을 유지한다.
샘플 ID나 출력 좌표로 분기하지 않는다. 독립 대조의 수정 전 양쪽 89.84%가 수정 후 100.00%다.

연결 그림은 정상 그림 하나와 미해결 그림 둘이 있는 독립 혼재 Print로 인쇄 설정과 분리했다.
`ImageNode::is_nonprinting_picture_placeholder`는 기존 그림 미지정 또는 채우기 모드가 아닌
외부 경로 그림에 실제 데이터가 없는 경우만 판정한다. `paint/builder.rs:342`와 레거시
`svg.rs:2124`가 같은 판정을 소비한다. RenderTree bbox·저장 줄·캡션·명시 테두리는 유지하고
인쇄 paint에서 편집용 표시만 제외한다. 바이트를 공급한 실제 연결 그림과 정상 내장 그림은 출력한다.
디코딩 실패 전체로 계약을 확대하지 않는다.

자세한 원 해결 PR·통합 SHA·반례는 [구현 기록](../../working/task_m100_8957_stage2.md),
[독립 출력](../assets/issue8957/linked-print/README.md),
[굵기 전진의 역사 근거](../assets/issue8957/tracking/README.md)에 있다.

## 검증 결과

[필수 검증 원장](../assets/issue8957/final-validation/README.md)의 source·producer를 재사용했다.
제출 head에서 다시 빌드한 Native와 편람 TSV는 해당 원장과 바이트까지 같다. 이번 보호 검토에서는
제품 코드·테스트·baseline을 추가로 수정하지 않았다.

| 검증 | 확인 결과 |
| --- | --- |
| fmt·manifest·lint·build | Native/WASM/workspace Clippy와 workspace build 통과 |
| 최소 글꼴 회귀 | 집중 37개, 전체 13,320개 통과; 기존 설정의 45개 제외 |
| Native Skia | rhwp 3,852개·내부 crate 182개, 그림 미지정 2개·직접 PDF 36개 통과 |
| fresh WASM | locked 빌드·wasm-opt 완료한 동일 producer로 실제 브라우저 출력 |
| 표지 4종 | 각 전체 1쪽, Native/WASM 모두 raw 100.00%; 정상 그림·내용 순서 보존 |
| 편람 제출 게이트 | Native 383쪽 `--handbook-gate`, 최저 99.96%, 하락 0쪽 |
| 기존 개선 | 통합 39개 기능 44조건/88측정쪽, 표 경계 8문서/전체 14쪽, 그림 미지정 전체 2쪽의 기준/후보 PNG 동일 |
| 정본 보호 | 18문서·36실행·3,542쪽, 누락·쪽수 불일치·표시값 하락 0쪽 |
| 정렬 | 14자 원점 모두 base와 같고 v0.9.0 대비 x −4.26pt; 첫 원점 `(211.62, 708.82)pt`, 크기 18pt |

[기존 개선 원장](../assets/issue8957/preservation/README.md)의 통합 기능 88쪽은 주장 페이지다.
이를 해당 모든 문서의 전쪽 검증으로 표현하지 않는다. 정본 보호의 sample10은 **HWP 원본 전체
763쪽**이고 사용자 HWPX 763쪽 원본의 전체 검증과는 다르다. HWPX 표지는 독립 1쪽 대조본 및
원본 1쪽 PNG 동일 근거로 검증했다.

[정본 보호 TSV·명령·해시·확대 PNG](../assets/issue8957/protected-review/README.md)에
모든 페이지의 기준/후보 차이를 보존했다. 각 문서/backend의 최종 전쪽 실행 하나만 사용한다.
전체 39개 원본·PDF·연결 그림의 commit 포함 및 내용 해시를 확인했다. sample10 기준 PDF는
96,185,197바이트의 로컬 보관 예외이며 SHA-256·763쪽·원 정본 출처를 원장에 기록했다.

초기 보호 하락은 글꼴·이미지 공급 환경으로 분리했다. 승인된 세 독립 문자표의 한컴 2020 Print로
구형 수식과 한컴바탕을 복원하고 실제 HYHWPEQ/HCRBatang 및 원본 슬롯의 보조 글꼴을 공급했다.
sample10의 family 전체 대체가 본문까지 바꾸는 반례를 확인해 **기호 슬롯의 환경만** 지정하고
양쪽 763쪽을 재실행했다. 공급 환경 수정이며 공통 조판 코드나 드라이버 보정이 아니다.
상용 글꼴 프로그램·복원용 PDF는 재배포하지 않는다. 대상 정답 PDF의 글꼴을 공급에 전용하지 않았다.

## 조판 원칙 준수 검토

| 검토 항목 | 확인한 근거 | 판정 |
| --- | --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 위 필수 원장, 최소 글꼴 환경, 호스트 최적화 fresh WASM, 일반 Open PR | 충족 |
| 기존 이슈·해결 PR의 선행 분석 | #741→#753, #8858→#8865→#8923, #1197→#1252와 보정 `3bd66137`; 정본 #8397·후속 #8896/#8984의 공급 원장 추적 | 충족 |
| 구현 근거와 일반성 | 보통/굵게 6행·정상/미해결 그림 혼재 독립 Print, 실제 프로그램 소유와 공통 ImageNode 판정; 임의 좌표·clamp 없음 | 충족 |
| 측정·배치 일관성 | 선언/실제 프로그램의 공통 family → text_measurement 전진 → 가운데 배치; 그림은 기존 bbox를 공통 paint가 소비 | 충족 |
| 리베이스 반영 PR의 기능 보존 | 구현 단계의 최신 base 반영 후 39기능·표 경계·기존 그림·Print 프레임 기준/후보 대조 원장 | 충족 |
| 최종 출력 소비 계약 | 동일 producer의 Native Skia·최적화 WASM 브라우저·레거시 SVG, 글꼴/이미지 공급·Print profile 명시 | 충족 |
| 분할·이어받기 계약 | pagination·rowspan·continuation·컷/예산 로직은 수정하지 않음; 관련 기존 경계 14쪽 출력 보존 | 비해당 |
| 줄 소속과 점유 높이 | 재조판 대조 입력의 가운데 배치 관계, 저장 줄 원본의 내용·그림 bbox 보존; 줄 높이 생산 경로 변경 없음 | 충족 |
| 사례와 증거의 독립성 | 정상 한컴 Print와 라이선스 확인 합성 CFF/PNG 계약 검사 구분; 대상 PDF 공급 금지 준수 | 충족 |
| 기준값 변경 | 기존 baseline·golden·허용치 변경 없음. 보호 TSV는 기준과 별도 증거 경로 | 비해당 |
| 회귀 추가·변경의 선행 조건 | 같은 독립 대조 원본 전쪽 Native/WASM 100%·확대 판독 후 관계형 회귀 등록; #1197 잘못된 가짜 그림만 실제 합성 PNG로 교체, z-order 관계 유지 | 충족 |
| 90% 통과 후 확대 판독 | 표지 문구·6행·혼재 그림, 수식 분수/괄호/퍼센트·앞뒤 본문, 편람 기호·목차 간격, 중첩 표 화살표, sample10 삼각형 직접 대조 | 충족 |
| 주장과 검증 범위 | 고정 head·producer·전쪽 TSV·직접 판독을 연결; 역사 부분 측정을 전쪽 결과와 구분 | 충족 |

새 회귀는 사용자 설치 상용 글꼴에 의존하지 않는다. `verified_hft_bold_tracking.rs`는 선언과
실제 CFF의 소유·실제 Bold·공백 반례를 검사한다. 연결 그림 회귀는 화면/인쇄 분리와 같은 문서의
내용·배치 보존, 독립 Print의 정상 그림 수를 보호한다. 기존 동일 원본 검사에서 제거할 부적합한
절대 px 핀은 없었고 원본 cs/sw 관계는 유지했다. 코드 검토상 남은 차단 사항은 없다.

## CI와 병합 조건

검토 head `71c0024956`의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/43210504580/job/131172989616)
및 전체 39개 check/status를 재조회했다. 33개 SUCCESS check·정상 조건부 SKIPPED 5개·SUCCESS status
1개이며 실패·진행 중 항목은 없었다. [CI 원장](../assets/issue8957/protected-review/code-head-ci.json)에
각 URL과 결과가 있다. 이 문서의 trailing head는 아직 미래이므로 CI 통과나 병합 완료로 기록하지 않는다.

최신 base/head 재확인 → 병합 트리 공백·링크·오늘할일 보존 → 같은 fork PR branch에 문서-only push →
새 head의 실제 preflight·required aggregate와 충돌 상태 확인 → 정확한 head로 일반 병합한다.
`--admin`이나 reviewer 부족 우회는 사용하지 않는다.

## Merge 후 contributor PR comment 계획

실제 merge SHA와 trailing CI URL, 이 review 및 정본 보호 원장을 연결한 한국어 존댓말 댓글을 남긴다.
표지의 Native/WASM 전후·독립 PDF 이미지와 기호/수식 확대 자료를 merge SHA의 raw URL로 게시하며,
게시 전 형식·SHA를 확인하고 게시 뒤 API 본문·이미지 표시를 확인한다. 자동 2px 실루엣 수치의 의미와
드라이버 차이를 공통 조판에 넣지 않았다는 점, 정본 3,542쪽 하락 0쪽을 적는다.

#8957 자동 닫힘을 확인하고 필요하면 승인 범위에서 수동 닫기·완료 댓글을 처리한다. #7774는 참조만
유지한다. 병합 뒤에는 duration 갱신/보류만 확인하고 제품 검증 CI를 재실행하지 않는다.
review·오늘할일은 원 PR에 포함하므로 중복 기록 PR을 만들지 않는다. 사용자 변경을 보존하며 devel을
fast-forward하고 전용 worktree·임시 산출물을 정리한다. 다른 실행 작업이 사용하는 공유 output/target은
해당 사용 근거와 미정리 상태를 명시한다.
