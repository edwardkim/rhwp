---
name: rhwp-exam-ingest
description: 'PDF·이미지·Markdown·DOCX 시험문제 자료를 HWPX 시험지로 변환한다. 문제·지문·선택지와 이미지 위치를 읽고, 제공된 helper와 ingest 스키마로 파일을 조립할 때 사용한다. "HWPX로 만들어줘", "한글 시험지로 변환", "exam ingest", "/rhwp-exam-ingest", "시험문제 변환" 요청에 적용한다.'
---

# rhwp-exam-ingest — 시험지 → HWPX 변환 Skill

사용자가 PDF/이미지/MD/DOCX 로 준 시험문제를 한컴에서 편집 가능한 HWPX 로 만든다.
이 스킬은 **실 에이전트 경로**다. gym 이 아니고, 새 CLI 를 발명하지 않으며,
`DocumentCore` 의 `exam_paper` 빌더 로직을 이 작업에서 바꾸지 않는다.

`build-from-ingest`는 로컬 PNG·JPEG·GIF·BMP·TIFF를 검증해 Picture/BinData로 삽입한다.
기준 경로는 JSON 디렉터리이며 `--media-dir`로 바꿀 수 있다. 경로 이탈·누락·손상·한도 초과는
`INGEST_IMAGE_*`로 실패한다. placeholder로 대체하지 않으며 실제 이미지 표시도 확인한다.
문장 중간의 위치는 `paragraph.runs`의 text/image 순서로 지정한다.

텍스트 시험지 조립과 입력 자료 준비에 사용할 구성은 다음과 같다.

- 입력 정규화: `helpers/pdf_to_pngs.sh` · `helpers/extract_docx.py` · 이미지 패스스루 · MD `![alt](path)`
- 중간 표현: `tools/rhwp-ingest/schema/ingest_schema_v1.json` (`version: "1"`)
- 자르기: `helpers/crop_image.sh` (bbox 픽셀 계약)
- 조립: `rhwp build-from-ingest <ingest.json> --media-dir <dir> -o <out.hwpx>`
- 의존성: `helpers/check_deps.sh [--json]`

입력 종류에 맞는 helper를 실행하고 스키마 필드를 채운다.
실패 응답에 따라 고칠 입력과 더 진행할 수 없는 범위를 구분한다.

상세는 `references/` 를 단계별로 연다. SKILL.md 는 인덱스와 정지 규칙만 담는다.

## 작동 원리

```
[입력: PDF/PNG/JPG/MD/DOCX]
        │
        ▼ helpers/ 로 입력 정규화
        │  (PDF→페이지 PNG, DOCX→텍스트+이미지, MD→텍스트+이미지 ref, IMG→그대로)
        │
        ▼ Read tool 로 페이지 PNG 1장씩 Vision 분석
        │  문제 번호 · 지문 · 선택지 · bbox · placement
        │
        ▼ Write 로 ingest.json (ingest_schema_v1, deny_unknown_fields)
        │
        ▼ helpers/crop_image.sh 로 bbox crop
        │
        ▼ rhwp build-from-ingest --media-dir -o
        │
        ▼ rhwp dump / export-text / unzip -l 로 게이트
        │
        ▼ [출력: out.hwpx]
```

현재 세션의 이미지 읽기 기능으로 시험지를 확인한다. 이 기능이 없으면 그 한계를
알리고 이미지 내용을 추측하지 않는다. 외부 OCR(PaddleOCR, Tesseract)이나
별도 Anthropic API 호출은 이 절차에 포함하지 않는다.

- 한국어 시험지 layout 을 자연어로 이해 (지문 vs 선택지)
- 이미지 의미 배치 (`between` / `above` / `below` / `inline`)
- helper 실행에는 별도 모델 API 키나 OCR 서비스가 필요하지 않다.
  현재 에이전트의 이미지 분석에 필요한 네트워크는 실행 환경에 따라 다르다.
- 사용자 환경에 OCR 모델 설치 불필요

## 사다리 (강제 순회 아님)

`check_deps → 입력 정규화 → Vision → ingest.json → crop → build-from-ingest → dump/export-text`

