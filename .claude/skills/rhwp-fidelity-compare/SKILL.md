---
name: rhwp-fidelity-compare
description: 한컴이 내보낸 공식 PDF와 rhwp export-svg를 쪽별로 대조합니다. tools/fidelity_compare의 페이지 시트·픽셀 diff% 랭킹(최악 쪽 우선)·text-report.tsv 문자 멀티셋(소실/과잉/치환)을 후보 검출로 읽고, --font-style·로컬 face 별칭·두부 오염·RHWP_FONT_PATH_DIR·provenance를 닫습니다. 트리거 — 사용자가 "한컴 PDF와 비교", "공식 출력 기준 대조", "fidelity_compare 돌려줘", "한컴이 뽑은 PDF랑 rhwp가 같은지"를 요청할 때. 독립 한컴 PDF가 있을 때만. 자기 일관성 render-diff나 버그 헌팅 여정과는 다른 축이다.
---

# rhwp-fidelity-compare — 한컴 기준 PDF 대조 Skill

에이전트가 **한컴이 내보낸 공식 PDF** 와 `rhwp export-svg` 를 쪽별로
대조할 때 쓰는 규약이다. 코어는 이미 있는
`tools/fidelity_compare/fidelity_compare.py` 와 `rhwp export-svg` 뿐이다.

이 스킬은 **실 에이전트 경로**다. gym 이 아니고, 새 CLI 명령을 발명하지
않는다. `rhwp-visual-regression`(자기 일관성 render-diff) 과
`bug-hunter`(여정 방법론) 를 이 폴더에서 재작성하지 않는다.

픽셀 diff% 와 문자 멀티셋은 **후보 검출**이지 절대 판정이 아니다.
최종 시각 판정은 유지자/작업지시자가
[visual_verification_governance.md](../../../mydocs/manual/verification/visual_verification_governance.md)
를 따라 내린다.

상세는 `references/` 를 단계별로 연다. SKILL.md 는 인덱스와 정지 규칙만 담는다.

정본:

- [`tools/fidelity_compare/README.md`](../../../tools/fidelity_compare/README.md)
- [`mydocs/manual/verification/visual_verification_governance.md`](../../../mydocs/manual/verification/visual_verification_governance.md)

## 언제 쓰는가 / 언제 쓰지 않는가

| 손에 있는 것 | 정직한 도구 | 이 스킬 |
| --- | --- | --- |
| 한컴 도구·버전·경로가 기록된 **독립 공식 PDF** + 원본 HWP/HWPX | `tools/fidelity_compare` | **여기** |
| 편집/변환 전후 두 HWP, 공식 PDF 없음 | `rhwp render-diff` | 쓰지 않음 → `rhwp-visual-regression` |
| 같은 파일을 두 번 렌더해 결정성만 본다 | `render-diff A A` | 쓰지 않음 |
| 원인 미확정 실사용 결함을 여정으로 좁힌다 | bug-hunter playbook | 후보 검출만 넘긴다 |
| gym pack / 채점 / 리더보드 | gym 도구 | **금지** |

독립 한컴 PDF 가 없으면 이 하네스는 정직하지 않다. 샘플 옆의 동반 PDF 는
도구·버전·출처를 확인하기 전에는 최종 기준으로 승격하지 않는다.

## 바이너리·venv

새 `fidelity-*` 하위명령은 없다. 비교는 저장소 로컬 venv 의 Python 이
기존 하네스를 호출한다.

```bash
# 저장소 루트, 최초 1회. 시스템 pip / --break-system-packages 금지
python3.12 -m venv venv
venv/bin/python -m pip install pypdf pypdfium2 pillow

# Windows
py -3.12 -m venv venv
venv\Scripts\python.exe -m pip install pypdf pypdfium2 pillow
```

