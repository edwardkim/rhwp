# `samples/2010-01-06.hwp` 자동 보정(reflow) 후 빈 페이지 4 삽입

| 항목 | 내용 |
|------|------|
| 발견일 | 2026-04-30 |
| 샘플 | `samples/2010-01-06.hwp` (1월~6월 통합재정통계 6페이지 문서) |
| 환경 | rhwp-studio UI (브라우저), WASM 4,209,750 bytes |
| 트리거 | 문서 로드 시 "HWPX 비표준 감지" 모달에서 "자동 보정(권장)" 선택 |

## 증상

자동 보정 후 페이지 수가 6 → 7로 증가하고 페이지 4 자리에 빈 페이지가 삽입됨.

| 페이지 | "그대로 보기" (6p) | "자동 보정" (7p) |
|--------|-------------------|-------------------|
| 1 | 1월 (pi=0~12) | 1월 |
| 2 | 2월 (pi=14~26) | 2월 |
| 3 | 3월 (pi=28~41) | 3월 (pi=28~**39** — 빈 줄 빠짐) |
| **4** | **4월 (pi=43~57)** | **빈 페이지 (pi=40, 41 빈 줄 2개만)** ⚠️ |
| 5 | 5월 | 4월 |
| 6 | 6월 | 5월 |
| 7 | — | 6월 |

빈 페이지(idx=3) SVG: svgLen=470, textCount=0, rectCount=2 (페이지 외곽선만).

## 재현

### 1) studio UI

```bash
cd rhwp-studio && npm run dev
# 브라우저에서 file-input 으로 samples/2010-01-06.hwp 로드
# 모달에서 "자동 보정 (권장)" 클릭 → 7페이지, 4번이 빈 페이지
```

### 2) CLI (디버깅용)

```bash
./target/release/rhwp dump-pages samples/2010-01-06.hwp           # 6페이지
./target/release/rhwp dump-pages samples/2010-01-06.hwp --reflow  # 7페이지, 4번 빈 페이지
```

(`--reflow` 옵션은 디버깅용 — `RHWP_DUMP_REFLOW=1` 환경변수 동작)

### 3) WASM 단독 검증

```bash
cd rhwp-studio
node e2e/check-2010-pages.mjs --choice=fix    # 7p, idx=3 빈 페이지
node e2e/check-2010-pages.mjs --choice=asis   # 6p, 정상
```

## 근본 원인

`reflow_linesegs_on_demand`(src/document_core/commands/document.rs:387)가 비표준 lineseg(paragraph당 1개)를 한컴 textRun reflow 기준으로 재계산할 때:

**paragraph 37** (`" ㅇ △19,513십억원 = △7,020십억원(통합재정수지) - 12,4..."`):

| 측정 | reflow 전 | reflow 후 |
|------|----------|----------|
| line_segs | 1개 (vpos=46043) | 2개 (vpos=46043, 48121) |
| height | 17.3px | 34.6px (+17.3) |

이 paragraph 37의 +17.3px 증가가 페이지 3 누적 높이를 부풀려 페이지 끝에 있던 빈 줄(pi=40, 41 각 16px)을 페이지 3에 못 들어가게 만든다. 빈 줄 2개가 페이지 4로 밀려나고, paragraph 42(`page_break_before=true`)가 paragraph 43 직전에 있어 4월 본문은 다시 페이지 5로 시작 → **페이지 4 = 빈 줄 2개만 = 빈 페이지**.

reflow 누적 영향:

| 페이지 | reflow 전 used | reflow 후 used | 차이 |
|-------|---------------|---------------|------|
| 1 | 725.4px | 744.4px | +19.0 |
| 2 | 725.4px | 744.4px | +19.0 |
| 3 | 751.0px | 757.0px | +6.0 |

## 핵심 코드

| 파일 | 위치 | 역할 |
|------|------|------|
| `src/document_core/commands/document.rs` | 387 | `reflow_linesegs_on_demand` — paragraph 별 reflow 적용 후 paginate 재호출 |
| `rhwp-studio/src/main.ts` | 438 | `if (choice === 'auto-fix') { wasm.reflowLinesegs(); }` — 사용자 선택 |
| `rhwp-studio/src/ui/validation-modal.ts` | — | 모달 기본 포커스가 "자동 보정" 버튼 |
| `src/main.rs` | dump-pages 명령 | `--reflow` 옵션 추가 (디버깅용) |

## 수정 방향 후보

| 옵션 | 변경 | 영향 / 리스크 |
|------|------|----------------|
| **A** | paginator가 페이지 끝의 trailing 빈 줄을 다음 페이지로 넘기지 않고 흡수 (`section.section_def.hide_empty_line` 자동 활성화 또는 페이지 끝 빈줄 흡수 룰) | paginator 변경. 다른 문서 영향 가능 |
| **B** | `reflow_line_segs` 알고리즘이 한컴 결과와 정렬되도록 정정 — paragraph 37이 한컴에서 실제로 1 line이라면 reflow 후에도 1 line | reflow 정확도 향상. 회귀 위험 큼 |
| **C** | studio UI 모달 기본값을 "그대로 보기"로 변경 (validation-modal.ts) | 임시 회피. 사용자가 명시적으로 자동 보정 선택 시 동일 문제 |
| **D** | 자동 보정 후 paginator가 빈 페이지(빈 줄만 있는 페이지)를 자동 제거 | post-processing. 정확하지만 다른 case 영향 |

## 확인 필요 사항 (수정 전)

1. 한컴 한글 2010/2020 + 한컴독스에서 본 파일이 6페이지인지 7페이지인지 (정답 기준)
2. 한컴이 paragraph 37을 1 line으로 표시하는지 2 lines로 표시하는지
3. `section.section_def.hide_empty_line` 값 확인

(메모리 가이드 [PDF 비교 결과는 절대 기준이 아님](feedback_pdf_not_authoritative.md): PDF 200dpi 보조 ref. 한컴 2010/2020 정답지 함께 점검 필수.)

## 임시 회피책

- 사용자 안내: 모달에서 "**그대로 보기**" 선택 → 6페이지 정상 출력
- 또는 옵션 C 적용 (validation-modal.ts 기본 포커스 변경)

## 별개 이슈로 등록 후보

본 트러블슈팅 분석을 GitHub Issue로 등록하여 추적 가능. 본질 수정(옵션 A 또는 B)은 큰 작업으로 별도 task 권장.
