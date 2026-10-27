---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase12-survey-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase12-survey-report.md` · 원문 SHA-256: `d24a5c461e278accc352214ad0fe698f78ae247b5983dbfde50c8f08c098db8f`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## 추가 조사와 기존 검사 검증 — 2026-10-26

조사 head는 `238de6de41a208648f4ade6b494fe433a81c1fce`, 브랜치는 `audit/8232-px-regressions-20261025`입니다.

**기존 검사 자체의 기대값도 검증합니다.** 기존 형태·단언·동작의 무조건 유지를 전제로 하지 않습니다. 독립 근거에서 유효한 검증 목표를 확인하고 부적합한 기대값/방식은 대체·제거하며, 실제 보정 대상만 `issue_8232` 이름으로 재구성합니다. 이 기준은 현 이슈 본문에 반영되어 있습니다.

### 새로 확인한 검사 의미

- CLI/OOXML 차트/계획 schema 181그룹, replay fixture wrapper 133그룹, shaping/kerning/HML 131그룹, 차트 CSV 25그룹의 직접 단언을 확인했습니다. 그룹 수는 위반 수나 실행 PASS 수가 아닙니다.
- 중첩 helper 132그룹과 replay helper15그룹, Studio 검사/helper52그룹을 추가 대조했습니다. 동적 입력 경로는 chart corpus의 `samples/chart`와 scene fixture의 `tests/fixtures/render_backend/scenes/*.json` 등 실제 loader 범위로 구분했습니다.
- 소수점이라는 이유로 모두 페이지 px 핀으로 판정하지 않았습니다. 차트4.3→91.7은 데이터, shaping의 정밀한 bbox는 직접 선언한 원점/크기와 글꼴 design-unit의 변환, Studio 캐시의 숫자는 페이지/셀 식별자입니다. 독립 HarfBuzz CLI의4개 수직 geometry와 kerning 합계차이−120 font units를 연결했습니다. 이 단위 검증은 HWP 페이지 시각 자격을 대신하지 않습니다.
- 원본74쪽과 줄간격100% 복제 양쪽에74쪽을 강제하는 중첩 helper를 발견했습니다. 원본의 증거만으로 편집 복제까지 승인하지 않습니다. runtime kerning fixture의 고정1쪽도 독립 계약 확인 대상으로 남겼습니다.

### 78494 법령 원본: 현재 전체74쪽과 기존 위치 검사

입력은 `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx`입니다. **확장자와 달리 실제 HWP5/한컴2018 저장본**이며, 기존 독립 engine2020 Print PDF와 대조했습니다.

- 입력 SHA256: `5090e9d20ce75814febeb4b79e106ea6abfd2546e511f229900d137d42623617`
- PDF: `pdf/issue7642-legal-table-origin/78494-virtual-convergence-industry-decree-2020.pdf`
- PDF SHA256: `674ef63b51adc926c7c89f530bd142a45bb64cb655a7117bc20285d7a6e5a8d2`

| 현재 출력 | 전쪽 | 최저 raw | 90% 미만 | 누락/추가 | 제목 검사21쪽 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Native Skia PNG | 74/74 | 95.57490% | 0 | 0 | 100.00000% |
| fresh WASM PrintCanvas PNG | 74/74 | 95.57554% | 0 | 0 | 100.00000% |

PDFium5.13.0/96CSSdpi/색상 경계 보정 전2px 기준입니다. Mac fresh WASM은 locked wrapper의 web/no-opt 로컬 대체 빌드이며 Docker 최적화 결과가 아닙니다. source/build/JS/WASM/입력/PDF/canonical script와 전쪽 TSV hash를 고정했습니다.

21쪽 `□ 간접편익`의 독립 PDF 문자 원점을 직접 읽었습니다. CSS 좌표 `(75.47909037272136,472.2838541666667)`이 기존 기대값과 일치합니다. Native/WASM의 review·overlay와 제목/후속 표를 확대 판독하여 소속·내용·순서 대응을 확인했습니다. 일부 선 가장자리·글리프 차이는 남습니다. raw100은 픽셀 완전 동일을 뜻하지 않습니다.

기존 제목 검사는 ±2px와 원본 소속·제목 내용·단일 출현·부모 표 종료 뒤 배치를 함께 검사하므로 **해당 위치 검사 유지 근거**를 확보했습니다. 줄간격100% 복제의 독립 Print/전체 쪽수 근거와62쪽 `LAYOUT_OVERFLOW` 진단·실제 paint 대응은 별도 미완료입니다.

### 실제 실행과 남은 작업

- 기존 replay fixture 모듈: **196 PASS / 0 FAIL / 0 ignored**
- 기존 Studio 단위 검사3파일: **21 PASS / 0 FAIL / 0 skipped**
- 기존 원본/간격/제목 모듈: **5 PASS / 0 FAIL / 0 ignored**

기존 코드를 그대로 실행한 집중 검사입니다. standalone Rust 테스트를 integration 전체·CI 통과로 보고하지 않습니다.

현재 통합 원장은 **9,854그룹·2,103파일**, 미분류는 **5,129그룹**입니다. 시각 증거 목록은 **88조건·각 backend2,486쪽**으로 갱신했습니다. 수치 목록에 들어갔다는 이유로 검사 적합/문서 완료로 판정하지 않습니다.

제품·테스트 수정과 이름 변경은 아직0건이며 PR은 생성하지 않았습니다. 전수 판정, 부적합 검사 실제 보정·새 이름·호출/등록 갱신과 필수 재검증이 남아 있습니다.

로컬 조사 기록은 `output/issue8232-audit/`의 `audit-ledger.json/.tsv`, `phase12-original-title-qualification.json`, `paired-evidence-catalog.json`, `existing-6776-tests.log`, `existing-studio-contracts-tests.log`, `font-unit-independent-oracles.json`에 보존했습니다. 로컬 파일을 공개 GitHub 첨부로 주장하지 않습니다.
