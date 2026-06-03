# Task #1245 Stage3: 2023 4쪽 `문26)` 중복 표시 보정

## 배경

- 2026-06-02: Stage2 커밋 `2a392639` 이후 새 스테이지를 시작했다.
- 작업지시자가 `samples/3-09월_교육_통합_2023.hwp` 4쪽에서 `문26)`이 두 번 보인다고 보고했다.
- 비교 기준 PDF는 `pdf/3-09월_교육_통합_2023.pdf`이다.

## 증상

- 대상 문서: `samples/3-09월_교육_통합_2023.hwp`
- 대상 페이지: 4쪽, 렌더러 페이지 인덱스 `3`
- 관찰 증상:
  - 왼쪽 단의 `문26)` 제목이 정상 위치에 보인다.
  - 같은 영역에 작은 `문26)` 표기가 한 번 더 나타나 본문 첫 줄 부근과 겹쳐 보인다.

## 초기 분석 계획

1. 4쪽 SVG/PDF 비교 산출물을 만든다.
2. SVG의 `문26)` 텍스트 노드 개수와 좌표를 확인한다.
3. 렌더 트리에서 중복이 텍스트 런 복제인지, 미주 제목/본문 조합인지, 또는 inline object/TAC 처리의 중복 배치인지 구분한다.
4. 원인 범위를 좁힌 뒤 `tests/issue_1139_inline_picture_duplicate.rs`에 회귀 테스트를 추가한다.

## 원인

- `pi=258` 문단 원문에는 `문26)`이 포함되어 있지 않고, 첫 control이 미주(`Endnote`)이다.
- `layout_composed_paragraph`는 미주 선두 번호를 본문 크기의 일반 `TextRun`으로 먼저 렌더링한다.
- 이후 같은 `Endnote` control의 `FootnoteMarker`가 같은 줄에 위첨자로 한 번 더 생성되어 조판부호 표시 상태에서 `문26)`이 중복 표시되었다.

## 수정

- 선두 미주 번호가 prefix `TextRun`으로 이미 렌더된 경우, 같은 위치의 `FootnoteMarker` 생성은 건너뛰도록 했다.
- 각주용 `FootnoteMarker`는 유지하고, 미주 선두 번호 중복만 제외한다.
- `3-09월_교육_통합_2023.hwp` 4쪽 렌더 트리에서 `문26`이 한 번만 나타나는 회귀 테스트를 추가했다.

## 산출물

- `output/task1245_stage3_after/svg/3-09월_교육_통합_2023_004.svg`
- `output/task1245_stage3_after/png/page4.png`

## 검증 대기

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과, 43개 테스트.
- `cargo fmt --all -- --check`: 통과.
- `rsvg-convert output/task1245_stage3_after/svg/3-09월_교육_통합_2023_004.svg -o output/task1245_stage3_after/png/page4.png`: 통과.
