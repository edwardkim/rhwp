---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-30
---

# Task #3486 Stage 12-A — 중단 가능한 visual sweep 증적 계약

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 기준 `devel`: `42e1f125dae664bf50f2053784b5e0a213bea2e2`
- 선행 결론: [Stage 11](task_m100_3486_stage11.md)의 HWP3 전역 decode 보정·`ᄒᆞᆫ → 한` 치환 배제
- 권위 계획: [수행계획서 v2](../plans/task_m100_3486_v2.md) Stage 12-A

## 목적과 경계

이 단계는 page raster/overlay가 오래 걸려 단일 실행 한도에서 끝나더라도, 이미 완성된 페이지를
잃거나 전체 sweep 완료로 오인하지 않게 한다. PDF와 rhwp SVG의 일치율은 후보 정렬용 보조값이며,
Studio Canvas 또는 한컴 편집기의 최종 fidelity 판정이 아니다.

암호 HWP3 기준 fixture의 password 값·환경 값은 이 문서나 명령 로그에 기록하지 않는다. 실제
HWP3 PDF/Studio source-to-canvas 판정은 Stage 12-B의 별도 증적이다.

## 구현 중인 계약

`scripts/visual_sweep.py`에 다음을 추가했다.

- 기본 실행은 target output을 비우고 새로 만들며, 명시적 `--resume`만 기존 산출물을 유지한다.
- `run_manifest.json`은 입력 HWP/PDF SHA-256, Git HEAD, sweep script SHA-256, rhwp binary SHA-256,
  DPI와 pixel diff threshold를 기록한다. `--resume`은 어느 하나라도 다르면 즉시 거부한다.
- 각 페이지는 SVG·render tree·rhwp/PDF raster·compare·overlay·review·page metrics가 모두 생긴 뒤
  `pages/page-XXX.json`을 atomic replace로 기록한다. artifact가 빠진 checkpoint는 재사용하지 않는다.
- target `manifest.json`과 root `summary.json`은 완료 page manifest만 합쳐 `requested_pages`,
  `completed_pages`, `missing_pages`, `run_state`를 쓴다. contact sheet도 완료 페이지들만 사용한다.
- 같은 provenance에서는 `--pages 1-4`와 `--pages 5-8`을 `requested_page_shards`로 누적한다.
  shard 밖의 미완료 페이지는 `missing_pages`에 남는다.

## Focused smoke — checkpoint 계약

암호 입력을 위한 local stdin launcher가 현재 환경에 없으므로, 재개 메커니즘 자체는 독립 한컴 PDF
쌍 `samples/복학원서.hwp` ↔ `pdf/복학원서-2022.pdf`의 1쪽으로 검증했다. 이 fixture는 #3486의
원인 판정이나 렌더 품질 합격 근거가 아니다.

```bash
python3 scripts/visual_sweep.py \
  --key task3486-stage12-resume-smoke \
  --hwp samples/복학원서.hwp \
  --pdf pdf/복학원서-2022.pdf \
  --pages 1 \
  --out /private/tmp/rhwp_3486_stage12_resume_smoke \
  --rhwp-bin target/debug/rhwp \
  --dpi 144
```

| 확인 | 결과 |
| --- | --- |
| 첫 실행 | p1 checkpoint, compare·overlay·review·analysis·contact sheet 생성. `requested=[1]`, `completed=[1]`, `missing=[]`, `run_state=complete` |
| 같은 명령 + `--resume` | `resume: p001 checkpoint를 재사용합니다.`; raster/overlay를 다시 만들지 않음 |
| 같은 명령 + `--resume --dpi 96` | `--resume provenance가 기존 실행과 다릅니다: dpi`로 명시 거부 |
| review PNG 육안 확인 | 좌우 compare, overlay, proxy label과 review footer가 한 패널에 생성됨. overlay의 실제 문서 차이는 smoke fixture의 fidelity 판정으로 사용하지 않음 |

Focused 코드 검증도 통과했다.

