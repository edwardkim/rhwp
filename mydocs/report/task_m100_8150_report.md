# #8150 결과보고 — Native CLI의 Kitty graphics 출력

- Issue: [#8150](https://github.com/edwardkim/rhwp/issues/8150).
- 브랜치: `feat/issue8150-kitty-20261022`, 기준 devel `2e087a5b4bc1515ad877e8020b30114639a59392`.
- [수행 계획](../plans/task_m100_8150.md), [구현 절편](../plans/task_m100_8150_impl.md)의 1~3단계를 승인받아 진행했다.
- Rust 제품 구현 `2d2d9b1921eaef5f586937739920dbee4689b75d`, 실사용 검증
  `ac851c24e7da0a218d7cefd726fc5907f3658ab5`와 이 보고서가 속한 생성기·사용 문서 보완 커밋.

## 결과와 사용법

`rhwp export-kitty`로 문서의 한 쪽을 Kitty graphics 지원 터미널에 직접 표시하거나
쪽별 `.kitty` 파일로 저장할 수 있다. 기존 Native 래스터와 PNG 인코더를 사용하며
흰 배경에 합성한 색을 유지한다. 최대 4,096바이트 Base64 payload의 직접 PNG 전송으로
SSH 양 끝에서 파일 시스템을 공유할 필요가 없다.

```bash
rhwp export-kitty document.hwp -p 0 --width 360 --profile print
rhwp export-kitty document.hwp -o output/kitty --width 720
rhwp export-kitty document.hwp -p 0 --font-path /실제/글꼴/디렉터리
```

`native-skia`가 필요하며 stdout 기본값은 첫 쪽이다. 파일 모드에서는 쪽 생략 시 전쪽을 저장한다.
쪽은 0부터 시작하고 반복 글꼴 경로·기존 프로필·2022/2024 호환 옵션을 지원한다.
그래픽 stdout과 진단 stderr를 분리하며 JSON은 지원하지 않는다. 성공 0, 실행 실패 1, 사용법 오류 2다.

[CLI 매뉴얼](../manual/cli_commands.md), [Kitty 가이드](../manual/export_kitty_command.md),
[Sixel 가이드](../manual/export_sixel_command.md), [에이전트 명령 장](../manual/agent_codex/40_변환과_렌더.md)을
실제 help/capabilities에 맞췄다. 두 터미널 출력 모두 글자가 없을 때 렌더링 호스트에 글꼴을 공급하고
`--font-path`로 폴더·파일을 지정하는 방법, SSH 원격 경로와 저장 파일 재생성 예시를 포함한다.

## 검증 결과

| 항목 | 결과 |
| --- | --- |
| fmt·기본/WASM32/workspace/all-target/Native Clippy·workspace build | 모두 PASS |
| release-test 전체 nextest | **11,361 PASS·0 FAIL·45 skip** |
| Native lib와 관련 unit target | **4,041 PASS·0 FAIL·12 ignored** |
| Native missing-picture / 직접 PDF | 2 PASS / 14 PASS |
| Native Sixel / Kitty CI 호출 | 7 PASS / 8 PASS |
| suite manifest 정책 | 1,685 sources·48/48 targets PASS |
| 에이전트 생성기/스킬 Python 계약 / Kitty 독립 검사기 | 14 PASS / 2 PASS(13개 스트림) |
| CLI 글꼴 예시 / 프로필·호환 옵션 | 4 PASS / 8 PASS, 잘못된 값 2건 exit 2 |
| 문서 링크 / 매뉴얼 메타데이터 / 공백 | 10개 / 5개 / PASS |
| 최종 Native CLI 빌드 | PASS |

- 4개 공개 문서 11쪽×360/720px의 22건에서 독립 복원 PNG와 같은 조건 흰 배경 PNG의 픽셀이 같았다.
  기존 기본 PNG 11쪽·축소 PNG 22건·Sixel 22건도 변경 전후 같았다.
- kitty 0.49.2에서 로컬/SSH의 직접 출력·저장 파일 재표시를 확인했다.
  표·한글·각주·crop·화살표·종횡비와 이미지 아래 프롬프트를 직접 판독했다.
- Windows 11 WezTerm 자체 SSH는 사용자가 그래픽과 한글 표시를 확인했다.
  원격 폰트 누락은 작업 폴더에 공개 글꼴과 라이선스를 공급하고 재출력해 해소했다.
  WSL 경유 실패를 직접 SSH의 판정으로 쓰지 않았다.
- 매뉴얼 예시대로 다른 작업 디렉터리에서 절대 글꼴 폴더·반복 글꼴 파일을 지정한 4건도 실행했다.
  프로필 4종×호환 옵션 2종의 8건은 같은 PNG와 일치했다.
- 보조 Native 비교의 4개 문서 전쪽 raw 최저는 표 96.64066%, 각주 98.95047%,
  crop 99.95079%, 복합 개체 100%다. 독립 Print와 쪽수가 같으며
  base 대비 11쪽의 하락은 0.00000%p, render tree도 동일했다.
  대표 review/overlay를 직접 판독했으며 기존 글꼴·획 농도 차이는 조판 개선으로 주장하지 않는다.

상세 명령·source/producer SHA·직접 관측과 문서 검사는
[1단계](../working/task_m100_8150_stage1.md), [2단계](../working/task_m100_8150_stage2.md),
[3단계](../working/task_m100_8150_stage3.md)에 연결했다.
새 실물 조판 fixture·golden·좌표 pin을 추가하거나 기존 기대값을 완화하지 않았다.

## 범위와 후속 절차

사용자의 범위 확인에 따라 Native CLI 전용 변경임을 다시 명확히 했다.
새 전송 모듈은 비 WASM·`native-skia`로 제한되며 공통 조판·paint·글꼴·기존 출력 producer를
바꾸지 않는다. WASM 재빌드·Studio 실행·WASM 시각 검증은 실제 소비 경로상 비해당이며,
Rust 공통 lint의 WASM32 Clippy는 통과했다.

지원 탐지·터미널 자동 설정·Sixel 자동 전환·tmux/screen 중계·TUI는 이번 기능에 포함하지 않는다.
종료 코드 0은 전송 성공이며 터미널 지원 여부는 실제 표시로 확인한다.
Sixel 대비 성능 향상이나 한컴과의 새로운 조판 일치를 주장하지 않는다.

승인된 로컬 단계와 문서화를 완료했다. remote push·PR 생성·GitHub comment·merge·이슈 close는
수행하지 않았으며 후속 원격 작업은 별도 승인 범위를 따른다.


## PR 제출과 self-review

2026-10-22 사용자 승인으로 [PR #8154](https://github.com/edwardkim/rhwp/pull/8154)를 devel 대상 Open 상태로 생성했다.
code head `d030087e240fe851edfe9ad486176bd1624dcf40`의 CI·CodeQL·Render Diff·Adapter·Proptest가 모두 실제 성공했다.
이후 사용자 지시로 [self-review](../pr/archives/pr_8154_review.md)를 수행해 승인 판정했다.
검토·오늘할일 기록은 같은 PR의 문서-only trailing으로 반영하며 제품·test·workflow는 변경하지 않는다.
최신 trailing head의 실제 CI/재사용 판정·merge tree·원격 SHA 확인과 사용자 merge 승인은 남은 조건이다.
merge·이슈 close는 수행하지 않았다.
