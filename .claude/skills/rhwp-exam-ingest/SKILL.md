---
name: rhwp-exam-ingest
description: PDF/이미지/MD/DOCX 형태의 시험문제 자료를 HWPX 시험지 파일로 변환합니다. Claude가 시험지를 직접 읽어(Vision) 문제 구조(지문/선택지/이미지)를 인식하고, 이미지를 정확한 bbox로 잘라 "지문과 선택지 사이" 또는 "지문 아래"에 배치한 HWPX를 생성합니다. 트리거: 사용자가 시험지 파일을 주며 "HWPX로 만들어줘", "한글 시험지로 변환", "exam ingest", "/rhwp-exam-ingest", "시험문제 변환".
---

# rhwp-exam-ingest — 시험지 → HWPX 변환 Skill

## 목적

사용자가 PDF/이미지/MD/DOCX 형태로 제공하는 시험문제 자료(수능, 모의고사, EBS, 학원 자체 시험지, 자체 출제 자료 등)를 한컴오피스에서 편집 가능한 HWPX 파일로 변환합니다. **OCR/Vision/레이아웃 분석은 Claude 본인의 멀티모달 능력으로 직접 수행**하며, Anthropic API 별도 호출은 하지 않습니다.

## 작동 원리

```
[입력: PDF/PNG/JPG/MD/DOCX]
        │
        ▼ helpers/ 스크립트로 입력 정규화
        │  (PDF→페이지 PNG, DOCX→텍스트+이미지 추출, MD→직접, IMG→그대로)
        │
        ▼ Claude가 Read tool로 페이지 PNG 1장씩 검토
        │  → 시험문제 구조 인식 (자연어 분석):
        │     - 문제 번호 ("1.", "2.", ...)
        │     - 지문 (stem) — 주관식 문장 또는 글
        │     - 선택지 ① ② ③ ④ ⑤
        │     - 이미지/도형/그래프 영역 (bbox 좌표 포함)
        │     - 이미지의 의미적 위치 (지문과 선택지 사이? 지문 아래? 인라인?)
        │
        ▼ Write tool로 ingest.json 작성 (ingest_schema_v1 준수)
        │
        ▼ Bash로 helpers/crop_image.sh 호출
        │  → bbox로 PNG 자르기
        │
        ▼ Bash로 rhwp build-from-ingest 호출
        │  → HWPX 파일 생성
        │
        ▼ [출력: out.hwpx]
```

**중요**: Claude(이 모델)가 직접 시험지 이미지를 보고 분석합니다. 외부 OCR 엔진(PaddleOCR, Tesseract 등)을 호출하지 않습니다. 이는 다음 장점을 제공합니다:
- 한국어 시험지 layout을 자연어로 이해 (지문 vs 선택지 의미 분석)
- 이미지의 의미적 배치 위치 결정 (between/above/below/inline)
- 별도 API 키나 네트워크 호출 불필요
- 사용자 환경에 OCR 모델 설치 불필요

## 사용자 트리거 예시

자연어로 호출:
- "이 시험지 PDF를 HWPX로 변환해줘 — samples/2010-exam_kor.pdf"
- "/rhwp-exam-ingest <input> [-o <output>]"
- "수능 영어 모의고사 30문제를 한컴 시험지로 만들고 싶어"

## 입력 형식별 처리

### 1. PDF 시험지

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh <input.pdf> <out_dir>
```

각 페이지를 PNG로 변환 (300 DPI 기본). `out_dir/page_001.png`, `page_002.png`, ...

PDF가 텍스트 layer를 가지면 **Vision 분석 보조**로 사용 (OCR 부담 감소). `pdftotext`로 텍스트 추출 후 Claude의 Vision 분석 결과와 교차 검증.

### 2. 이미지 (PNG/JPG)

직접 Read tool로 시험지 이미지를 보고 분석. 큰 이미지는 사전에 분할이 필요할 수 있으나 보통 한 페이지 분량은 그대로 처리 가능.

### 3. Markdown (MD)

```bash
# MD 파일 직접 읽기 — 텍스트 + 이미지 ref만 있으므로 단순.
```

`![alt](path)` 패턴의 이미지를 media 항목에 추가. 텍스트는 `# 1.`, `## 1.` 등 마크다운 패턴으로 문제 번호 추론 또는 사용자 입력대로 그룹핑.

