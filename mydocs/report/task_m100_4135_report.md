# Task M100 #4135 완료 보고서

- **Issue**: [#4135](https://github.com/edwardkim/rhwp/issues/4135)
- **기준**: `upstream/devel` `94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **완료일**: 2026-08-28 KST
- **원격 상태**: push/PR 생성 전

## 결과

F5 셀 블록 상태의 `Ctrl/Cmd+Shift+S`를 `table:block-sum`으로 보내는 좁은 문맥
라우터를 추가했다. 셀 블록 밖에서는 기존 `file:save-as`, 수정자 없는 `S`는
`table:cell-split`을 유지한다. 한글 자모 `ㄴ`과 조합 중 `Process/KeyS`도 같은 물리 키
계약으로 처리한다.

embed 프로파일에서는 숨겨진 `file:save-as`가 비활성일 때 후순위 표 명령으로 우회하지
않고 이벤트만 소비한다. 전역 단축키 맵에서는 도달 불가였던 `table:block-sum` 중복을
제거하고, Save As 항목에 물리 `KeyS` 보정을 추가했다.

## 최신 기준선에서 확인한 원인

이슈가 작성된 시점에는 동일 단축키의 첫 매칭 `file:save-as`가 블록 합계를 가렸다.
최신 devel에서는 그 중복과 함께, F5 셀 선택 분기의 modifier를 확인하지 않는 `S` 처리도
매처보다 먼저 실행됐다. 실제 브라우저 기준선은 이슈 본문의 Save As가 아니라
**셀 나누기 대화상자**였다.

따라서 전역 매처 계약 전체를 바꾸는 대신 다음 세 지점을 함께 보정했다.

1. IME와 셀 선택 `S` 분기보다 먼저 실행되는 셀 블록 전용 라우터
2. modifier 없는 `M`/`S`에만 셀 합치기/나누기를 허용하는 가드
3. 일반 Save As의 `KeyS` 보정과 전역 exact 슬롯 중복 제거

## 단계별 커밋

| 단계 | commit | 내용 |
| --- | --- | --- |
| RED·계획 | `a191509f9` | 재현 기록, full/embed·Ctrl/Meta·영문/IME 계약, 중복 슬롯 검사 |
| 구현·GREEN | `2141666e0` | 문맥 라우터, 입력 순서/가드, shortcut-map 정리, 브라우저 증빙 |
| 최종 기록 | 이 보고서 commit | 전체 검증과 PR 준비 상태 정리 |

## 검증 결과

```text
focused Node: 31 pass, 0 fail
npm test: 1,231 tests / 1,230 pass / 1 skip / 0 fail
npm run build: TypeScript + Vite production build, 239 modules transformed
npm run e2e:embed: 17 assertions PASS
cargo fmt --all: exit 0
cargo fmt --all -- --check: exit 0
git diff --check: exit 0
```

embed E2E의 첫 시도는 제품 코드가 아니라 `puppeteer-core`의 Chrome 실행 경로 미설정으로
시작 전에 실패했다. 설치된 Chrome의 `CHROME_PATH`와 실행 중인 개발 서버의 `VITE_URL`을
명시해 다시 실행했고 전 판정이 통과했다.

실제 macOS Codex in-app browser의 최신 런타임에서 2행 3열 표를 만들고 `F5`, `F5`,
`ArrowRight`로 복수 셀을 선택해 다음을 확인했다.

| 입력 | 결과 |
| --- | --- |
| 셀 블록 + `Ctrl+Shift+S` | 셀 나누기/Save As 대화상자 없이 블록 계산 경로 실행 |
| 셀 블록 + `Cmd+Shift+S` | 셀 나누기/Save As 대화상자 없이 블록 계산 경로 실행 |
| 셀 블록 해제 + `Ctrl+Shift+S` | 다른 이름으로 저장 대화상자 표시 |
| 셀 블록 + `S` | 셀 나누기 대화상자 표시 |

## 범위

이번 작업은 #4135의 단축키 도달성만 수정했다. 기존 `table:block-sum`이 현재 셀에
`=SUM(above)`를 적용하는 계산 의미는 바꾸지 않았다. 선택 범위 자체를 피연산자로 삼아야
한다는 별도 요구가 확인되면 연결 이슈에서 계산 명령을 확장하는 편이 안전하다.

## 작업 상태

로컬 구현, 단계별 커밋, 자동·실제 브라우저 검증과 PR 직전 포맷 게이트를 완료했다.
GitHub 원격 push와 PR 생성은 저장소 지침에 따라 작업지시자의 별도 승인 전까지 수행하지 않는다.