질문이 이미 답이면 다음 단으로 내려가지 않는다. 각 단의 정지 조건은
[14_failure_envelopes.md](references/14_failure_envelopes.md) 와 아래 정지 표.

```
check_deps.sh [--json]
  ├─ DEP_MISS_RHWP ──▶ 중단 (F01)
  ├─ DEP_MISS_IMAGEMAGICK ──▶ crop 불가. 텍스트만이면 진행 가능 (F02)
  └─ poppler/python-docx 누락은 입력 종류에 따라 (F03/F04)
       │
       ▼ 입력 종류
            ├─ PDF ──▶ pdf_to_pngs.sh (F05)
            ├─ PNG/JPG ──▶ 패스스루. crop 만 (F06)
            ├─ MD ──▶ 본문 + ![alt](path) (F07)
            └─ DOCX ──▶ extract_docx.py (F08)
                 │
                 ▼ Vision / 구조 인식
                      ├─ 페이지 30+ 문항 빽빽 ──▶ 사분면 분할 (F09)
                      ├─ 스캔 흐림 ──▶ 원본 재요청 (F10)
                      └─ ingest.json 작성
                           ├─ deny_unknown_fields 거부 ──▶ 오타 수정 (F11)
                           ├─ boxed 에 text 필드 ──▶ blocks[] 로 (F12)
                           ├─ stem 에 번호 이미 있음 ──▶ auto_number:false (F13)
                           └─ crop_image.sh
                                ├─ bbox 비정수/0 ──▶ exit 4 (F14)
                                └─ build-from-ingest --media-dir -o (F15)
```

## 요청 → 명령

| 사용자 요청 | 무엇 | 레퍼런스 |
| --- | --- | --- |
| 이 PDF 를 HWPX 로 | `pdf_to_pngs.sh` → Vision → ingest → `build-from-ingest` | 02_pdf_to_pngs.md |
| 이 스캔 PNG 한 장 | 패스스루 → Vision → crop → build | 04_image_passthrough.md |
| 이 MD + 그림 | `![alt](path)` 를 media 로 | 05_md_image_refs.md |
| 이 DOCX 시험지 | `extract_docx.py` | 03_extract_docx.md |
| 공유 지문 [1~3] | `passages[]` + `passage_ref` | 07_passages_questions.md |
| `<보기>` 박스 | `stem_blocks` `type:boxed` | 08_stem_blocks_boxed.md |
| 그림이 지문과 선택지 사이 | `placement: between` | 09_media_placement.md |
| 번호가 지문에 이미 있음 | `auto_number: false` | 10_auto_number.md |
| 그래프만 잘라서 넣어 | `crop_image.sh x y w h` | 11_crop_bbox.md |
| 의존성 있니? | `check_deps.sh --json` | 13_check_deps.md |
| 수식/표가 많아 | 이미지로 캡처. 한계 고지 | 15_known_limits.md |

살아 있는 동사는 `pdf_to_pngs.sh` → `extract_docx.py` → `crop_image.sh` →
`rhwp build-from-ingest` → `rhwp dump` / `rhwp export-text` 뿐이다.
`exam-from-pdf`, `ingest-exam`, `hwp_doc_exam` 같은 명령은 **없다**.

## 정지 규칙

