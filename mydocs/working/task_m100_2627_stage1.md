---
kind: working
status: active
issue_or_pr: 2627
stage: 1
last_verified: 2026-07-21
---

# PR #2627 · #2655 · #2561 메인터너 통합 Stage 1

## 목적

- PR #2627의 빈 꼬리말 밴드 각주 회수 규칙을 최신 `upstream/devel` 조판 경로에 수동 통합한다.
- PR #2655에서 이미 반영된 #2552 변경은 보존하고, 남은 `dump-pages` 오류 메시지와 미지 옵션 조기 종료만 보완한다.
- PR #2561의 r17 서베이 보고서는 원시 산출물 보존 여부를 확인해 재현 가능한 근거만 남길 수 있는지 판정한다.

## 현재 기준

- 기준 커밋: `upstream/devel`의 `8ed630944567cfe23e61e7d89ea6f7a67c77f74f`
- PR #2627은 `typeset.rs`의 이후 조판 변경과 충돌하므로 원 커밋을 그대로 적용하지 않는다.
- PR #2655의 범위 검사와 파싱 실패 처리는 PR #2552로 이미 반영됐으며, 오류 값 표시와 미지 옵션의 즉시 종료만 남아 있다.
- PR #2561은 보고서 외에 `BINARY_FINGERPRINT`, aggregate, 이동표 등 원시 근거가 PR 트리에 없다.

## Stage 1 완료 조건

1. 최신 조판 구조에 맞는 #2627 수동 통합과 대상 회귀 테스트를 준비한다.
2. `dump-pages`의 잘못된 옵션이 파일 읽기 전에 즉시 실패하도록 보정하고 테스트 경로를 마련한다.
3. #2561 문서의 원시 근거 보존 여부와 처리 방침을 명확히 기록한다.

## 구현 및 대상 검증 결과

### PR #2627 수동 통합

- 최신 `typeset.rs`의 #2439 이후 조판 규칙은 유지했다.
- 구역에 실제 꼬리말 정의가 없고 각주가 있는 경우에만 `footer_area.height`를 각주 예약에서 회수했다.
- 대표 샘플 `samples/issue2559/1341000_research_report_footnotes.hwp`는 rhwp 94쪽으로 고정됐다.
  - 한컴 2020 MCP Print 기준 PDF는 92쪽이다.
  - 수정 전 98쪽이라는 원 PR의 보고 대비 과다분할을 4쪽 줄였지만, 한컴과의 +2쪽 차이는 남아 있다.
- `tests/issue_1733.rs`의 HWP/HWPX는 모두 242쪽에서 241쪽으로 바뀌었다.
  - 한컴 2024/PDF 기준 242쪽과 다른 알려진 `-1` 트레이드이므로, 테스트 이름과 주석에 이를 명시하고 241쪽을 현재 핀으로 고정했다.

### PR #2655 수동 통합

- 이미 통합된 #2552의 범위 검사·파싱 실패 처리는 유지했다.
- `dump-pages`에만 잘못된 페이지 값 표시, 실제 사용 옵션 이름 표시, 미지 옵션 즉시 종료를 반영했다.
- `tests/dump_pages_cli.rs`는 잘못된 `-p`, 값 누락, 미지 옵션이 파일 읽기와 문서 전체 덤프로 이어지지 않음을 확인한다.

### PR #2561 문서 보정

- 원 보고서는 보존하되 `HEAD=485106b9`, 바이너리 지문, aggregate, 이동표가 저장소와 PR에 없다는 재현성 공백을 맨 앞에 명시했다.
- PAGE_DELTA 전체 434건과 서술 버킷 403건의 차이 31건, 픽셀 분류 9,942건과 표본 10,000건의 차이 58건도 원 manifest 없이 재분해할 수 없음을 기록했다.
- 병합된 PR 상태와 #2559 후속 PR 상태를 최신화했다.

### 시각 검증

- 기준 PDF: `pdf/issue2559/1341000_research_report_footnotes-2020-print.pdf`
  - HWP 2020 MCP Print, job `2721affd-0888-4785-89db-b0b96da76466`, `run_status=0`, `validation=ok`
  - 92쪽, SHA-256 `ec7cebed92cf114da486eb4f8b4cbefa0739243e037d9a09ceebc433063e7e5e`
- visual sweep: `target/visual-sweep-issue2559/`
  - 선택 페이지 1, 46, 92에서 frame/line/order/tail 후보 `0/3`
  - rhwp SVG 94쪽 / 기준 PDF 92쪽이며, 선택 review 이미지의 픽셀 기반 일치도는 글꼴과 누적 조판 차이의 영향으로 낮다.
  - 따라서 이 sweep은 clipping·순서 겹침 부재의 보조 근거일 뿐, 한컴 92쪽 완전 정합이나 글꼴 fidelity의 증명으로 사용하지 않는다.

### 실행 명령

```bash
CARGO_INCREMENTAL=0 cargo test --profile release-test \
  --test dump_pages_cli \
  --test issue_2559_footnote_footer_band \
  --test issue_1733

CARGO_INCREMENTAL=0 cargo build

python3 scripts/task1274_visual_sweep.py \
  --key issue2559-footnote \
  --hwp samples/issue2559/1341000_research_report_footnotes.hwp \
  --pdf pdf/issue2559/1341000_research_report_footnotes-2020-print.pdf \
  --pages 1,46,92 \
  --rhwp-bin target/debug/rhwp \
  --out target/visual-sweep-issue2559
```