- POSIX: `venv/bin/python`
- Windows: **`venv\Scripts\python.exe`**
- Chrome/Chromium 은 `--text-only` 가 아닐 때 필수
- `--text-only` 는 `pypdf` 만 있으면 되고 Chrome·pypdfium2 를 요구하지 않는다
- 하네스 SVG 는 기본 `--font-style` (로컬 face 별칭, embed 아님)

실행 파일 자동 탐색: `rhwp` 는 `target/release-test` → `target/release` →
`PATH`. Chrome 은 플랫폼 기본 경로. 안 맞으면 `RHWP_BIN` / `CHROME_BIN`.

수정 직후 비교 전 최신 바이너리:

```bash
cargo build --profile release-test --target-dir target/pr-review
RHWP_BIN=target/pr-review/release-test/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan
```

이 `cargo build` 는 시각 대조용 **컴파일 전용** 이다. Rust 테스트를 대신하지
않는다.

## 사다리 (강제 순회 아님)

```
독립 한컴 PDF 가 있는가
  │
  ├─ 없음 ──▶ rhwp-visual-regression (render-diff). 여기서 멈추지 말고 스킬을 갈아탄다 (F01)
  │
  ├─ 있음, 텍스트 후보만 먼저 ──▶ --text-only [--export-all-svg] [--layout-ledger] (F02)
  │     text-report.tsv 소실/과잉/치환
  │     page-count-ledger.tsv 쪽수 불일치는 후보
  │
  ├─ 픽셀 시트 ──▶ (기본 모드) cmp-pNNN.png + report.tsv 최악 쪽 우선 (F03)
  │     Chrome 필수. 없으면 --text-only 로 내려가거나 정지 (F10)
  │
  ├─ 글꼴 두부 의심 ──▶ --font-style · RHWP_FONT_PATH_DIR · svg-glyph-risk-report.tsv (F04)
  │
  └─ 사람/유지자 시트 감사 ──▶ 실질 결함만 이슈 승격 (F05)
        자동 diff% 통과는 최종 판정이 아니다
```

질문이 이미 답이면 다음 단으로 내려가지 않는다. 정지 조건은
[27_exception_catalog.md](references/27_exception_catalog.md) 와 아래 표.

## 요청 → 명령

| 사용자 요청 | 명령 | 레퍼런스 |
| --- | --- | --- |
| 한컴 PDF 랑 같은지 | 등록키 또는 `--source`+`--reference-pdf`+`--label` | 01_when_to_use.md |
| 일단 글자만 | `--text-only --export-all-svg` | 06_text_report.md |
| 최악 쪽부터 보자 | 기본 모드 → `report.tsv` 내림차순 | 05_pixel_ranking.md |
| 글자가 □ 로 나온다 | `--font-style` 기본인지, `RHWP_FONT_PATH_DIR` | 07·09·17 |
| 윈도에서 돌려 | `venv\Scripts\python.exe`, break-system-packages 금지 | 03_windows.md |
| 임의 HWP+PDF 쌍 | `--source --reference-pdf --label --reference-grade` | 19_direct_pair.md |
| Chrome 없다 | `--text-only` 또는 정지 F10 | 13_missing_chrome.md |
| 암호화 PDF | 정지 F13. 암호 우회 CLI 발명 금지 | 16_encrypted_pdf.md |

등록 키: `plan` `manual` `bunjang` `korexam` `math` `eng`.
키는 ASCII 글롭이다. 한글 argv 는 배경 셸에서 cp949 로 깨질 수 있다.

```bash
# 등록 fixture
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --out-dir /tmp/rhwp-fidelity-plan

# 임의 쌍, 텍스트만
RHWP_BIN=target/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 214 \
  --source 'samples/입력.hwp' \
  --reference-pdf 'pdf/한컴-기준.pdf' \
  --label issue-3738-hwp \
  --reference-grade '한컴 2020 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-issue-3738
```

Windows:

