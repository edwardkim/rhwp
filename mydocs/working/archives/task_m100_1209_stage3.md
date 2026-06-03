# Task M100 #1209 Stage 3

## 목적

`3-09월_교육_통합_2024-구분선아래20.hwp`와 `3-09월_교육_통합_2024-미주사이20.hwp`가 각각의 한컴오피스 2024 PDF 기준과 맞는지 확인한다. Stage2에서 정리한 compact 미주 공통 gap 보존 로직이 기본 7mm 파일뿐 아니라 `구분선 아래 20mm`, `미주 사이 20mm` 설정 파일에도 문항별 예외 없이 적용 가능한지 판단한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 선행 커밋: `90eb8385` (`task 1209: Stage2 미주 제목 공통 간격 보정`, `upstream/devel` rebase 후)
- 대상 HWP:
  - `samples/3-09월_교육_통합_2024-구분선아래20.hwp`
  - `samples/3-09월_교육_통합_2024-미주사이20.hwp`
- 대상 PDF:
  - `pdf/3-09월_교육_통합_2024-구분선아래20-2024.pdf`
  - `pdf/3-09월_교육_통합_2024-미주사이20-2024.pdf`

## 확인 질문

1. 두 파일의 `FootnoteShape.raw_unknown`/구분선 아래 값이 실제 한컴 UI의 `미주 사이 20mm`, `구분선 아래 20mm`와 대응되는가?
2. 페이지 수와 주요 미주 시작 위치가 PDF 기준과 맞는가?
3. Stage2의 공통 gap 보존 로직이 20mm 설정을 중복 적용하거나 누락하지 않는가?
4. 현재 구조에서 `미주 사이`와 `구분선 아래`를 같은 공통 정책으로 다룰 수 있는지, 아니면 서로 다른 레이어에서 처리해야 하는지 판단한다.

## 진행 계획

1. 두 HWP의 미주 모양 레코드와 페이지 수를 dump한다.
2. 대응 PDF 페이지 수를 확인한다.
3. RHWP SVG를 내보내고 `rsvg-convert`로 PNG를 생성한다.
4. PDF는 `pdftoppm`으로 같은 페이지 PNG를 생성한다.
5. 주요 미주 제목/구분선 위치를 비교하고, 필요 시 테스트를 추가한다.
6. 구현 변경이 필요하면 별도 승인 후 진행한다.

## 현재 상태

- 2026-06-01: 작업지시자가 Stage2 커밋 후 2024 변형 파일 2종을 새 스테이지에서 비교하라고 지시했다.
- 2026-06-01: Stage2 커밋 이후 Stage3 문서를 생성했다. 이후 작업지시자가 `upstream/devel` 동기화와 rebase를 지시해 `upstream/devel` `73034de9` 기준으로 rebase했고, Stage2 커밋 해시는 `90eb8385`로 갱신됐다.
- 2026-06-01: `pdfinfo` 기준 PDF 페이지 수는 `구분선아래20-2024.pdf` 23쪽, `미주사이20-2024.pdf` 24쪽이다. RHWP `dump-pages` 기준도 각각 23쪽, 24쪽이라 페이지 수만 보면 기존 회귀 테스트와 일치한다.
- 2026-06-01: 기존 `issue_1139_endnote_spacing_reference_files_match_hancom_page_counts` 테스트가 두 파일의 미주 모양 값을 이미 검증하고 있음을 확인했다.
  - `구분선아래20.hwp`: `note_spacing≈20mm`, `raw_unknown≈7mm`
  - `미주사이20.hwp`: `note_spacing≈2mm`, `raw_unknown≈20mm`
