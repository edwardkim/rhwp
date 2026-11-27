# #7777부터 오래된 본인 이슈 10건 처리 계획

## 요청과 고정 범위

2026-11-26 작업지시자는 `edwardkim/rhwp`의 본인 작성 열린 이슈에서 #7445를 제외한
오래된 10건을 처리하고, 이미 완료된 항목은 종료하도록 요청했다. `gh api user`의
현재 계정은 `jangster77`이다. 생성 시각 오름차순 조회로 아래 목록을 고정했다.
처리 중 이슈가 닫혀도 새로운 11번째 항목으로 바꾸지 않는다. #7445는 종료 대상이 아니다.

- 기준 SHA: `32014de6e0ade1a3baea8852c02270e40b27b345` (`upstream/devel`).
- 작업 브랜치: `task/7777-oldest-ten-issues`.
- 이슈 본문과 후속 댓글, 원 해결 PR, 통합·보정 커밋 및 독립 검증을 순서대로 조사한다.
- 승인 범위는 이슈 처리와 완료 이슈 종료다. 원격 push·PR·merge는 필요한 구현과 검증을
  완료하고 구체적인 후보를 준비한 뒤 세션의 승인 범위에 따라 처리한다.

| 이슈 | 대상 | 종료에 필요한 증거 |
| --- | --- | --- |
| #7777 | 37787 규제영향분석서 | 정확한 원본·독립 Print 전49쪽의 쪽 소속·표 이어받기·내용 보존과 두 backend 전쪽 대조 |
| #7778 | 59043 규제영향분석서 | 전37쪽의 표·제목·그림 소속, 36→37쪽 순서 및 실제 사진 겹침 판독 |
| #7779 | 76076의 `.hwpx` 확장자 OLE 입력 | 정확한 SHA의 원본·2020 Print 전82쪽, 6→7쪽 행 이월·쪽번호 비겹침 |
| #7780 | KTX-003 | 전1쪽의 겹침, 노선도·소요시간·운임안내 제목의 획·그림자 차이 해결 |
| #7781 | 반복 표 2439 | 전10쪽의 표·글자 소유, 원본 용지·인쇄 설정과 독립 Print 전체 대조 |
| #7782 | 저장 표 빈 문단 테두리 | 이관 철회 뒤 실제 보정 포함 여부, 빈 문단 물리 테두리·닫힌 프레임 회귀와 해당 독립 Print |
| #7784 | 76076 원본 HWP | 별도 입력 SHA의 전82쪽 및 33→34쪽 중첩 표 조각 관계 |
| #7785 | 수식 HML | 원본 HML·파생 한컴 저장본의 서로 다른 Print 기준, 본문 크기·수식 소속·첨자 확대 판독과 보류 검사 복원 |
| #7786 | KTX 목차 | 보존 경로 `mydocs/pr/assets/issue7445/ktx-toc27/KTX.hwp`의 전27쪽, 목차2쪽 내용·번호·외곽선 관계 |
| #7787 | overlay-top | 전1쪽 중첩 표 후속 세로 전진, Center/Bottom 등 기존 정상 대조군 보존 |

## 실행 순서

1. 원본·PDF·보류 검사·기존 증거의 경로와 SHA 및 기준 PDF의 실제 Creator/Producer를
   확인한다. 오래된 수치와 현재 재측정을 구분한다.
2. 현재 source에서 Native Skia 및 최적화 fresh WASM을 순차 빌드하고 producer를
   `output/pr-review/issues7777-7787/base-tools/`에 고정한다. 공용 `target/pr-review`를
   사용하는 다른 실행이 끝나기 전에는 Cargo를 시작하지 않는다.
3. 정확한 입력·같은 글꼴 공급·조판 세대·96 DPI·PDFium 조건으로 전체 페이지 TSV를
   양쪽에서 산출한다. 점수·쪽수와 직접 판독으로 완료·미완료를 구분한다.
4. 완료된 항목은 해결 커밋의 devel 포함과 독립 증거를 확인해 종료한다. 미완료는 원본
   저장값·사양과 실제 소비 경로를 추적해 공통 원인을 수정한다. 검사 이관이나 기준 완화로
   이 작업을 완료 처리하지 않는다.
5. 구현 변경의 경계·정상 대조군을 먼저 검증하고, 해당 최종 head의 전체 회귀·lint·fresh
   WASM·편람 및 정본 보호 게이트를 수행한다. 필요한 검사 복원은 시각 선행 조건을 따른다.

## 착수 관측

- 작업 트리는 깨끗했고 현재 devel과 upstream/devel이 일치했다.
- 본인 열린 PR 조회는 0건이었다.
- #7780의 PR #8191 후속 댓글은 제목 획·그림자 잔여를 명시한다. 99%대 점수만으로
  완료로 종료하지 않는다.