| ID | 언제 | 행동 |
| --- | --- | --- |
| F01 | `DEP_MISS_RHWP` | 중단. `cargo build --release` 안내 |
| F02 | `DEP_MISS_IMAGEMAGICK` 그리고 media 가 필요 | 중단. 텍스트만이면 crop 생략 후 진행 |
| F03 | PDF 입력인데 pdftoppm·magick 둘 다 없음 | `PDF_MISS_TOOLS` / `DEP_MISS_POPPLER`. poppler 설치 안내 |
| F04 | DOCX 입력인데 python3 없음 | `DEP_MISS_PYTHON3`. `DEP_MISS_PYTHON_DOCX` 만이면 fallback |
| F05 | `pdf_to_pngs.sh` 입력 파일 없음 | `PDF_SRC_MISSING` exit 1 |
| F06 | 이미지가 한 페이지 분량 | 패스스루. 분할하지 않는다 |
| F07 | MD 의 `![alt](path)` 가 깨진 경로 | 사용자에게 경로 확인. 추측으로 media id 만들지 않음 |
| F08 | `extract_docx.py` 입력 없음 | `DOCX_SRC_MISSING` exit 1 |
| F09 | 한 페이지 30+ 문항 | 절반/사분면으로 나눠 Vision. 한 장에 몰아 추측 금지 |
| F10 | 스캔이 흐려 번호가 안 읽힘 | 선명한 원본 또는 PDF 재요청 |
| F11 | `build-from-ingest` 가 unknown field 거부 | 스키마 필드만 사용. 새 키 발명 금지 |
| F12 | boxed 블록에 `text` 를 직접 넣음 | #3358. `blocks:[{type:text,text:...}]` |
| F13 | stem 첫 텍스트가 `"2. …"` 로 시작 | `auto_number: false`. 아니면 `"2. 2. …"` |
| F14 | bbox 가 소수·음수·0 | `CROP_BBOX_*` exit 4. 다시 추정 |
| F15 | `-o` 누락 | CLI 가 사용법 오류. `-o` 는 필수 |
| F16 | 편집 가능한 수식 요청 | ingest에 수식 필드 없음. 동의하면 수식 crop을 이미지로 삽입 |
| F17 | 편집 가능한 표 요청 | ingest에 표 필드 없음. 동의하면 표 crop을 이미지로 삽입 |
| F18 | `INGEST_IMAGE_*` 입력 오류 | 경로·형식·실제 치수·자원 한도를 확인. 누락된 그림을 성공으로 보고하지 않음 |
| F19 | 사용자 질문이 이미 답변 가능 | 다음 단으로 내려가지 않는다 |

**금지 기본값**

- 새 CLI (`exam-from-pdf`, `ingest-exam`, `build-exam`) 발명
- `src/document_core/builders/exam_paper.rs` 수정
- gym pack / gym 과제 작성
- 외부 OCR 엔진 호출
- `deny_unknown_fields` 를 피하려고 스키마에 없는 키를 넣기
- `auto_number` 기본을 무시하고 stem 에 `"1. "` 를 중복
- 원본 PDF 를 `--in-place` 로 덮어쓰기 (이 명령은 애초에 원본을 쓰지 않음)
- 이 스킬 안에서 onboarding / form-fill / table-exchange / safe-edit 를 재작성

## 입력 형식별 한 줄

### PDF

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh <input.pdf> <out_dir>
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh --json --dry-run <input.pdf> <out_dir>
```

`out_dir/page_001.png` … . 텍스트 layer 가 있으면 `pdftotext` 가 `text.txt` 를 보조로 남긴다.
상세: [02_pdf_to_pngs.md](references/02_pdf_to_pngs.md)

### 이미지 (PNG/JPG)

Read tool 로 직접 본다. 한 페이지 분량은 그대로. crop 만 `crop_image.sh`.
상세: [04_image_passthrough.md](references/04_image_passthrough.md)

### Markdown

파일을 읽고 `![alt](path)` 를 `media[].id` 로 옮긴다. `# 1.` / `## 1.` 로 문제 번호를 추론한다.
상세: [05_md_image_refs.md](references/05_md_image_refs.md)

### DOCX

```bash
python3 .claude/skills/rhwp-exam-ingest/helpers/extract_docx.py <input.docx> <out_dir>
```

`out_dir/text.txt` + `out_dir/img/*`. python-docx 없으면 zip 정규식 fallback.
상세: [03_extract_docx.md](references/03_extract_docx.md)

## ingest.json 핵심

스키마: `tools/rhwp-ingest/schema/ingest_schema_v1.json`.
샘플: `sample_minimal.json` · `sample_structured.json`.
Rust 모델: `src/parser/ingest/schema.rs` (`#[serde(deny_unknown_fields)]`).