- 2026-06-01: 첫 미주 페이지인 9쪽을 `rsvg-convert`/`pdftoppm`으로 비교했다. `구분선아래20`은 구분선 아래 첫 미주 제목까지 RHWP 약 `78px`, PDF 약 `76px`이고, `미주사이20`은 RHWP 약 `12px`, PDF 약 `9px`라서 구분선 아래 설정 자체는 대체로 렌더에 반영된다.
- 2026-06-01: 그러나 `구분선아래20` 9쪽 export에서 `LAYOUT_OVERFLOW`가 다수 발생했다. 렌더는 구분선 아래 20mm를 실제 y 이동으로 소비하지만, pagination은 해당 시각 높이를 충분히 예약하지 않아 같은 쪽/단에 너무 많은 미주 paragraph를 배치하는 것으로 보인다.
- 2026-06-01: `구분선아래20` 후반부는 PDF와 크게 다르다. PDF 22쪽은 오른쪽 단에서 `문30)`이 시작하지만 RHWP 22쪽은 `문29)`가 왼쪽 단 하단에 남고, RHWP 23쪽은 `문30)` 시작부터 크게 남는다. PDF 23쪽은 `문30)` 후반 왼쪽 단 일부만 남는다.
- 2026-06-01: `미주사이20`도 정상 처리로 판정하기 어렵다. PDF 23쪽은 `문29)`가 페이지 첫머리에서 시작하지만 RHWP 23쪽은 이전 풀이 꼬리가 위쪽에 더 남고, RHWP 24쪽은 PDF 24쪽보다 내용이 앞에서 시작하며 일부 텍스트가 그림 아래쪽에서 겹쳐 보인다.
- 2026-06-01: 구현 보정 방향은 문항별 예외가 아니라 미주 모양 공통 정책으로 정리했다.
  - `구분선 아래`: separator가 실제 렌더에서 소비하는 높이를 compact 여부와 분리해 pagination budget에도 반영한다.
  - `미주 사이`: HWP5 line-seg vpos에 기본 7mm 흐름이 이미 포함되어 있으므로, 7mm 초과분 중 pagination이 별도로 예약할 몫을 공통 helper에서 계산한다.
- 2026-06-01: 보정 후 `구분선아래20`은 23쪽 유지, 22쪽 오른쪽 단에서 `문30)`이 시작하고 23쪽에는 `문30)` 후반 꼬리만 남는다.
- 2026-06-01: 보정 후 `미주사이20`은 24쪽 유지, 23쪽에 `문29)`와 `문30)` 시작이 함께 배치되고 24쪽에는 `문30)` 후반 꼬리만 남는다. 다만 PDF 23쪽은 `문29)`가 페이지 첫머리에서 바로 시작하는 반면 RHWP는 `문28)` 꼬리 일부가 상단에 남아 시각 정합은 아직 완전하지 않다.
- 2026-06-01: `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(37 passed, 0 failed). 전체 `cargo test --tests`는 PR 전 검증으로 남긴다.

## 산출물

- `output/task1209_stage3_2024/below20/rhwp_page09.png`
- `output/task1209_stage3_2024/below20/pdf_page09.png`
- `output/task1209_stage3_2024/below20/rhwp_page22.png`
- `output/task1209_stage3_2024/below20/pdf_page22.png`
- `output/task1209_stage3_2024/below20/rhwp_page23.png`
- `output/task1209_stage3_2024/below20/pdf_page23.png`
- `output/task1209_stage3_2024/between20/rhwp_page09.png`
- `output/task1209_stage3_2024/between20/pdf_page09.png`
- `output/task1209_stage3_2024/between20/rhwp_page23.png`
- `output/task1209_stage3_2024/between20/pdf_page23.png`
- `output/task1209_stage3_2024/between20/rhwp_page24.png`
- `output/task1209_stage3_2024/between20/pdf_page24.png`
- `output/task1209_stage3_2024/between20/rhwp_page22.png`
- `output/task1209_stage3_2024/between20/pdf_page22_tmp-22.png`

## 판단

- 두 20mm 변형 파일은 문항별 예외가 아니라 공통 미주 모양 정책으로 일부 보정 가능하다.
- `구분선 아래`와 `미주 사이`는 같은 UI 대화상자에 있지만 저장 슬롯과 조판 레이어가 다르므로, 하나의 값으로 합치지 않고 각각 공통 helper로 처리해야 한다.
- `구분선 아래`는 separator 자체가 차지하는 시각 높이를 pagination에도 예약해야 한다.
- `미주 사이`는 full raw 값을 그대로 pagination에 더하면 렌더 `line_spacing`과 중복되고, 전혀 더하지 않으면 20mm 변형의 24쪽 분기가 사라진다. 따라서 기본 7mm를 제외한 초과분의 pagination 예약 몫만 공통 helper로 계산한다.
- 현재 보정은 page count와 후반 `문30)` 분배 회귀를 잡지만, `미주사이20` 23쪽 상단의 `문29)` 시작 위치는 PDF보다 아래라 추가 시각 정합 여지가 남는다.