- #7778의 PR #9298 검토는 EXIF 회전 수정과 전체 조판 잔여를 구분한다.
- 별도 worktree `review-integration-9042`의 Cargo가 같은 target에서 실행 중이므로
  빌드를 겹치지 않고 입력·계보 조사를 먼저 진행한다. 다른 작업과 산출물은 보존한다.
- KTX 목차의 이전 `samples/KTX.hwp`는 위 자산 경로로 보존됐다. Git의 이전 원본과
  SHA-256 `b6c1492152f53e8dd7d4bbbb4faca88866bb8458e9018c70c936cd469ea6fab3`이
  같다. `samples/basic/KTX.hwp`는 별도 1쪽 입력이므로 사용하지 않는다.
- #7781의 기존 PDF는 파일명과 기존 보고서에 한컴 Print라고 적혀 있지만 실제 Producer는
  `Microsoft: Print To PDF`다. 기존 100% TSV는 공식 완료 근거로 쓰지 않고 정확한 원본을
  한컴 2024 MCP로 새로 인쇄한다.
- #7778의 이전 Hwp 2022 PDF는 참고로 보존하고, 같은 원본의 기존 2024 Print
  `pdf/semanticist21-20261123/59043_regulatory_analysis-2024.pdf`와 그 영수증을 사용한다.
- 시스템의 Python 갱신으로 기존 `venv/bin/python3.12`가 끊어져 있었다. 사용자 영역에
  CPython 3.12.16을 설치하고 `uv venv --allow-existing`으로 기존 패키지를 보존해 복구했다.
  PDFium 153.0.7999.0 / pypdfium2 5.13.0 및 `pdfium_smooth_rgb_exact_dpi_v2`를 확인했다.

## 완료 판정

10개 항목 각각에 실제 보정·양 경로 전쪽 수치·독립 Print 쪽수·확대 판독·회귀 및 통합
상태의 근거가 있어야 한다. 일부 항목의 종료나 조사표 작성은 전체 목표의 완료가 아니다.
미해결 항목은 원래 범위로 계속 처리하며 목표를 유지한다.

## 2026-11-26 확인 및 종료

- #7782를 GitHub에서 `completed`로 종료했다. [PR #7505](https://github.com/edwardkim/rhwp/pull/7505)의
  기존 검증과 [통합 PR #7568](https://github.com/edwardkim/rhwp/pull/7568)의 보정은 모두 기준 devel에
  포함됐다. 빈 문단 테두리는 해당 한컴 2020 Print 전 1쪽 Native·fresh WASM 모두 96.86991%였고,
  기존 17개 집중 검사가 통과했다. 판독 시 빈 줄, 테두리 아랫변과 뒤 문단 시작의 맞닿음이 보존됐다.
  [전후 TSV](../pr/assets/planet6897_green_20261002/stored-border-before-after.tsv) ·
  [판독 PNG](../pr/assets/planet6897_green_20261002/stored_border_fixed_native_review.png).
- #7780을 GitHub에서 `completed`로 종료했다. 부분 보정 뒤 [PR #9393](https://github.com/edwardkim/rhwp/pull/9393)
  병합본의 같은 HWP와 한컴 2020 PDF 전 1쪽은 Native·fresh WASM에서 모두 99.99%였다. 두 review에서
  노선도·소요시간·운임안내의 제목 획과 그림자가 복원됐으며 지도·시간표·운임표의 내용 소속과 비겹침을
  보호하는 검사도 현재 devel에 남아 있다. 이슈 본문의 예전 PDF는 시간·운임표 우측의 여러 셀을
  누락해 최신 검토가 사용한 같은 원본의 완전한 인쇄본을 근거로 사용했다.
  [Native review](../pr/assets/issue7766/mac-rebase-final/review/native/ktx/review_001.png) ·
  [WASM review](../pr/assets/issue7766/mac-rebase-final/review/wasm/ktx/review_001.png) ·
  [출처와 TSV 해시](../pr/assets/issue7766/mac-rebase-final/verification.json).
- #7781의 [한컴 2024 Print](../../pdf/issue7781/issue2439-repeat-table-2024.pdf)는 정확한 원본의
  가로 A4 10쪽이다. MCP 작업 `25a63090-2346-44b5-b5b3-4cd891dd8a7d`, 제품 `13.0.0.4225`,
  `hancom2024_pdf_driver_one_up`, PrintMethod `0`, SHA-256
  `0c35f90ad920d67c842d934e82f29538e91482b25bfb369664c24f46515de354`를 확인했다.
  96 DPI에서 기존 Microsoft PDF와 새 PDF는 전 10쪽 raw 100.00%였다. 이는 두 PDF 간 대조이며,
  새 Print 기준의 RHWP 전쪽 대조는 별도로 진행한다.