### 4. DOCX

```bash
python3 .claude/skills/rhwp-exam-ingest/helpers/extract_docx.py <input.docx> <out_dir>
```

`out_dir/text.txt` (본문) + `out_dir/img/*.png` (임베디드 이미지) 출력.

## Claude의 분석 작업 (단계별 instructions)

사용자가 시험지를 제공하면 다음 순서로 작업합니다:

### Step 1: 입력 정규화

사용자 입력에 따라 helpers/ 스크립트를 호출하여 페이지 PNG들과 임베디드 텍스트(있으면)를 임시 디렉토리에 생성합니다.

```bash
TMP=$(mktemp -d /tmp/rhwp-ingest.XXXXXX)
bash .claude/skills/rhwp-exam-ingest/helpers/pdf_to_pngs.sh user_input.pdf "$TMP"
```

### Step 2: Vision 분석 — 페이지별 시험문제 구조 인식

각 `page_NNN.png` 를 Read tool로 읽고 다음을 자연어로 분석한 뒤 자료구조에 정리합니다:

- **문제 번호**: "1.", "2.", "1)", "①" 등을 stem 시작 마커로 인식.
- **지문(stem)**: 문제 번호 다음에 오는 본문. 글 한 단락 또는 지시문 + 본문 ("다음 글의 주제로 가장 적절한 것은?" + 글).
- **선택지(choices)**: ①, ②, ③, ④, ⑤로 시작하는 줄. 보통 5개. label은 그대로 포함.
- **이미지/도형 영역**: 시험지 안에 포함된 그래프/일러스트/표/도형. 각 이미지마다 `bbox=[x, y, w, h]`(픽셀 단위) 추정.
- **이미지의 의미적 placement**:
  - `between`: 지문과 선택지 사이 (가장 흔함 — 그래프/그림 보고 답하는 문제)
  - `below`: 선택지 다음 (드물지만 연관 자료가 뒤에 올 때)
  - `above`: 지문 앞 (지시문 위에 그림이 먼저 오는 경우)
  - `inline`: 지문 텍스트 안에 그림이 끼어들 때 (지시문 일부)

### Step 3: ingest.json 작성

스키마는 `tools/rhwp-ingest/schema/ingest_schema_v1.json` 준수. 샘플은 `tools/rhwp-ingest/schema/sample_minimal.json`.

핵심 구조:
```jsonc
{
  "version": "1",
  "page_size": {"width_mm": 210, "height_mm": 297},
  "default_font": "함초롬바탕",
  "questions": [
    {
      "number": 1,
      "stem": "다음 글의 주제로 가장 적절한 것은?",
      "auto_number": true,
      "stem_blocks": [
        {"type": "text", "text": "다음 글의 주제로 가장 적절한 것은?"},
        {"type": "text", "text": "긴 지문..."},
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

`media[].id`는 `--media-dir` 기준 상대 경로. Claude가 Step 4에서 자르기 결과 PNG를 그 경로에 저장.

#### `auto_number` 필드 사용법 (v2 정책 — Task #660 후속)

빌더는 첫 stem 텍스트 앞에 `{number}. ` prefix를 자동으로 붙인다. 다만 다음 두 경우에는 **사용자가 stem 텍스트 자체에 명시적으로 작성**하는 편이 자연스럽다 — 이 때 `auto_number: false`를 함께 설정해 빌더의 자동 prefix를 끈다.

| 상황 | `auto_number` | stem_blocks 첫 텍스트 예시 |
|------|---------------|----------------------------|
| 일반 문제 (default) | `true` 또는 미지정 | `"다음 글의 주제는?"` → 빌더가 `"1. 다음 글의 주제는?"` 생성 |
| **공유 지문 그룹 지시문** | `false` | `"[1~3] 다음 글을 읽고 물음에 답하시오."` → 그대로 출력 |
| **사용자가 명시적으로 prefix 작성** | `false` | `"2. ㉠에 해당하는 내용으로 가장 적절한 것은?"` → 그대로 출력 |
| **`<보기>` / `[보기]` 본문** | `true` | `"[보기]를 참고하여…"` → 빌더가 `"12. [보기]를 참고하여…"` 생성 |

**권장**: 가능하면 `auto_number: true`(default) + stem 텍스트는 prefix 없이 작성하는 것이 깔끔하다. 공유 지문이나 그룹 지시문 등 빌더의 자동 prefix가 어색한 경우에만 `false`로 끈다.

### Step 4: 이미지 자르기

각 `media[]` 항목에 대해:

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/crop_image.sh \
    "$TMP/page_001.png" \
    "<x>" "<y>" "<w>" "<h>" \
    "$MEDIA_DIR/img/q1_passage.png"
```

