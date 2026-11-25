# #9318 리베이스 후 새 base·후보 검증

새 base는 `622af208ee12c4cb16357d465e4802e7fd4726bc`, 리베이스 직후 head는
`2393eb7ae6041d40489295b914c5b8d8dc93f827`이다. 75개 커밋 중 72개 patch는 동일하다.
3개 patch 차이는 corpus 원장 정렬·등록 정보의 적용 순서이며 제품 구현 차이가 아니다.
기준 파일 4개의 중간 충돌은 시작 전 branch와 새 base를 Git으로 최종 자동 병합한 blob으로
해소했다. 리베이스 최종 전체 tree가 그 독립 자동 병합 tree
`3bd2061f201d32ad716d2868c216796a5ec8d3b5`와 정확히 일치함을 확인했다.
[리베이스·공간 확보 증적](../pr/assets/issues7754_7765/source52-pr9318-rebase/README.md)

첫 해소 시도는 기존 중복 행을 오류로 판정한 검사 실패 뒤 후속 명령이 이어져 중단했다.
해당 rebase만 abort해 원 head와 clean 상태를 확인한 뒤 다시 적용했다. 두 번째 시도는
파일 해시·JSON 구조·충돌 marker 검사를 먼저 완료한 경우에만 stage·continue했다.
실행 중 Cargo/Rust가 없고 열린 파일 참조·잠금이 없는 고정 target의 증분 cache 121개만
정리했다. 예상 할당량은 14,226,780,160 bytes이고 이후 여유 공간은 약 14.9 GB였다.
프로필·deps·fingerprint·실행 파일·기존 검증 결과는 보존했다.

검증 경로는 collaborator 본인 PR 준비의 `collaborator_self_merge`와 `local_validation`,
조판 증거의 `visual_fixture_evidence`를 따른다. PR 번호·review·오늘할일을 새로 만들지 않는다.
원 TTF → 실제 glyph를 확인한 TTC → CFF 순서, 정본 보호 TSV와 전쪽 98% 이상·가급적
100% 목표를 유지한다. #9316의 Windows 직접 PDF 기능 검사는 사용자 지시대로 제외한다.

새 base·후보의 clean 전용 worktree를 사용하고 고정 `target/pr-review`에서 Cargo를 직렬로
실행한다. 각 역할을 빌드하기 직전에 tracked production 파일의 바이트를 보존하며 mtime을
갱신한다. 실제 rhwp 재컴파일·source hash·frozen Native binary/rlib·최적화 fresh WASM을
확인한다. 예전 Source88의 잘못된 캐시 재사용을 반복하지 않도록 두 역할의 프로그램 hash와
규제분석 HWPX 53쪽의 실제 개선을 작은 선행 검사에서 확인한다.

원 PR #9240·#9269·#9278·#9282·#9287의 주 원본과 #547 보조 원본 6종 167쪽을
새 base·후보의 Native/fresh WASM 양쪽에서 같은 독립 Print·입력·글꼴 조건으로 대조한다.
표 배경·캡션 표·통째 이월·끝 조각 빈 줄·표 뒤 도형 줄 간격·OLE/EMF·WMF mask/DIB와
HFT 표시 대체·빈 CFF 가운뎃점의 공통 경계를 포함한다. 기존 9개 이슈, 전쪽 보호,
암호 문서, 중첩 표 편집·저장·다시 열기, 실제 Studio 소비 검증도 새 producer로 다시 실행한다.
sample16 세 저장본 18·35쪽의 WASM 차이는 새 base·후보를 먼저 대조한 뒤 원인을 보정한다.

Native Skia 기존 CI selector·최소 글꼴 전체 Rust·집중 회귀·모든 Rust lint·workspace build,
fresh WASM 최적화·브라우저·Studio unit/TypeScript와 해당 시각 증적을 완료한다.
같은 head·조건의 Native 보호 편람 HWPX 383쪽 전쪽 TSV가 정본을 통과하면 새 정본에 따라
해당 결과를 편람 gate로 사용하고 중복 실행은 하지 않는다. 전쪽 TSV와 직접 확대 판독을
완료한 뒤 완료 페이지의 좌표 회귀를 100% → 99%대 → 98%대 순서로 다시 확인한다.
필수 미완료가 남으면 PR 준비 완료로 판정하지 않으며, 준비가 끝나도 PR 제출 없이 대기한다.