```jsonc
{
  "version": "1",
  "page_size": {"width_mm": 210, "height_mm": 297},
  "default_font": "함초롬바탕",
  "header_text": "국어 영역",
  "footer_text": "1/20",
  "form_label": "홀수형",
  "passages": [
    {
      "id": "p1-3",
      "blocks": [
        {"type": "text", "text": "[1~3] 다음 글을 읽고 물음에 답하시오."},
        {"type": "text", "text": "긴 공유 지문..."}
      ]
    }
  ],
  "questions": [
    {
      "number": 1,
      "passage_ref": "p1-3",
      "stem": "다음 글의 주제로 가장 적절한 것은?",
      "auto_number": true,
      "stem_blocks": [
        {"type": "text", "text": "다음 글의 주제로 가장 적절한 것은?"},
        {
          "type": "boxed",
          "title": "<보기>",
          "blocks": [{"type": "text", "text": "보기 본문..."}]
        },
        {"type": "image", "ref": "img/q1_passage.png", "placement": "between"}
      ],
      "choices": [
        {"label": "①", "text": "선택지 1"},
        {"label": "②", "text": "선택지 2"},
        {"label": "③", "text": "선택지 3"},
        {"label": "④", "text": "선택지 4"},
        {"label": "⑤", "text": "선택지 5"}
      ],
      "media": [
        {"id": "img/q1_passage.png", "natural_w": 800, "natural_h": 600,
         "target_w_mm": 80, "placement": "between"}
      ]
    }
  ]
}
```

`media[].id` 는 `--media-dir` 기준 상대 경로.

### auto_number

빌더는 첫 stem 텍스트 앞에 `{number}. ` 를 붙인다. stem 에 번호를 이미 썼으면
`auto_number: false`. 공유 지문 지시문 `[1~3] …` 은 `passages[]` 에 둔다.
상세: [10_auto_number.md](references/10_auto_number.md)

### placement

`between` (기본, 지문↔선택지) · `above` (지문 위) · `below` (선택지 다음) · `inline`.
상세: [09_media_placement.md](references/09_media_placement.md)

## crop · build

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/crop_image.sh \
    "$TMP/page_001.png" "<x>" "<y>" "<w>" "<h>" "$MEDIA_DIR/img/q1_passage.png"

rhwp build-from-ingest "$TMP/ingest.json" --media-dir "$MEDIA_DIR" -o "$OUT_HWPX"
```

bbox 는 픽셀, 좌상단, 10진 정수, `w>=1`, `h>=1`. dry-run:

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/crop_image.sh \
    --json --dry-run "$TMP/page_001.png" 120 400 640 360 "$MEDIA_DIR/img/q1.png"
```

`-o`는 필수다. `--media-dir`을 지정하면 그 디렉터리 안의 이미지 파일을 검증해 삽입한다.
생략하면 JSON 디렉터리를 기준으로 읽는다. `INGEST_IMAGE_*` 오류가 발생하면 F18에 따라
입력을 보정하고 다시 실행한다. 그림이 필수인 입력을 텍스트만으로 완료 처리하지 않는다.
상세: [11_crop_bbox.md](references/11_crop_bbox.md) ·
[12_build_from_ingest.md](references/12_build_from_ingest.md)

## 검증

```bash
rhwp dump <out.hwpx>          # IR 구조
rhwp export-text <out.hwpx> -o <dir>   # 지문/선택지 텍스트 대조
unzip -l <out.hwpx>           # BinData/ 이미지 포함 여부
```

한컴오피스 2024 또는 LibreOffice + hwpx 플러그인으로 시각 확인 권장.
원본 PDF 와 픽셀 일치는 이 스킬의 게이트가 아니다.

## 알려진 한계

- **이미지 입력**: PNG·JPEG·GIF·BMP·TIFF를 실제 삽입한다. 파일 경로·치수·한도를
  검증하며 INGEST_IMAGE_* 오류를 placeholder로 대체하지 않는다. CLI 종료 코드 0이나
  텍스트 대조만으로 완료 처리하지 않고 Picture/BinData와 실제 표시를 확인한다.
- **수식**: 이 ingest 스키마에는 편집 가능한 수식 필드가 없다.
  동의하면 수식 crop을 이미지로 삽입한다. 편집 가능한 수식으로 보고하지 않는다.
- **표/정밀 박스**: 이 ingest 스키마에는 편집 가능한 표·프레임 필드가 없다.
  동의하면 표 crop을 이미지로 삽입한다. 편집 가능한 표로 보고하지 않는다.

상세: [15_known_limits.md](references/15_known_limits.md)

