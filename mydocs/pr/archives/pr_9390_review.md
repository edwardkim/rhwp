# PR #9390 리뷰 — 한컴 Print 오류의 MCP 대체 저장본 수용

## 최종 판정

**승인 — 검증된 MCP 대체 저장본의 출처 예외를 문서에 반영했으며 기존 시각 검증 기준을 유지했습니다.**

[PR #9390](https://github.com/edwardkim/rhwp/pull/9390)은 2026-11-24 20:23:55 KST에
`e43780db6eae9e168e2f60a4fa2d4af9e404a1f2`로 병합됐습니다. 제출 head는
`20f06d8b3bb143f2298a142fbc8ce34892d589fb`, 비교 base는
`e961a7877186de626643a6466cbecb137874c530`입니다. 병합은 작업지시자가 다른 환경에서
완료했으며, 누락된 저장소 검토 기록을 이번 후속 기록 PR에서 보완합니다.

## 변경과 검토 범위

AGENTS·CLAUDE와 MCP 사용법, 시각 검증 거버넌스, Visual Sweep 가이드, PR 시각 증적 지침의
Markdown 6개를 변경했습니다. 같은 MCP job의 성공 상태, 엔진별 `pdf_output_mode`, 오류 검출·
재저장 `comment`, 입력·출력 해시와 쪽수로 출처를 확인한 PDF 저장본을 공식 기준으로 수용합니다.

한컴 Print 버그의 출력 경로 예외이며 전쪽 Native/fresh WASM TSV, 쪽수·내용 보존, 90% 하한·
98% 제출 기준, devel 대비 하락 검사와 확대 판독은 유지합니다. 기존 Print 영수증 도구가
복구 결과를 인쇄 성공으로 기록하지 않도록 상태·다운로드 JSON의 별도 보존도 명시했습니다.
[수용 조건](../../manual/verification/visual_verification_governance.md#한컴-print-오류-시-mcp-대체-저장본-수용)

## 로컬 검증과 실제 응답 대조

- 제출 head의 `git diff --check`와 변경 문서 6개의 상대 링크 검사를 통과했습니다.
- 변경 매뉴얼의 메타데이터, UTF-8, 새 앵커 10개와 JSON 예시를 확인했습니다.
- 로컬 제주 문서의 2020·2024 최종 응답은 각각 엔진에 맞는 `hancom2020_pdf_save_as`·
  `hancom2024_pdf_save_as`, 313쪽, 복구 `comment`를 반환했고 다운로드 해시가 일치했습니다.
- 운영 서버 사용자 테스트 job `69ff19d3-4091-4927-9f8a-ff8ae7d85516`은 한컴 `13.0.0.4225`,
  `hancom2024_pdf_save_as`, 313쪽, 7,597,480바이트로 성공했습니다. 처리 시간은 541초였습니다.
  이 응답은 MCP 복구 경로의 근거이며 rhwp 시각 일치율 검증 결과로 인용하지 않았습니다.
- 최신 base와 제출 head의 `git merge-tree --write-tree`는 exit 0이었으며 결과 tree는
  `3256517e33cb080611bfa177295bd183c44f8d40`입니다. 제출 전 본인 열린 PR은 없었습니다.

조판 원칙 준수와 검증 입력 첨부 검토는 **비해당**입니다. 실제 diff에 제품 코드·테스트·fixture·
baseline·CI 실행 정의 변경이 없어 Cargo/WASM·새 Visual Sweep 실행은 요구하지 않았습니다.
검토 기록의 MCP 사례를 새 렌더링 회귀나 보호 기준으로 등록하지 않았습니다.

## 제출 head의 GitHub 검사와 병합 후 실행

[CI run](https://github.com/edwardkim/rhwp/actions/runs/45383304746)은 정확한 제출 head에서 성공했습니다.
preflight 로그의 판정은 `fast_pass=true reason=all-review-only-no-code-impact`였습니다.
`CI preflight`와 최종 `Build & Test`가 성공했고 Rust lint·Native·WASM·archive worker는
변경 범위에 따라 건너뛰었습니다. [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/45383383699)도 성공했습니다.

병합 SHA에서는 [issue 상태 자동화](https://github.com/edwardkim/rhwp/actions/runs/45383808735)만
실행돼 성공했습니다. duration 갱신은 AGENTS·CLAUDE·일반 mydocs만 바뀐 push의 경로 필터에 따라
생략됐습니다. 병합 후 CI·CodeQL·Adapter·Proptest 재실행은 없었으며 수동 재실행도 하지 않았습니다.
[병합 후 실행 정책](../../manual/github_operations.md#병합-후-자동-실행-정책-7070)

## 이슈와 후속 처리

[이슈 #9263](https://github.com/edwardkim/rhwp/issues/9263)은 기존에 닫힌 참조 이슈입니다.
이번 PR은 해당 이슈의 Print 영수증 생성 구현을 변경하지 않았으며 새 close를 요구하지 않습니다.

후속 기록을 devel에 반영한 뒤 원 PR과 참조 이슈에 병합 SHA·실제 검증·남은 사항을 기록합니다.
문서 전용 변경이므로 시각 asset은 추가하지 않습니다. 원 PR 전용 두 브랜치와 후속 기록 브랜치는
최종 SHA·병합·활성 작업 여부를 확인해 정리하고, 기본 작업공간의 devel을 upstream/devel과 맞춥니다.
`output/`과 `target/pr-review/debug`는 후속 처리 시작 시 이미 없었습니다. 최종 확인과 정리 로그는
저장소 밖에 남깁니다. 관련 없는 backup 브랜치와 다른 target profile은 보존합니다.