| 명령 | 결과 |
| --- | --- |
| `python3 -m py_compile scripts/visual_sweep.py` | 성공 |
| `python3 scripts/tests/test_visual_sweep.py` | 11 tests 통과 — 기존 선택/raster·glyph 경계와 provenance 거부, shard 누적, 불완전 checkpoint 비재사용, 완료 page만 합친 `incomplete` summary |
| `git diff --check` | 성공 |

## Historical binary의 암호 HWP3 p3 checkpoint 계약 확인

비밀을 출력하지 않는 일회성 stdin launcher로 실제 기준 fixture의 p3도 실행했다. launcher는 기존
`target/release-test/rhwp` (`v0.8.2`)를 사용했으며 증적 생성 직후 삭제한다. 오래된
`target/debug/rhwp` (`v0.7.19`)는 같은 암호 HWP3에서 deflate 오류가 나므로 이 실행의 binary
provenance로 사용하지 않았다.

이 `release-test` binary는 실행 시점의 `devel` source에서 새로 빌드한 산출물이 아니다. p3 review의
검은 표 셀은 Stage 10 최신 Studio 결과(현재 Studio에서는 재현되지 않음)와 모순되므로, 아래 결과는
**checkpoint·provenance 계약 검증만을 위한 historical binary 실행**으로 제한한다. current `devel`의
bug-hunter 판정은 전용 target에서 다시 빌드한 binary로 재실행한 뒤에만 기록한다.

```bash
python3 scripts/visual_sweep.py \
  --key hwp3-password-stage12-resume-p003 \
  --hwp samples/HWP3-password-123456.hwp \
  --pdf pdf/HWP3-password-123456.pdf \
  --pages 3 \
  --out /private/tmp/rhwp_3486_stage12_hwp3_p003 \
  --rhwp-bin <password-stdin-local-launcher> \
  --dpi 144
```

| 확인 | 결과 |
| --- | --- |
| source export | SVG와 render tree 모두 24쪽 생성 |
| 선택 checkpoint | p3만 raster·compare·overlay·review·analysis 뒤 `pages/page-003.json`으로 완료 기록 |
| p3 자동 후보 | pixel match 90.84843%, visual proxy 6.57090%, `content_bottom_drift`와 `legacy_glyph_visual_mismatch` |
| PDF text layer | `pdftotext -bbox-layout` exit -6. marker 분석은 생략했고 text layer 일치 주장을 하지 않음 |
| 같은 provenance `--resume` | `resume: p003 checkpoint를 재사용합니다.`; p3 raster/overlay를 다시 만들지 않음 |

p3 review PNG에서 이 historical binary의 rhwp 쪽은 검은 표 셀과 목록 조판 차이를 보인다. 하지만 이는
current `devel` 증상이 아니라는 반증이 있으므로, parser·renderer 보정 후보나 Stage 12-B의 원인 증적으로
사용하지 않는다.

## Current `devel` source p3 재검증

`CARGO_TARGET_DIR=target/task3486-stage12`, `CARGO_INCREMENTAL=0`으로 현재 source의
`cargo build --profile release-test`를 수행해 전용 binary를 만들었다. 이 build는 전체 test나 clippy가
아니며, p3 실문서 재검증을 위한 focused executable 준비다. run manifest는 기준 `devel` HEAD뿐 아니라
sweep script SHA-256과 이 새 binary SHA-256을 기록했다.

| 항목 | 결과 |
| --- | --- |
| source export | SVG와 render tree 모두 24쪽 생성 |
| 선택 범위 | p3만 요청·완료. `requested=[3]`, `completed=[3]`, `missing=[]`; **24쪽 전체 완료 주장이 아님** |
| review 육안 판정 | historical binary에 있던 검은 우측 표 셀은 사라졌고 셀 내용이 표시된다. Stage 10의 current Studio 관찰과 일치한다. |
| p3 자동 후보 | pixel match 93.66418%, visual proxy 6.81538%, `content_bottom_drift`와 `legacy_glyph_visual_mismatch` |
| PDF text layer | `pdftotext -bbox-layout` exit -6. marker 분석은 생략했고 text layer 일치 주장을 하지 않음 |
| 같은 provenance `--resume` | `resume: p003 checkpoint를 재사용합니다.`; p3 raster/overlay를 다시 만들지 않음 |