```powershell
venv\Scripts\python.exe tools\fidelity_compare\fidelity_compare.py plan 0 9 `
  --out-dir $env:TEMP\rhwp-fidelity-plan
```

`--out-dir` 은 지정한 디렉터리 자체를 산출 루트로 쓴다. 생략하면
`output/fidelity/<키>/`. worktree 를 깨끗이 두려면 저장소 밖 경로를 쓴다.

## 산출물 (읽는 법)

| 파일 | 의미 | 판정 권위 |
| --- | --- | --- |
| `cmp-pNNN.png` | 기준 PDF ‖ rhwp 렌더 쪽별 비교 시트 | 사람 눈 |
| `report.tsv` | `page`, `diff%`, `note` — **최악 쪽 우선** 정렬 | 후보 순위 |
| `text-report.tsv` | 쪽별 문자 멀티셋. `reference_only`=소실, `svg_only`=과잉, 둘 다=치환 | 후보 |
| `provenance.tsv` | 원본·기준 PDF 경로와 기준 등급 | 재현 기록 |
| `run-state.tsv` | requested/completed/missing. 누락이면 종료 코드 ≠ 0 | 실행 완전성 |
| `page-count-ledger.tsv` | PDF / SVG / render-tree 쪽수 | 후보. 전역 page-break 수정 근거 아님 |
| `svg-glyph-risk-report.tsv` | SVG raw PUA / U+FFFD | 두부 후보 (PDF 추출과 독립) |

그 외 owner-shift · table-fragment · layout-candidates 는
[20_outputs.md](references/20_outputs.md). 모두 candidate 다.

`text-report.tsv` 헤더:

```
page	reference_only	svg_only	reference_only_chars	svg_only_chars	note
```

공백·순서를 무시하고 NFC 멀티셋을 비교한다. PDF 텍스트층에 없는 path
글리프, 숨김 텍스트, 추출기 매핑 차이는 최종 시각 판정을 대신하지 않는다.

## 글꼴·두부

하네스는 `rhwp export-svg --font-style` 을 기본으로 쓴다. 원 문서 legacy
face 와 설치된 family/full name 이 달라도 `@font-face src: local(...)`
별칭을 Chrome 이 쓰게 해서, 한양중고딕·휴먼명조가 설치돼 있는데도 비교 PNG 가
□ 로 채워지는 **하네스 오염** 을 막는다.

- 글꼴 바이너리를 SVG 에 embed 하지 않는다 (기본 모드)
- `RHWP_FONT_PATH_DIR` 로 라이선스 있는 로컬 글꼴 디렉터리를 넘긴다
- 휴먼명조/휴먼고딕의 HMKMM/HMKMG 처럼 EBDT local face 가 Chrome 에서
  `.notdef` 를 그리는 경우, 고정 좌표를 유지한 채 outline 명조·고딕을 먼저
  고른다. HY신명조는 원 face 우선순위를 보존한다
- 시트가 온통 두부면 **문서 결함이 아니라 하네스 오염** (F14). 글꼴을 고치고
  다시 돌린 뒤에야 후보를 읽는다

## provenance (필수 기록)

한컴 출력은 도구·버전·출력 경로·폰트에 따라 달라진다. 재현 기록에 남긴다.

| 필드 | 예 |
| --- | --- |
| 한컴 도구 | 한글 2022 / 2024 / 한컴오피스 |
| 버전 | 12.0.0.xxxx |
| 내보내기 경로 | 파일→PDF로 저장 / 인쇄→PDF / `pdf/` 보존본 |
| 글꼴 | 설치 목록 또는 `RHWP_FONT_PATH_DIR` |
| 원본 경로 | `samples/....hwp` |
| 오라클 PDF 경로 | `pdf/....-2022.pdf` |
| 기준 등급 | 한컴 2022 기준 PDF / 참고 PDF (미확인) |

`provenance.tsv` 는 `role`, `path`, `grade` 세 열이다. `--reference-grade` 는
direct pair 에서만 쓴다. `bunjang` 동반 PDF 는 등급이 "참고" 이며 별도 확인
전에는 최종 기준으로 쓰지 않는다.

## 정지 규칙

| ID | 언제 | 행동 |
| --- | --- | --- |
| F01 | 독립 한컴 PDF 없음 | `rhwp-visual-regression` 으로 인계. 가짜 기준 PDF 를 만들지 않음 |
| F02 | `--text-only` 로 후보만 모음 | 시트 없이 원장만. 결함 확정 금지 |
| F03 | `report.tsv` 상위 쪽 | 최악부터 시트를 눈으로 감사. diff% 절대값으로 merge 하지 않음 |
| F04 | PUA / U+FFFD / □ | 글꼴 별칭·경로를 먼저. 문서 결함으로 바로 승격 금지 |
| F05 | 시트 감사 끝 | 실질 결함만 이슈. 최종 판정은 유지자 |
| F06 | gym / 새 CLI 요청 | 거절. 이 스킬은 기존 하네스만 |
| F07 | visual-regression 스킬을 여기서 고침 | 거절. 이웃 스킬 재작성 금지 |
| F08 | bug-hunter 여정을 여기서 다시 씀 | 거절. 후보만 넘긴다 |
| F09 | venv 없음 / 시스템 pip | 정지. `--break-system-packages` 금지 (F15) |
| F10 | Chrome 없음, `--text-only` 아님 | `--text-only` 로 내리거나 정지. Chrome 설치를 강제 스크립트하지 않음 |
| F11 | 쪽수 불일치 | `page-count-ledger.tsv` 를 후보로 기록. 전역 page-break 패치 금지 |
| F12 | `run-state` incomplete | 누락 쪽을 먼저. 부분 랭킹을 전수로 포장하지 않음 |
| F13 | 암호화 PDF | 정지. 암호 제거 CLI / 우회 발명 금지 |
| F14 | 시트가 두부 가득 | 하네스 오염. 글꼴을 고치고 재실행 |
| F15 | `--break-system-packages` | 거절. 저장소 `venv/` 만 |
| F16 | 질문이 이미 답 | 다음 단 금지 |
| F17 | `samples/` 동반 PDF 를 공식 기준으로 승격 | 도구·버전 확인 전 금지 |
| F18 | 원본 HWP 를 비교 중 덮어씀 | 금지. `--out-dir` 만 기록 |

**금지 기본값**

- 새 비교 CLI (`fidelity-diff`, `pdf-compare`, `hangul-diff`) 발명
- gym pack / gym 과제 / 채점기 작성
- `rhwp-visual-regression` / `bug-hunter` 스킬 재작성
- DocumentCore / 렌더러 구현을 이 PR 에서 고침
- 시스템 Python 에 pip install, `--break-system-packages`
- 암호화 PDF 암호 우회
- diff% 0 을 "한컴과 동일" 으로 발표
- 두부 시트를 문서 회귀로 오진
- 원본을 비교 과정에서 덮어쓰기

## 인계

- 공식 PDF 없이 전후 레이아웃만 → `rhwp-visual-regression`
- 원인 미확정 실사용 여정 → `bug-hunter` (이 하네스 원장을 입력으로)
- 미지 문서 파악만 → `rhwp-doc-triage` (비교하지 않음)
- 편집 자체 → `rhwp-safe-edit` / `rhwp-form-fill` (끝난 뒤 공식 PDF 가 있으면 여기로)
- CLI 일반 → `rhwp-cli` (`export-svg` 단건)

상세: [26_handoff.md](references/26_handoff.md)

## 예외 경로 (짧게)

| 예외 | 증상 | 처방 |
| --- | --- | --- |
| Chrome 없음 | `Chrome/Chromium을 찾을 수 없습니다` exit 2 | `CHROME_BIN` 또는 `--text-only` (F10) |
| venv 없음 | `pypdf`/`pypdfium2` ImportError exit 2 | 저장소 `venv` 재생성 (F09) |
| 쪽수 불일치 | ledger 의 PDF≠SVG | 후보. 전역 패치 금지 (F11) |
| 암호화 PDF | pypdf/pypdfium2 암호 예외 | 정지 (F13) |
| 두부 하네스 | 비교 PNG 가 □ 투성이 | `RHWP_FONT_PATH_DIR` + `--font-style` 재실행 (F14) |

## 레퍼런스 목차

1. [00_tree.md](references/00_tree.md) — 판단 트리
2. [01_when_to_use.md](references/01_when_to_use.md) — 독립 PDF 가 있을 때만
3. [02_setup_venv.md](references/02_setup_venv.md) — venv · pypdf · pillow
4. [03_windows.md](references/03_windows.md) — `venv\Scripts\python.exe`
5. [04_page_sheets.md](references/04_page_sheets.md) — `cmp-pNNN.png`
6. [05_pixel_ranking.md](references/05_pixel_ranking.md) — 최악 쪽 우선
7. [06_text_report.md](references/06_text_report.md) — 소실/과잉/치환
8. [07_font_style.md](references/07_font_style.md) — `--font-style`
9. [08_local_face_aliases.md](references/08_local_face_aliases.md) — local() 별칭
10. [09_tofu.md](references/09_tofu.md) — PUA · U+FFFD · □
11. [10_font_path_dir.md](references/10_font_path_dir.md) — `RHWP_FONT_PATH_DIR`
12. [11_provenance.md](references/11_provenance.md) — 도구·버전·경로·글꼴
13. [12_visual_verdict.md](references/12_visual_verdict.md) — 유지자 판정
14. [13_missing_chrome.md](references/13_missing_chrome.md) — Chrome 부재
15. [14_missing_venv.md](references/14_missing_venv.md) — venv 부재
16. [15_page_count_mismatch.md](references/15_page_count_mismatch.md) — 쪽수
17. [16_encrypted_pdf.md](references/16_encrypted_pdf.md) — 암호화 PDF
18. [17_tofu_harness.md](references/17_tofu_harness.md) — 하네스 두부 오염
19. [18_registered_keys.md](references/18_registered_keys.md) — plan/manual/…
20. [19_direct_pair.md](references/19_direct_pair.md) — `--source` 쌍
21. [20_outputs.md](references/20_outputs.md) — TSV/시트 카탈로그
22. [21_vs_visual_regression.md](references/21_vs_visual_regression.md) — 축 분리
23. [22_vs_bug_hunter.md](references/22_vs_bug_hunter.md) — 여정과 다름
24. [23_journeys.md](references/23_journeys.md) — 실사용 여정
25. [24_pitfalls.md](references/24_pitfalls.md) — 함정
26. [25_worked_traces.md](references/25_worked_traces.md) — 재현 트레이스
27. [26_handoff.md](references/26_handoff.md) — 이웃 스킬
28. [27_exception_catalog.md](references/27_exception_catalog.md) — 예외 카탈로그

예제: [examples/](examples/). 기계 가독 픽스처: [fixtures/](fixtures/).
처리 결과: [`mydocs/working/agent_fidelity_compare.md`](../../../mydocs/working/archives/agent_fidelity_compare.md)

## 권위

- [`tools/fidelity_compare/README.md`](../../../tools/fidelity_compare/README.md)
- [`mydocs/manual/verification/visual_verification_governance.md`](../../../mydocs/manual/verification/visual_verification_governance.md)
- [`mydocs/manual/verification/hangul_pdf_baseline.md`](../../../mydocs/manual/verification/hangul_pdf_baseline.md)
- `rhwp export-svg` — 이미 있는 명령. 이 스킬이 발명하지 않음
- 이슈: [#5329](https://github.com/edwardkim/rhwp/issues/5329)
