# 이슈 #7642 — 복원 코드의 전쪽 대조와 최초 배치 차이 추적

시작 커밋: `5cb859903`. 생산 Rust 코드는 `e08eae718`과 동일하다.
이번 작업의 모든 회귀 실패는 현재 브랜치에서 보정하며 검사·원본을 이관하지 않는다.

## 검증 대상과 고정 근거

원본 `samples/3-11월_실전_통합_2022.hwpx`와 이미 커밋한
`pdf/issue7642-failed-regression-print/3-11월_실전_통합_2022-hwpx-print-2024.pdf`를
2024 조판으로 전쪽 대조한다. 15쪽만 95.75481%가 된 것을 전체 문서 통과로
보고하지 않는다. Native 바이너리는 Stage33의 SHA-256
`03507a186ea558abb586d207b5ef744339e2c688ee4be8dfdd3e61f9d16acc85`이다.
실행 전 `git diff e08eae718 -- src`가 비고 frozen build의 모든 기록 소스 해시가
현재 파일과 일치함을 확인했다. 실패한 Stage34 바이너리를 사용하지 않는다.

전쪽 TSV는 ignored `output/pr-review/issue7642/failed-inputs-20261008/`
`stage15-current-validation/endnote-stable-generation-native-all/`에 저장한다.
정확한 명령·소스 대응은 `endnote-stable-generation-native-all-provenance.json`,
실행 로그는 같은 접두어의 `.log`에 남긴다. timeout은 7200초로 지정했다.
MCP 재출력이나 PDF 복제 없이 원본에 대응하는 기존 Print를 사용한다.

## 다음 순서

1. 전쪽의 최저값·미달·누락·쪽수를 확인하고 최초 배치 차이부터 PDF/Native를 직접 확대 대조한다.
2. 현재 단의 실제 컷·원점과 측정 문맥을 대조한다. 입력 저장 정보와 현재 메트릭의 소비를
   확인한 뒤 구현을 수정한다. 물리 예산의 오류를 컷 변경이나 bleed로 숨기지 않는다.
3. 수정 영역과 정상 대조군을 다시 캡처한다. source가 변하면 해당 결과를 재검증한다.
4. 요청된 52개 원본의 최종 Native/fresh WASM 전쪽 및 미세 수식 차이를 확인한다.
5. 90% 이상 선행 조건 후 관계형 검사 교정, 전체 회귀·lint·정책·Studio 검증을 완료한다.

Stage31 fresh WASM은 이 생산 코드의 미주 보정을 포함하지 않는다. 이후 fresh WASM을
다시 빌드하며 이전 bundle을 최신 검증으로 보고하지 않는다. 새 PR·원격 push는 아직
실행하지 않았고 PR/CI/merge/후속 처리까지의 기존 목표를 유지한다.

## 실제 저장본 쌍에서 확인한 단 시작 정보

요청된 52개 원본 중 HWPX의 미주 저장 되감김 57건을 대응하는 HWP와 대조했다.
같은 미주 번호·문단 수에 더해 해당 문단의 텍스트, 수식 스크립트, 줄 수가 모두
일치함을 확인했다. 50건은 기존 0/음수 되감김이다. 양수 되감김 7건은 다음 관계를
모두 만족한다.

- 앞 줄에서 직전 줄까지는 저장 높이+간격만큼 연속으로 전진한다.
- 다음 줄의 양수 vpos가 직전 줄의 높이+간격과 같지만 직전 vpos보다 작다.
- 대응하는 HWP는 직전 줄 vpos를 0으로 저장하고 다음 줄 좌표를 그대로 유지한다.

이는 다음 줄에서 처음 새 단이 시작하는 것이 아니라 직전 줄이 이미 새 단의
첫 줄임을 나타낸다. 두 서로 다른 저장 높이·간격 조합이 관측됐다. 독립 2024 Save도
같은 원점을 기록하며 원본 2024 Print의 줄 이월과 일치한다. 문서명·문항 번호·실물
좌표·픽셀 허용치를 구현 조건으로 쓰지 않는다. 진단은 ignored
`implicit-endnote-column-continuity-candidates.tsv`, `all-endnote-rewind-candidates.tsv`에
남긴다. 이 TSV는 페이지별 Visual Sweep TSV를 대체하지 않는다.

저장 HWPX 미주를 재사용하는 다단 경로에서만 이 단 시작을 로컬 사본에 복원한다.
원문 Document/IR과 저장 파일은 변경하지 않는다. `row_projection::project_equation_rows`
이전에 복원해 줄 메트릭 투영·측정·실제 배치가 같은 0 원점을 소비하게 한다.
기존 명시적 0/음수 컷과 다른 되감김, 편집 후 재조판은 이 보정으로 바꾸지 않는다.
원본 HWP/2020과 HWPX/2024 15·16·17쪽, 서로 다른 간격을 가진 HWPX 대조를
실행한다. 이전 frozen 전쪽 작업은 변경 전 증거로 계속 보존한다.

## 실행 결과와 남은 차이

Native build와 Native Clippy, `cargo fmt --all -- --check`는 exit 0이다.
빌드 소스 해시와 frozen 바이너리는 ignored `endnote-column-start-native-provenance.json`에
고정했다. 바이너리 SHA-256은 `f59c0b8f077aedbb5fb50466bef4570c6dc04f6bd9afa4181f31cdc585dec760`이다.

| 원본·기준 | 페이지 | 보정 전 → 후 2px 실루엣 일치율 |
| --- | --- | --- |
| 2022 HWPX·2024 Print | 15 / 16 / 17 | 95.75481 / 95.62674 / 44.09679 → 95.75481 / 96.33499 / 97.23218% |
| 2022 HWP·2020 Print 정상 대조 | 15–17 | 보정 조건 비적용, 최저 95.73324% |
| 구분선 위 20·미주 사이 0·구분선 아래 20 HWPX·2024 Print | 14 / 15 | 64.52423 / 75.79976 → 93.10374 / 75.79976% |

전후 모두 원본 전체 쪽수는 기준과 같은 21쪽이며 위 표는 선택 쪽의 결과다.
zero-gap 대조의 미달 15쪽은 이번 단 시작 보정으로 새로 발생한 회귀가 아니다.
같은 frozen 변경 전 바이너리·독립 Print·세대·페이지를 다시 실행해 같은 75.79976%를
확인했다. 실행은 `endnote-column-start-focused-native/results.json`과
`endnote-column-start-zero-gap-before/summary.json`에 보존한다.
17쪽과 zero-gap 15쪽의 실제 review PNG를 열어 사용자에게 표시했다.
17쪽 첫 적분 줄의 단 소속과 그림 배치는 개선됐으나 수식 모양의 미세 차이는 남아 있다.

변경 전 21쪽 전수 결과의 미달은 13·14·17·18·19·20쪽이다.
13쪽의 (iii) 그림 설명은 앞 단 하단에 들어갈 공간이 있는데 다음 단으로 이동한다.
직전 미주 끝에서 예약한 간격을 다음 미주의 저장 vpos 전진량에서 다시 소비하는
경로를 확인했으며 다음 단계에서 간격의 소유를 보정한다.
현재 보정의 전쪽·fresh WASM·전체 회귀는 미완료이므로 문서 통과나 PR 준비 완료로
판정하지 않는다. 기존 회귀 기대값·baseline은 변경하지 않았다.
