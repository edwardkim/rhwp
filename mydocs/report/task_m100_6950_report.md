# #6950 결과 보고서 — 문단 끝 자리차지 표의 배치와 후속 흐름

- Issue: [#6950](https://github.com/edwardkim/rhwp/issues/6950)
- 작성일: 2026-09-10
- 브랜치: `task_m100_6950`
- 상태: 필수 로컬 검증 완료, [Open PR #6994](https://github.com/edwardkim/rhwp/pull/6994) 제출. 원격 CI 확인 대기.
- 상세 이력: [Stage 1](../working/task_m100_6950_stage1.md),
  [Stage 2](../working/task_m100_6950_stage2.md), [Stage 3](../working/task_m100_6950_stage3.md)

## 1. 해결 범위

문단의 마지막 텍스트 뒤 자리차지 표가 남은 너비에 들어가지 않을 때 다음 줄에 배치하고,
표·셀·테두리를 같은 원점으로 이동시킨다. 문단 종료는 확정된 표와 텍스트의 점유 범위를
소비하고, 후속 빈 문단과 다음 본문은 자신의 줄간격을 반영한다.

단, 뒤 문단의 텍스트가 위쪽 빈 공간을 사용하다가 떠 있는 표에서만 회피해야 하는
기존 배치는 유지한다. 표의 세로 위치가 크다는 이유만으로 문단 종료 위치를 내려잡지 않는다.
실제 줄의 잔여 너비와 표·바깥 여백 폭으로 `Exclusion`(배제 영역)과
`NextLine`(줄 너비 부족으로 다음 줄을 점유)을 구분한다.

일반 배치·fit·pagination·조각 출력에 `ParagraphFloatPlacement`를 전달해 원점을 중복
보정하지 않는다. 저장 LineSeg의 유효성과 실제 재조판 결과를 사용하며 샘플명·표 개수·
특정 세로 오프셋 임계값으로 이번 사례를 하드코딩하지 않았다.

추가로 표 속성 조회의 크기·위치·여백을 저장용 raw 바이트가 아닌 공통 IR에서 읽도록
보완했다. HWPX의 raw 부재 때문에 0으로 표시되던 값을 수정한 조회 변경이며,
문서 mutation·Undo/Redo 동작을 새로 추가하지 않았다.

## 2. 원인 계보와 경계

최초 가설은 역사 A/B로 확인했다. 기존 미해결 문제가 앵커 줄 보정 확장으로 해소됐다가,
#6718 등 다른 문서 회귀를 보호하려던 `980c80203`의 적용 제한으로 재발했다.
제한을 무조건 삭제하면 보호 문서가 악화되어, 공통 배치 결과를 전달하는 방식으로 수정했다.
표 속성 조회의 raw 의존은 이번 작업 전 기준선에도 존재한 별도의 기존 문제였다.

작은 용지 높이의 합성 표 분할 실험은 여러 쪽 지원 속성을 충분히 고려하지 않은
기대값이었으므로 메인테이너 결정에 따라 실패 판정을 철회하고 범위에서 제외했다.
이를 기존 실문서의 회귀를 제외하거나 기준선을 올리는 근거로 사용하지 않았다.

## 3. 샘플·시각 판정

- 원본: `samples/hwpx/20260909-para-table.hwpx`
- 한컴 기준: `pdf/hwpx/20260909-para-table-2024.pdf`
- 최종 원본은3쪽이며 문단 끝 표·후속 pi=1/pi=3 흐름에 메인테이너 시각 승인을 받았다.
- `samples/synam-001.hwp`30쪽: 명시적 개행 보존과 문단 위3.70mm 배치 보정을 SVG·WASM으로 승인받았다.
- `samples/issue6025/3232693_employment_support_criteria.hwpx`1쪽: 속성 바인딩과
  첫 표 조각 원점 보정을 SVG·WASM으로 승인받았다.
- `samples/issue1510_coanchored_float_tables.hwp`1쪽 / 같은 stem의 HWPX2쪽:
  표 위 본문과 표 아래 재개 흐름에 시각 승인을 받았다. 한컴에서도 HWP1쪽·HWPX2쪽이므로
  형식별 기존 쪽수 래칫을 유지했다. 두 형식의 쪽수를 강제로 같게 하지 않았다.

최신 devel 통합 뒤 원본3쪽과 #1510 HWP1쪽·HWPX2쪽의 SVG6개를 동일 `--font-style`
옵션으로 재생성해 병합 전 승인본과 **바이트 동일**함을 확인했다.
이는 해당 페이지의 보존 증거이며 전체 코퍼스의 시각 동등성을 주장하지 않는다.

## 4. 검증과 통합

병합 전 승인 후보 `716624893`의 전체 nextest는9,414통과·0실패·46skip이었다.
Canvas3/3·Direct PDF3/3·CanvasKit readiness8/8의 CI 조건 로컬 검증도 통과했다.
상세 수치·경고·환경·한계는 Stage 3 §29~30에 기록했다.

PR 제출 전에 최신 `devel` `2a780e0d296846df577866eba6ac8f388527551b`를
`f2a8f3ac5b13e80932300b08e3a67613a199697a`로 병합했다.
#6985 글상자 배제와 #6643 래퍼 여백 보정을 보존했고, 새 #6972 테스트의
추가 필드 초기화 누락을 `a653d23dd`에서 보완했다. 기존 기대값은 변경하지 않았다.
통합 후보의 필수 lint·전체 회귀·Native Skia·WASM 및 Render Diff를 다시 실행했다.
상세 명령·로그·해시는 Stage 3 §31에 기록했다.

| 검증 | 통합 후보 결과 |
| --- | --- |
| fmt·native/WASM/workspace Clippy·workspace build | PASS |
| integration manifest·source-side unit tier | PASS |
| 전체 nextest | 9,444 passed / 0 failed / 46 skipped |
| Native Skia lib | 4,112 passed / 0 failed / 13 ignored |
| Native Skia placeholder / direct PDF 회귀 | 2/2, 4/4 PASS |
| Docker dev WASM·Native Skia CLI | PASS |
| CI 렌더 계약·Canvas / Direct PDF | PASS, 3/3 / 3/3 |
| CanvasKit readiness | 8/8 PASS |

검증 도중 추가된 #6991의 최신 devel `ec822767ae`도 `61eb331b0`으로 충돌 없이 반영했다.
이 추가 병합의 Rust·Studio·샘플·렌더 검사 입력 차이는0이다. CI delta 검사도
Python56/56·Node291/291을 통과했다. 리뷰 워크트리와 제출할 제품 파일의 동일성을 확인했다.

Canvas 최대 차이는0.01761%(허용0.05%), Direct PDF는1.15895%(허용2%)다.
보고용 Browser Canvas/PDF의4건 warn은 남아 있으며 gate 실패와 구분한다.
readiness의 초기 폰트·context·커서 경고도 로그에 보존했다. 최종 readiness는8건 모두
통과했지만 초기 오류가 전혀 없었다고 주장하지 않는다. nextest 버전·설정 경고도 기록했다.

## 5. 남은 조건과 한계

- 검증 완료 후보 `e4f2b1a38`을 push해 PR #6994를 생성했다. archive 제출 검증 기록·오늘할일·대표 PNG를 같은 PR에 포함한다.
- self-review 확정·병합·이슈 close는 최신 GitHub CI 확인 및 메인테이너의 후속 승인 절차다.
- 시각 지표는 폰트·래스터 차이를 포함하며 전체 fidelity의 합격 점수가 아니다.
- 성능은 공개 회귀 검사로 확인하며 동일 환경 전후의 전면적 성능 비교는 별도 측정하지 않았다.
- parser·serializer의 모든 표 정책이나 #6929·#6946·#6870의 해결까지 주장하지 않는다.
- generated suite·manifest, 임시 실행 로그·SVG·JSON, WASM 빌드 산출물은 PR에 포함하지 않는다.
