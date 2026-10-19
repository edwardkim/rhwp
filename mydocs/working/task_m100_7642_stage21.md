# 이슈 #7642 — DYAD 실제 표시와 장식 소유의 관계 검사

이전 단계: [DYAD 표시 원점 보정](task_m100_7642_stage20.md), source `603a41837`.

## 범위와 독립 근거

현재 작업의 전체 회귀 실패는 현재 브랜치에서 보정한다. 고정 upstream base에서 이미
미달했던 사례도 새 제거·이관 없이 실제 출력부터 보정한다. 전체 52개 원본과 90개
실패의 해결 여부는 개별 선행 검증과 최종 전체 검증으로 판단한다.

#7593 원본·최소의 독립 MCP2020 Print는 동일 BaseUnit의 AB 장식 폭이 같고, 큰 저장
프레임의 보기 ㄴ은 뒤 본문보다 위에 표시된다. 기존 검사는 실제 paint가 소비하는
LayoutBox 루트 y를 빠뜨렸으며, 장식 안의 AB가 한 text 요소라고 가정했다.

소비 경로는 저장 프레임의 줄 원점 → `layout_for_paint`의 루트 y → SVG/Canvas/Skia의
실제 glyph 원점이다. 검사도 이 실제 표시와 같은 장식 안의 A/B 순서를 확인한다.
같은 크기의 글자를 담은 저장 프레임 폭이 다르더라도 장식의 실제 폭 비율은 같아야 한다.

## 선행 검증과 계획

source `603a41837`의 Native 원본 98.56380%, 최소 99.19457%이며, math8 94.14918%,
eq002 HWP/HWPX 94.81146%, 국어 1p 98.90252%다. 누락은 없다. 원본의 확대 review에서
보기 ㄴ의 실제 표시 원점이 독립 Print와 같은 관계임을 확인했다.

같은 source의 fresh WASM 빌드는 exit 0으로 완료됐고, 루트 pkg와 Studio JS/WASM 해시가
일치한다. 6개 원본의 fresh WASM 전쪽 TSV와 같은 배율 확대 판독을 먼저 완료한 뒤
`issue_7593_print_placement.rs`의 두 검사를 교정한다.

- 실제 기준선은 루트 paint y를 포함해 뒤 본문과의 위아래 관계로 확인한다.
- 같은 DYAD 장식 안의 실제 A/B 내용·순서·정체와 모든 조상 transform을 소비한 폭 비율을 확인한다.
- 절대 px 좌표·영역, SVG 해시, 허용치 완화로 출력 차이를 숨기지 않는다.
- 수정 전 실패와 수정 후 통과를 연결하고 기존 소유·탭·내용 보존 검사는 유지한다.

아직 fresh WASM 대조·검사 교정·focused 회귀 결과는 미완료다. 전수 검증 또는 PR 준비 완료로
판정하지 않으며, 로그·TSV·JSON·nextrun은 ignored output에 보존한다.

## fresh WASM 선행 확인과 음성 대조

source `603a418370c4666f24ec7e48c4e5c2486f2c062f`의 fresh WASM 6개 원본 전쪽 대조가
모두 exit 0으로 완료됐다. 최저값은 Native와 동일하며 누락은 없다. `dyad-origin-wasm/results.json`에
입력·기준 Print PDF·source SHA·명령을 고정했다. 실제 브라우저에서 출력한 WASM을 사용했고,
WASM SHA-256은 `cdd1fffb385c169e1d24c1df9084750f9107d06e9c98e3b1be9fff28a0073bf1`이다.

`representative-wasm-dyad-origin/`의 6개 review를 만들었다. 원본 DYAD와 math8의 같은 배율
확대 PNG를 직접 열고 사용자에게 표시했다. 보기 ㄴ의 AB는 뒤 본문보다 위에 표시되며,
math8 CASES의 같은 칸·첨자·괄호 소속을 확인했다. glyph와 화살표 끝의 미세 모양 차이는
점수와 별도로 남기며, 이 관측으로 전체 52개 또는 다른 미달 페이지를 통과 처리하지 않는다.

기존 기준선 검사를 실제 생산 SVG glyph 원점을 소비하도록 교정했고, 같은 장식의 AB 내용과
A/B 순서·정체 및 모든 조상 transform을 소비한 장식 폭 비율을 검사한다. frame의 저장 앵커,
셀 소유와 뒤 본문 보존 검사는 유지한다. 실물 px 좌표·영역을 새 기대값으로 고정하지 않았다.

동일 원본의 변경 전 `83f790957` 생산 SVG와 변경 후 `603a41837` 생산 SVG에 같은 위아래
관계 진단을 실행했다. 전에는 AB와 뒤 본문 기준선이 같아 FAIL, 후에는 AB가 위라 PASS다.
`dyad-origin-negative-control.json`은 이를 source·SVG 해시와 연결한다. 이것은 생산 출력에 대한
음성 대조이며, 변경 전 코드에서 새 Rust 검사를 실행한 결과로 바꾸어 기록하지 않는다.
focused Rust 검사와 변경 전 코드의 직접 실행은 아직 미완료다.

## 집중 Rust 검사

`node scripts/run-rust-test.mjs issue_7593_print_placement -- --cargo-profile release-test
--target-dir /home/tsjang/rhwp/target/pr-review --no-fail-fast`는 컴파일 7분 17초 뒤 exit 0으로
완료됐다. 6개 검사 모두 통과했고 같은 suite의 비선택 검사 217개는 실행하지 않았다.
새 실제 glyph 기준선 검사, AB 장식 폭 관계, 기존 바탕쪽 흐름·셀 소유·탭 격자·수직/평행
관계 기호의 앞뒤 공간 검사를 포함한다. 이 집중 결과를 전체 회귀 통과로 보고하지 않는다.

변경 전 `1017fefff`의 직접 Rust 음성 대조를 위한 독립 worktree를 준비 중이다. 같은 공용
`target/pr-review`에서 Cargo를 순차 실행하고, 현재 작업 트리의 source를 되돌리지 않는다.
그 결과와 최종 전체 lint·회귀는 아직 미완료다. 현재의 원격 PR·CI·merge는 없다.

## 변경 전 코드의 직접 실패 확인

별도 worktree의 source `1017fefffec8272346ad6aa0986ad05e1349c194`에 동일한 교정 검사만
복사하고 같은 공용 target에서 위 집중 명령을 순차 실행했다. 제품 source는 변경하지 않았다.
컴파일 8분 42초 뒤 exit 100으로 끝났으며, 실제 보기 ㄴ 기준선 관계에서 **1개 FAIL / 정상
대조 5개 PASS**였다. 빌드·환경 실패가 아니다. `stage21-before-provenance.json`과 로그는
source·검사 해시·명령·실패 지점을 고정한다. 보정 source에서는 같은 검사 6개가 모두 통과한다.

실행 종료 뒤 임시 worktree에 다른 변경이 없는지 확인하고 이번 검사 복사만 복원했다.
clean 상태를 확인해 작업 전용 worktree를 제거했으며 공유 target은 보존했다.
전체 회귀와 최종 head의 lint·전수 시각 검증은 여전히 진행 중이다.
