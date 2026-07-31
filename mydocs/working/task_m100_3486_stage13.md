---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-31
---

# Task #3486 Stage 13 — HWP3 inline 도형·차례 세로 흐름 정합

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 기준 `devel`: `665982920b5d4739a58485060043a2a25b09c5e7`
- 기준 입력: `samples/HWP3-password-123456.hwp` (24쪽)
- 비교 입력: `samples/HWP5-nopassword-123456.hwp` (동일 문서의 한컴 HWP5 변환본, 24쪽)
- 기준 오라클: `pdf/HWP3-password-123456.pdf` (24쪽)
- 선행 결론: [Stage 12](task_m100_3486_stage12.md)의 제품명 display projection은 유지한다.

## 범위와 재현

현재 작업 브랜치 source를 `CARGO_TARGET_DIR=target/issue-3486-p3-20260731`,
`CARGO_INCREMENTAL=0`으로 `release-test` profile에 빌드했다. 비밀번호 값은 프로세스 인자,
출력, 이 문서에 기록하지 않고 local stdin launcher로만 HWP3에 공급했다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --key hwp3-stage13-post-toc-p01-06 \
  --hwp samples/HWP3-password-123456.hwp \
  --pdf pdf/HWP3-password-123456.pdf \
  --pages 1-6 --dpi 144 \
  --rhwp-bin <local-password-stdin-launcher> \
  --out /private/tmp/rhwp-issue-3486-p3-20260731/sweep-full-post-toc-p01-06
```

동일 명령을 `--pages 7-12`, `13-18`, `19-24`로 나누어 실행했다. 환경의 단일 실행
상한을 넘기지 않으면서도 각 실행에서 SVG/render tree 24쪽 전체와 지정 raster/overlay/review
6쪽을 모두 산출한다.

`pdftotext -bbox-layout`은 기준 PDF에서 exit 6으로 실패했다. 따라서 PDF 질문 marker
추출 단계는 생략됐으며, 이 결과를 marker 검증 성공으로 해석하지 않는다. PNG overlay와
render-tree 구조 분석은 정상 완료했다.

## HWP5·PDF 대조로 확정한 원인

HWP3 p3에는 두 독립 결함이 있었다. 4×2 표 우측 셀의 private 글머리표는 `▸`로 복원했고,
제목·inline 표 호스트의 줄간격은 HWP5 변환본의 저장 흐름에 맞췄다. p3의
`content_bottom_drift`는 이 두 보정 뒤 사라졌다.

| 대상 | 보정 전 HWP3 | HWP5/PDF 기준 |
| --- | --- | --- |
| p3 제목 inline 사각형 `LINE_SEG` | `ls=600` | `ls=960` |
| p3 폴더 표 `vpos` | `28540` | `29292` |
| 4×2 표 우측 셀 | 글머리표 누락 | `▸` 보존 |
| p1–p2 차례 항목 pitch | `line_height + 1629/1682 HU` | `line_height + 840 HU` |

전수 sweep에서 p1–p2가 추가 후보로 나타났다. 차례의 각 행은 `treat_as_char` 도형 하나와
marker·공백·쪽 번호만 가진다. 일반 160% 줄간격이 적용되어 행마다 `789 HU`씩 아래로
누적된 것이 원인이었다. HWP5 변환본의 같은 행은 후행 `840 HU`를 사용한다.

## 최소 수정과 회귀 계약

- `src/parser/hwp3/mod.rs`는 암호 HWP3 layout contract가 켜진 경우에만, marker·공백·쪽 번호와
  `treat_as_char` 도형 하나로 이루어진 차례 호스트를 판정한다.
- 이 구조에만 HWP5/PDF와 같은 `840 HU` 후행 간격을 적용한다. 일반 HWP3, 제목 텍스트가
  함께 있는 도형, marker-only 도형(`600 HU`), inline 표 계약은 바꾸지 않는다.
- `tests/hwp3_password_fixture.rs`는 p1–p2 11개 차례 행의 `vpos`·`line_height`·`line_spacing`과
  도형 구조를 고정한다.

## 전체 시각 스윕 결과

| 페이지 | 요청/완료 | 구조 후보 | 평균 pixel match | 평균 ink proxy |
| --- | --- | --- | ---: | ---: |
| 1–6 | 6/6 | 없음 | 93.12691% | 23.11761% |
| 7–12 | 6/6 | 없음 | 93.12251% | 9.92014% |
| 13–18 | 6/6 | 없음 | 92.95761% | 11.34811% |
| 19–24 | 6/6 | 없음 | 93.70045% | 10.52678% |

24/24 review PNG와 네 개의 contact sheet를 확인했다. p1–p2는 수정 뒤 차례 제목, 각 행,
leader, 쪽 번호의 세로 기준선이 PDF와 맞고, p3의 글머리표·표·본문 흐름도 유지된다. 나머지
쪽에서는 페이지 이탈, 표/텍스트 겹침, 문단 흐름 붕괴를 발견하지 못했다.

pixel/ink proxy는 글꼴 raster 차이를 포함하므로 단독 합격 기준이 아니다. 가장 낮은 ink proxy는
p7의 8.19555%이나 review PNG의 줄 흐름은 PDF와 일치하며 구조 후보도 없다. p1/p2의 proxy는
수정 전보다 각각 30.13753%→38.80579%, 28.99589%→61.94906%로 개선됐다. 이 수치는
목차 간격 보정의 보조 증거이며, 최종 판정은 overlay·구조 분석·사람 검토를 함께 따른다.

### 산출물

- [p1–6 review contact sheet](/private/tmp/rhwp-issue-3486-p3-20260731/sweep-full-post-toc-p01-06/hwp3-stage13-post-toc-p01-06/review_contact_sheet.png)
- [p7–12 review contact sheet](/private/tmp/rhwp-issue-3486-p3-20260731/sweep-full-post-toc-p07-12/hwp3-stage13-post-toc-p07-12/review_contact_sheet.png)
- [p13–18 review contact sheet](/private/tmp/rhwp-issue-3486-p3-20260731/sweep-full-post-toc-p13-18/hwp3-stage13-post-toc-p13-18/review_contact_sheet.png)
- [p19–24 review contact sheet](/private/tmp/rhwp-issue-3486-p3-20260731/sweep-full-post-toc-p19-24/hwp3-stage13-post-toc-p19-24/review_contact_sheet.png)

## 회귀 검증

- `cargo fmt --check`
- `cargo test --profile release-test --test hwp3_password_fixture` — 11 passed
- `cargo test --profile release-test --test issue_2151_hwp3_ghost_page` — 2 passed
  (`hwp3-sample11` 151쪽, `hwp3-sample14` 11쪽)
- `cargo test --profile release-test --tests` — 최종 exit code 0
- `cargo clippy --profile release-test --all-targets -- -D warnings`

초기 전체 integration은 일반 HWP3에서도 Shape TAC를 marker-only로 좁힌 탓에
`hwp3-sample11`이 152쪽으로 변하는 회귀를 발견했다. 일반 HWP3의 기존 600 HU 계약을
복원하고 암호 HWP3 layout contract에만 p3/차례의 구조별 분기를 남긴 뒤 위 검증을 모두
재실행했다.
