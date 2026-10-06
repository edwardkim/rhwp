---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage19_title_glyph_fit.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 19단계: #6712 제목 문자 겹침

## 분석

- 18단계 checkpoint: `e67e359a8` (코드 `ad990fd3f`). 제목 겹침을 사용자가 재지적했다.
  전체 9,089 테스트 통과는 glyph paint 폭의 정확성을 입증하지 않는다.
- 한국어 제목 원본 face는 `휴먼굵은팸체`다. 내장 메트릭은 em=512 대비 음절별 폭
  253~397인 비전각 글꼴이다. 실제 SVG의 '여→름' advance는 19.8267px,
  font-size=34.6667px로 한 글자가 1em보다 좁다.
- Chrome CDP `CSS.getPlatformFontsForNode`는 실제 공급 face를
  `Pretendard-Regular` (custom font)로 확인했다. '여' bbox 폭은 29.9534px라 다음
  글자의 시작점을 넘는다. 폰트 개수나 일반 fallback 추가만으로 해결되지 않는다.
- SVG glyph-fit은 ASCII/반각 괄호만 `textLength`로 맞추고 비전각 한글은 제외한다.
  원본 narrow advance와 fallback glyph 폭이 분리되어 겹친다. 저장된 x를 벌려 해결하면
  원본 줄 폭·정렬·쪽나눔이 달라지므로 배치 좌표는 유지해야 한다.

## 수정 계획

1. 기존 검증 프로세스가 종료된 뒤 변경한다. 18단계 잔여 결과는 별도로 기록한다.
2. 내장 메트릭으로 **1em보다 좁은 단일 완성형 한글**임이 확인된 SVG glyph에만
   원본 glyph advance를 `textLength`로 전달한다. 글꼴 이름이나 #6712 파일명 하드코딩을
   하지 않는다. 전각 한글, 자모 cluster, 미지의 face, 음수 자간 등은 기존 정책을 보존한다.
3. 배분 간격, 장평, 첨자/그림자 기존 보정과 이중 적용되지 않도록 기존 glyph-fit 경로를
   재사용한다. source-side test는 늘리지 않고 `tests/cases/`에 계약을 추가한다.
4. 실제 원본의 title SVG 및 Chrome에서 글자 bbox가 인접 advance를 넘지 않는지 검사한다.
   제목 확대와 전체 1/2쪽을 다시 보고 새 결과만 최종 코멘트 증적으로 선택한다.
5. focused/제어군과 필수 lint를 실행한다. 최종 PR 준비에서는 변경 후 전체 회귀,
   새 sample baseline, native-Skia/WASM 게이트를 완료하고 단계 결과와 함께 일반 커밋한다.

## 범위

#6712의 잔여 문자 겹침만 처리한다. 다른 미해결 이슈 구현이나 PR 생성은 진행하지 않는다.
로그, CDP JSON, 임시 확대 캡처는 `/tmp`에만 보관한다.

## 구현과 검증

- SVG의 기존 glyph-fit 경로에 단일 완성형 한글의 비전각 메트릭 확인을 추가했다.
  명시적 자간은 0인 경우로 제한하고, 미지의 face/전각 한글/자모 cluster의 기존 출력을
  보존한다. 양수 배분 간격은 기존 `glyph_fit_advance`가 제외한다. 그림자도 같은 폭을 쓴다.
- `tests/cases/issue_6712_title_glyph_fit.rs`의 6개 계약 통과(160 skipped, 0.047초, exit 0).
  원본 제목의 11글자, 두 페이지 유지, x 좌표 유지 및 glyph 폭 맞춤을 포함한다.
- 기존 SVG 제어군 52 passed / 4,016 skipped / 0.098초 / exit 0.
  `cargo fmt --all`, `git diff --check` 통과.
- 첫 focused 실행은 fmt 전후 source weight로 suite 배정이 달라져 **0 tests / exit 4**였다.
  성공으로 세지 않았다. fmt 후 `--prepare`를 재실행하여 wrapper와 생성 suite를 일치시키고
  실제 6개가 실행된 summary를 확인했다.
- 이전 18단계 나머지 게이트도 종료됐다: workspace build 150.616초, all-target Clippy
  125.114초, manifest check, Node raster 2 tests 모두 exit 0. 자동 pipeline의 browser 항목은
  env 미지정으로 skip이며 별도 env 지정 실행의 실제 1 test OK와 구분한다.
  이 결과는 제목 수정 전 코드의 검증이지 새 코드 전체 통과가 아니다.

## 실제 브라우저 검증

- 수정 전 원본 SVG에서 인접 glyph bbox 7쌍 겹침을 검출했다(exit 1).
- 새 CLI SHA-256: `47c5248eebcaa4d9f040a1bf91183d5afa31fccd57a243a711968e2b77fbe824`.
  `rhwp export-svg <한국어 원본> -o /tmp/rhwp-stage19-title-fixed -p 0 --json`으로 재산출했다.
- 새 `textLength`는 원본 advance와 일치한다. 실제 Chrome의 대체 face는 이전과 같은
  Pretendard-Regular이며, 폰트를 갈아 끼워 통과시킨 것이 아니다.
- 수정 후에도 `getBBox()`의 보수적인 경계 상자는 '감→염'에서 0.783px 겹친다고 보고했다.
  실제 ink 교차인지 구분하기 위해 각 glyph를 하나씩 표시하여 같은 Chrome 화면을 캡처하고,
  RGB 각 채널<128인 검은 픽셀의 좌우 범위를 비교했다. 최종 비교 이미지를 수정한 것이 아니라
  글리프별 진단 캡처이며 전부 임시 파일이다.
- 1배 빈 열 간격: `3, 2, 20, 3, 2, 19, 1, 5, 21, 4`px.
  2배 빈 열 간격: `5, 2, 39, 5, 2, 36, 1, 8, 41, 7`px. 모든 쌍이 양수(exit 0).
  제목 확대 1/2배를 직접 열어 검은 글자의 실제 교차가 사라진 것을 확인했다.
- 원본 휴먼굵은팸체 outline이 설치된 것과 같다고 주장하지 않는다. 해당 face가 없는 환경에서
  fallback 글리프가 원본 배치 폭을 넘던 결함을 수정한 것이다. 다른 폰트 디자인 차이는 남는다.
- 현재 단계는 focused/브라우저 검증까지다. 다음 단계에서 전체 회귀와 최종 sweep,
  필수 lint 및 남은 baseline/native-Skia/WASM 게이트를 수행한 후 PR 준비 여부를 판단한다.