## 의존성

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/check_deps.sh --json
```

| 도구 | 언제 | 없으면 |
| --- | --- | --- |
| `rhwp` | 항상 | F01. 빌드 안내 |
| `magick`/`convert` | crop, PDF fallback | F02. `DEP_MISS_IMAGEMAGICK` |
| `pdftoppm` | PDF 권장 | magick fallback. 둘 다 없으면 F03 |
| `pdftotext` | PDF 보조 | Vision 만으로 진행 |
| `python3` | DOCX | F04 |
| `python-docx` | DOCX 정밀 | zip 정규식 fallback. 실패 아님 |

실패 봉투: [13_check_deps.md](references/13_check_deps.md)

## 인계

- 누름틀 서식 채우기 → `rhwp-form-fill` (이 스킬을 재작성하지 않음)
- 표 CSV 왕복 → `rhwp-table-exchange`
- 배포 전 점검 → `rhwp-security-sweep`
- 미지 문서 파악만 → `rhwp-doc-triage` (읽기, 시험지를 만들지 않음)

이 스킬은 **새 시험지를 생성**한다. 기존 HWP 를 편집하지 않는다.

## 레퍼런스 목차

1. [00_tree.md](references/00_tree.md) — 판단 트리
2. [01_input_normalize.md](references/01_input_normalize.md) — 입력 정규화
3. [02_pdf_to_pngs.md](references/02_pdf_to_pngs.md) — PDF → PNG
4. [03_extract_docx.md](references/03_extract_docx.md) — DOCX 추출
5. [04_image_passthrough.md](references/04_image_passthrough.md) — 이미지 패스스루
6. [05_md_image_refs.md](references/05_md_image_refs.md) — MD 이미지 ref
7. [06_ingest_schema_v1.md](references/06_ingest_schema_v1.md) — 스키마
8. [07_passages_questions.md](references/07_passages_questions.md) — 지문·문항
9. [08_stem_blocks_boxed.md](references/08_stem_blocks_boxed.md) — stem_blocks·보기
10. [09_media_placement.md](references/09_media_placement.md) — 배치
11. [10_auto_number.md](references/10_auto_number.md) — auto_number 정책
12. [11_crop_bbox.md](references/11_crop_bbox.md) — bbox 계약
13. [12_build_from_ingest.md](references/12_build_from_ingest.md) — 빌드 게이트
14. [13_check_deps.md](references/13_check_deps.md) — 의존성 봉투
15. [14_failure_envelopes.md](references/14_failure_envelopes.md) — 실패 봉투
16. [15_known_limits.md](references/15_known_limits.md) — 한계
17. [16_pitfalls.md](references/16_pitfalls.md) — 함정
18. [17_sample_transcripts.md](references/17_sample_transcripts.md) — 트랜스크립트
19. [18_verify_gate.md](references/18_verify_gate.md) — dump/export-text 게이트
20. [19_intent_matrix.md](references/19_intent_matrix.md) — 발화 → 동작
21. [20_exit_codes.md](references/20_exit_codes.md) — 종료 코드

예제: `examples/`. 기계 가독 픽스처: `fixtures/` · `fixtures/catalog.json`.

## 권위

- [`tools/rhwp-ingest/schema/ingest_schema_v1.json`](../../../tools/rhwp-ingest/schema/ingest_schema_v1.json)
- [`src/parser/ingest/schema.rs`](../../../src/parser/ingest/schema.rs)
- [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md) §`build-from-ingest`
- 처리 결과: [`mydocs/working/agent_exam_ingest.md`](../../../mydocs/working/archives/agent_exam_ingest.md)

관련 이슈: #5319 (본 스킬 고도화), #660 (스키마+빌더), #667 (passages/boxed),
#3358 (deny_unknown_fields), #182 (Picture 직렬화 구현의 과거 작업).

[근거] `src/serializer/hwpx/picture.rs`, `src/parser/ingest/schema.rs`,
`src/cli/commands/generation.rs`, `src/document_core/builders/exam_paper.rs` ·
2026-11-02, upstream/devel `0b697f90c3637558196b9e3b81c4eb59049432af` 확인 · High.
소스 확인은 실제 한컴 표시 검증을 대신하지 않는다.