bbox 좌표는 Step 2에서 분석한 것을 사용. ImageMagick `magick` 명령어 기반.

### Step 5: HWPX 빌드

```bash
rhwp build-from-ingest "$TMP/ingest.json" --media-dir "$MEDIA_DIR" -o "$OUT_HWPX"
```

성공 시 사용자에게 결과 파일 경로 + 통계(문제 수, 문단 수, 바이트 크기) 보고.

## 한계 및 주의사항

### 현재 알려진 한계

- **이미지 직렬화**: rhwp의 HWPX writer가 Picture inline 직렬화 분기를 #182에서 추가 중입니다. #182 완료 전에는 이미지가 HWPX에 포함되어도 한컴이 표시 못 할 수 있습니다. 현재는 텍스트 위주 변환이 안전.
- **수식**: 복잡 수식(LaTeX 수준)은 이미지로 캡처하여 Picture로 삽입하는 것이 안전합니다. HWP Equation IR 매핑은 후속 마일스톤.
- **표**: 단순 표는 Picture로 캡처. Table IR 빌드는 후속 작업.

### Vision 분석 정확도 향상 팁

- 시험지가 스캔본이라 흐릿하면 사용자에게 더 선명한 원본 또는 PDF 제공을 요청.
- 한 페이지에 30+ 문제가 빽빽하면 페이지를 절반/사분면으로 나누어 분석.
- 문제 번호 매김이 일관되지 않으면 사용자에게 그룹핑 명시 요청.

### 검증

생성된 HWPX는 다음으로 확인:
```bash
rhwp dump <out.hwpx>      # IR 구조 확인
unzip -l <out.hwpx>       # ZIP 구조 확인 (BinData/ 이미지 포함 여부)
```

한컴오피스 2024 또는 LibreOffice + hwpx 플러그인으로 시각 확인 권장.

## 의존성

### 필수
- `rhwp` 바이너리 빌드됨 (`cargo build --release` 후 `target/release/rhwp` 또는 `cargo run --bin rhwp` 사용)

### PDF 입력 시
- `pdftoppm` (poppler-utils) 또는 `magick` (ImageMagick)
- 선택: `pdftotext` (텍스트 layer 추출)

### DOCX 입력 시
- `python3` + `python-docx`: `pip install python-docx`

### 이미지 자르기
- `magick` (ImageMagick 7) 또는 `convert` (ImageMagick 6)

설치 확인 helper:
```bash
bash .claude/skills/rhwp-exam-ingest/helpers/check_deps.sh
```

## 참조

- 전체 계획서: `~/.claude/plans/pdf-md-docx-optimized-neumann.md`
- spike 결과: `mydocs/tech/m100_neumann_spike.md`
- JSON 스키마: `tools/rhwp-ingest/schema/ingest_schema_v1.json`
- 샘플 입력: `tools/rhwp-ingest/schema/sample_minimal.json`
- 빌더 본체: `src/document_core/builders/exam_paper.rs`
- 관련 이슈: #660 (본 작업 1단계 완료), #182 (Picture 직렬화), #654 (사전 검증 spike)
