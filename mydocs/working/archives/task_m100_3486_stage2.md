---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-28
---

# Task #3486 Stage 2 — HWP3 PUA·쪽 배경·문단 stream 구조 복원

Issue: [#3486](https://github.com/edwardkim/rhwp/issues/3486)

## 이 단계의 결정

Stage 1의 기준선과 달리, 작업지시자가 제공한 최신 한컴 PDF의 1쪽에는 중앙의 컬러 그림이 실제로
표시된다. HWP3 원본의 추가 정보 블록 #6을 단순히 버리면 이 그림이 사라지므로, "alpha=0이므로
표시하지 않는다"는 이전 가설은 수용하지 않았다. 이 단계에서는 원본 블록의 내장 이미지·fill mode·
brightness·contrast·effect를 공통 IR의 쪽 `BorderFill` 이미지 채우기로 보존했다.

같은 문서의 머리말에는 일반 공개 글꼴에 없는 한컴 PUA 회사명과 Enter pictogram이 있다. HWP3의
조합형 여섯 코드를 HWPX 비교 문서에서 확인한 공통 PUA로 보존하고, renderer의 별도 확인표에서만
표준 한글/의미 fallback으로 투영했다. 미확인 PUA 전체를 범위 치환하지 않았고 원문 IR도 유지한다.

## 구현 결과

- HWP3 문단 앞뒤 간격의 저장 스케일을 HWP5 변환본의 `LineSeg.vpos` 계약에 맞췄다. 문단 여백과
  문단 간격은 같은 HWP3 hunit으로 보이지만 공통 IR의 저장 배율이 같지 않은 경우를 분리했다.
- 인라인 개체의 가시 마커는 하나여도 HWP5 `PARA_TEXT` stream에서 8 UTF-16 slot을 차지한다. HWP3
  `LineInfo.start_pos`와 CharShape가 뒤쪽 도형 뒤에서 앞당겨지지 않도록 이 좌표 계약을 복원했다.
- HWPX `DISTRIBUTE_SPACE`인 회사명+우측 logo 줄은 일반 글자 나눔이 아니라 trailing space 하나가
  남은 폭을 흡수해야 한다. 확인된 전체 PUA 시퀀스와 정렬값이 일치하는 줄에만 제한해 적용했다.
- 구현 커밋: `b775832f4` (`fix: HWP3 한컴 PUA와 쪽 배경 복원`).

## focused 검증

모든 Rust 실행은 `CARGO_TARGET_DIR=target/task_3486_render_v2`, `CARGO_INCREMENTAL=0`에서 수행했다.

| 명령 범위 | 결과 |
| --- | --- |
| HWP3 회사명 조합형 decode 단위 테스트 | 1 passed |
| `hwp3_password_fixture` | 2 passed |
| `issue_3486_hancom_pua_display` | 2 passed |
| `issue_3486_hancom_company_pua_alignment_tests` | 1 passed |

이전 원격 MCP 실행 보고에는 HWPX PDF 24쪽과 server validation `ok`가 있었지만, 그 Stage 18 기록은
이 Mac에 보존되어 있지 않다. 현재 제공된 평문·암호 HWPX를 로컬 `info`로 다시 열면 모두 23쪽이며,
작업지시자가 제공한 Studio 화면의 23쪽 표기와도 일치한다. 그러므로 원격 24쪽 수치와 그 실행에서
파생된 비교는 이 단계의 로컬 증적·수용 근거로 사용하지 않는다. 잘못된 암호 입력이 PDF를 만들지
않는 MCP 계약 자체는 별도 암호 열기 작업에서 확인했으며, 비밀번호·token은 이 기록에 남기지 않는다.

## 시각 대조와 현재 판정

| 비교 | 결과 | 해석 |
| --- | --- | --- |
| HWP3 원본 ↔ 제공된 한컴 PDF | 양쪽 24쪽, 1쪽 `visual_accuracy_proxy_percent` 12.55730 | 머리말 회사명 tofu는 사라졌으나 HFT/폰트, 중앙 그림 색조, 본문·목차 기하 차이가 남음 |
| HWPX 비교 문서 | 현재 rhwp local 23쪽 | 이전 임시 sweep의 `20.22520`은 24쪽 MCP PDF와의 페이지 대응을 이 기록에서 재현·보존하지 못했다. 수용 근거에서 제외하고, 23쪽 기준 PDF를 고정한 뒤 다시 대조한다. |

현재 sweep의 일시 산출물은 다음에 있다. 아직 수정이 진행 중이므로 PR에 남길 최종 증적은 아니며,
최종 단계에서 `mydocs/pr/assets/`의 보이는 PNG로 별도 고정한다.

- HWP3 compare: `tmp/pdfs/task3486/sweep-hwp3-pua-v3.YNEYpR/hwp3-pua-display-v3/compare/compare_001.png`
- HWP3 overlay: `tmp/pdfs/task3486/sweep-hwp3-pua-v3.YNEYpR/hwp3-pua-display-v3/overlay/overlay_001.png`
- HWP3 review: `tmp/pdfs/task3486/sweep-hwp3-pua-v3.YNEYpR/hwp3-pua-display-v3/review/review_001.png`
- HWPX 후보 자료: `tmp/pdfs/task3486/sweep-hwp5-pua-v5.N1hM3r/hwp5-pua-display-v5/`
  (페이지 수 대응을 다시 고정하기 전에는 참고용이며 판정·PR 증적에 사용하지 않는다.)

레거시 HWP3 한컴 PDF의 PUA 텍스트층 때문에 `pdftotext -bbox-layout`이 종료한 경우가 있었으나,
PDF raster와 SVG raster는 정상 생성됐다. 따라서 HWP3 24쪽 대조에서는 marker 분석만 생략했고
raster overlay를 계속 사용했다.

## 다음 단계로 넘기는 사항

1. 중앙 그림이 기준 PDF보다 옅은 원인을 HWP3 추가 정보의 효과값, 공통 image effect 적용 순서,
   watermark/opacity preset 중 하나로 분리한다.
2. HWP3/HWPX 양쪽의 1쪽에서 그림 위치와 본문 line flow를 별도로 비교한다. 글꼴 메트릭으로 남는
   차이와 parser/layout 결함을 같은 수치로 묶어 수용하지 않는다.
3. 전체 CI나 최종 PR 증적 생성은 Stage 3의 원인별 수정과 focused 검증을 먼저 마친 뒤 작업지시자
   승인 범위에서 수행한다.