따라서 검은 표 셀은 current `devel`의 재현 가능한 수정 대상이 아니다. 남은 p3 격차는 표·목록의
글꼴/행 흐름/문자 기호와 PDF의 차이이며, 다음 단계는 이를 raw HWP3 → IR → render tree/paint → Studio
Canvas로 분해하는 것이다. SVG/PDF 후보와 낮은 proxy만으로 parser·renderer 보정을 결정하지 않는다.

## 다음 순서

1. p3의 표·줄높이·본문 흐름 차이를 이 제품명 glyph와 분리해 source → IR → layout → paint로 조사한다.
   이 기록의 보정만으로 전체 페이지 fidelity 합격을 주장하지 않는다.
2. 중단 상태를 실제 암호 HWP3 입력에서 재현할 때도 page manifest만으로 `incomplete` summary가 남는지 확인하고,
   동일 provenance shard를 이어 실행한다.
3. 이 변경의 PR CI가 성공하면 review artifact와 merge 판단을 별도 기록한다.

## Stage 12-B — 닫힌 legacy 제품명 display projection

사용자가 제시한 p3 review PNG의 표 제목은 rhwp에서 옛한글 glyph로, 한컴 PDF에서는 현대 `한글`로
보였다. p19의 본문도 PDF가 `한메일`·`한팩스`를 현대 글리프로 인쇄하는 것을 확인했다. raw source와
IR을 고치는 전역 `ᄒᆞᆫ → 한`은 Stage 11의 배제 결론대로 도입하지 않는다.

### 구현 경계

- `composer`는 문단 전체 run을 이어 보며 줄·글자모양 경계를 넘어선 제품명도 처리한다. 첫 자모 위치에만
  현대 음절을 `display_text`로 두어 model-character offset 공간을 보존한다.
- 표 셀·머리말처럼 composer를 거치지 않고 직접 생성된 `TextRunNode`는 render tree 최종화 순회에서
  동일한 닫힌 어휘를 처리한다.
- 어휘는 `ᄒᆞᆫ글`·`ᄒᆞᆫ메일`·`ᄒᆞᆫ팩스`·`ᄒᆞᆫ소프트` 네 개뿐이다. 일반 옛한글과 CharOverlap은
  대상이 아니며 raw `text`, parser/IR, 검색, caret offset은 그대로다.
- visual sweep의 legacy glyph 후보는 raw `text`가 아니라 실제 페인트하는 `displayText`를 우선 읽는다.
  원문과 표시 문자열이 다를 때만 후보 JSON에 원문을 보조 증적으로 남긴다.

### Focused 회귀와 실제 fixture 재대조

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test legacy_hancom_product --lib` (`CARGO_TARGET_DIR=target/task3486-stage12`, `CARGO_INCREMENTAL=0`) | 3 passed — 일반 옛한글 비변경, 줄 경계 보정, composer 우회 직접 `TextRunNode` 보정과 raw text 보존 |
| `cargo fmt --check`, `git diff --check` | 성공 |
| `python3 -m py_compile scripts/visual_sweep.py` | 성공 |
| `python3 scripts/tests/test_visual_sweep.py` | 12 tests passed — `displayText`가 해결한 glyph는 raw text가 옛자모여도 후보로 재발하지 않음 |
| 실제 HWP3 PDF sweep | current 전용 release-test binary, 144 DPI, pages 3·19 요청/완료, `missing=[]`, `run_state=complete`; SVG/render tree는 24쪽 export되었지만 **24쪽 raster sweep 완료 주장은 하지 않음** |

실제 p3·p19 run의 `legacy_glyph_visual_pages=[]`, 각 페이지의
`legacy_glyph_visual_candidates=[]`를 확인했다. p3의 사용자 지적 표 제목도 rhwp에서 `한글 97의 사용
환경`으로 표시된다. 한편 p3은 `content_bottom_drift`가 계속 남고 pixel match 93.68602%, ink match
6.82500%이므로, 표와 줄 흐름 문제를 이 PR에서 해결됐다고 기록하지 않는다.
