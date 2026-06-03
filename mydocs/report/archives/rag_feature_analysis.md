# HWP RAG 전처리 요구사항 분석 및 rhwp 대응 방향

**작성일**: 2026-04-13  
**작성자**: Claude Code (RAG 전문 에이전트 분석 기반)  
**참조**: `mydocs/feedback/rag_hwp_strategy.md`

---

## 1. 배경: 데이터 엔지니어가 마주하는 문제

한국 공공기관·기업 환경에서 RAG 파이프라인을 구축하는 데이터 엔지니어는 다음 시나리오를 반복한다:

1. 수백~수천 개의 HWP 문서가 특정 폴더 구조에 쌓여 있다
2. 이를 LlamaIndex / LangChain 등 RAG 프레임워크에 투입할 전처리 파이프라인을 만들어야 한다
3. 기존 도구(PDF 변환, 라마인덱스 기본 HWP 로더)를 사용하면 **표 병합 파괴, 날짜 필드 소실, 책갈피 구조 손상** 등 파싱 단계에서 정보가 훼손된다
4. 훼손된 데이터가 벡터 DB에 적재되면 LLM이 할루시네이션을 일으키고 검색 정확도가 떨어진다 — **GIGO(Garbage In, Garbage Out)**

이 보고서는 데이터 엔지니어가 실제로 겪는 문제를 중심으로 요구사항을 정의하고, rhwp가 그 요구사항을 어떻게 충족해야 하는지를 기술한다.

---

## 2. 데이터 엔지니어 워크플로우 분석

### 2.1 실무 처리 패턴

```
[HWP 문서 폴더]
    /data/docs/2025/
        사업계획서_A기관.hwp
        사업계획서_B기관.hwp
        예산집행_3월.hwp
        ...
         (수백~수천 개)
             ↓  [전처리 파이프라인]
[청크 스트림 (JSON / NDJSON)]
             ↓
[벡터 DB 적재 (Qdrant / Chroma / OpenSearch)]
             ↓
[LLM + 검색 (LlamaIndex / LangChain)]
```

데이터 엔지니어는 **파일 한 개의 내부 구조**보다 **폴더 전체를 어떻게 파이프라인에 연결하는가**에 관심이 있다.

### 2.2 요구사항 도출 기준

각 요구사항은 다음 질문에서 도출되었다:

- 이 기능이 없으면 RAG 결과물 품질(Faithfulness, Retrieval Accuracy)에 직접 영향을 미치는가?
- 데이터 엔지니어가 직접 우회 코드를 작성해야 하는가?
- 기존 오픈소스 HWP 파서로는 해결되지 않는 문제인가?

---

## 3. 핵심 요구사항 정의

### R1. 폴더 단위 배치 처리

**문제**:
데이터 엔지니어는 파일 한 개가 아닌 **폴더 전체**를 처리한다.
현재 rhwp는 파일 한 개 단위 API만 제공하여, 배치 처리를 위해 셸 스크립트나 Python 루프를 직접 작성해야 한다.
이는 오류 처리, 진행 상황 추적, 실패 재시도를 모두 사용자가 구현해야 함을 의미한다.

**요구사항**:
- 폴더 경로 또는 글로브 패턴을 단일 CLI 인자로 지정
- HWP/HWPX 파일 자동 탐색 (하위 폴더 재귀 옵션 포함)
- 파일별 처리 오류 시 해당 파일만 건너뛰고 나머지 계속 처리 (`--on-error skip`)
- 처리 진행 상황 stderr 출력 (stdout은 청크 데이터 전용)

**기대 사용법**:
```bash
rhwp export-chunks ./docs/                      # 폴더 전체
rhwp export-chunks ./docs/*.hwp --format ndjson # 글로브 + NDJSON 스트리밍
rhwp export-chunks ./docs/ --recursive          # 하위 폴더 재귀
rhwp export-chunks ./docs/ -o output/           # 파일별 output/{stem}.json 생성
rhwp export-chunks ./docs/ --on-error skip      # 오류 파일 건너뜀
```

---

### R2. 청크에 원본 파일 출처(source) 포함

**문제**:
RAG 검색 결과를 사용자에게 제시할 때 **"이 답변의 근거 문서는 무엇인가"**를 반드시 명시해야 한다.
현재 청크 출력에 원본 파일 정보가 없으면, 벡터 DB 적재 후 출처 역추적이 불가능하다.
데이터 엔지니어가 파일명을 별도로 주입하는 래퍼 코드를 매번 작성해야 한다.

**요구사항**:
- 모든 청크에 `source` (상대 경로), `source_abs` (절대 경로) 자동 삽입
- 배치 처리 시 NDJSON 스트림에 파일 경계 레코드 삽입

**기대 출력**:
```json
{
  "source": "docs/2025_사업계획서.hwp",
  "source_abs": "/home/user/docs/2025_사업계획서.hwp",
  "section": 0,
  "para": 3,
  "text": "제1장 사업 개요",
  "controls": [],
  "metadata": { "type": "paragraph" }
}
```

배치 NDJSON 스트림:
```ndjson
{"_file_start":"docs/2025_사업계획서.hwp","total_paras":120}
{"source":"docs/2025_사업계획서.hwp","section":0,"para":0,"text":"사업 계획서",...}
...
{"_file_end":"docs/2025_사업계획서.hwp","chunks":87,"errors":0}
{"_file_start":"docs/2026_예산서.hwp","total_paras":240}
...
```

---

### R3. 표 구조의 시맨틱 보존 (OLAP JSON)

**문제**:
공공 HWP 문서의 표는 단순 데이터 격자가 아니라 **병합 셀로 표현된 계층적 의미**를 담는다.
예산표의 항목 헤더, 계획표의 날짜×담당자 교차, 중첩 표 등이 대표적이다.

기존 파서는 이 구조를 파괴한다:
- PDF 변환: "사업명 | 2025년도 | 2026년도" 가 단순 텍스트 행으로 평탄화됨 → 어떤 셀이 어느 열의 헤더인지 소실
- 라마인덱스 기본 로더: 병합 셀 좌표 무시 → 검색 시 "사업명은 무엇인가"에 엉뚱한 셀 값 반환

RAG가 "3월 예산 집행 현황의 AI 사업 항목을 알려줘"라는 질문에 올바르게 답하려면,
셀과 헤더의 관계가 청크에 보존되어야 한다.

**요구사항**:
- `Cell.(row, col, row_span, col_span)` 좌표 보존
- 행/열 헤더 자동 감지 및 `is_header` 플래그
- 중첩 표 재귀 직렬화
- `semantic: "OLAP"` 구분자로 복잡 표와 단순 표 구별

**기대 출력**:
```json
{
  "type": "table",
  "rows": 4, "cols": 3,
  "semantic": "OLAP",
  "cells": [
    { "row": 0, "col": 0, "row_span": 2, "col_span": 1, "text": "사업명", "is_header": true },
    { "row": 0, "col": 1, "row_span": 1, "col_span": 2, "text": "2025년도", "is_header": true },
    { "row": 2, "col": 0, "row_span": 1, "col_span": 1, "text": "AI 기반 문서 분류", "nested_table": null }
  ]
}
```

---

### R4. 날짜 필드·책갈피·교차 참조의 엔티티 메타데이터화

**문제**:
HWP 문서에는 자동 날짜 코드(`FieldType::Date`)와 책갈피(`Control::Bookmark`)가 빈번하게 사용된다.
단순 텍스트 추출 시:
- 날짜 필드 → "yyyy년 MM월 dd일" 형식 문자열로 변환되거나 빈 문자열이 됨
- 책갈피 → 단순 텍스트에서 완전히 소실
- 교차 참조 → 링크 구조 소실

RAG에서 "이 계획서의 사업 종료일은 언제인가"라는 질문에 답하려면, 날짜 필드가 **temporal 메타데이터**로 보존되어야 LlamaIndex `MetadataFilter`에서 날짜 범위 검색이 가능하다.

**요구사항**:
- `FieldType::Date` / `DocDate` → `"temporal"` 메타데이터 (형식 코드 포함)
- `Control::Bookmark` → `"anchor"` 메타데이터 (이름 포함)
- `FieldType::CrossRef` → `"reference"` 메타데이터 (타겟 북마크 연결)
- `FieldType::Hyperlink` → `"link"` 메타데이터

**기대 출력**:
```json
{
  "source": "docs/계획서.hwp",
  "section": 1, "para": 12,
  "text": "사업 종료일: ",
  "controls": [
    {
      "type": "field",
      "field_type": "date",
      "temporal": { "type": "auto_date_field", "format_code": "yyyy년 MM월 dd일" }
    },
    {
      "type": "bookmark",
      "anchor": { "name": "project_end" }
    }
  ],
  "metadata": { "has_temporal": true, "has_anchor": true }
}
```

---

### R5. 재귀 구조 평탄화 옵션 (`--flatten`)

**문제**:
표를 하나의 OLAP JSON 청크로 출력하면(R3) 시맨틱이 풍부하지만,
LlamaIndex의 기본 `SimpleNodeParser`는 JSON 중첩 구조를 처리하지 못한다.
단순 파이프라인에서는 **표의 각 셀을 독립적인 텍스트 청크**로 받고 싶다.

**요구사항**:
- `--flatten` 옵션: 표 셀을 독립 청크로 평탄화
- 각 셀 청크에 표 위치(`para`), 셀 좌표(`row`, `col`), 헤더 여부 포함
- 중첩 표도 재귀적으로 평탄화

**기대 출력** (`--flatten` 적용 시):
```json
{"source":"docs/계획서.hwp","section":0,"para":5,"cell":{"row":0,"col":0},"text":"사업명","metadata":{"type":"table_cell","is_header":true}}
{"source":"docs/계획서.hwp","section":0,"para":5,"cell":{"row":0,"col":1},"text":"2025년도","metadata":{"type":"table_cell","is_header":true}}
{"source":"docs/계획서.hwp","section":0,"para":5,"cell":{"row":2,"col":0},"text":"AI 기반 문서 분류","metadata":{"type":"table_cell","is_header":false}}
```

---

### R7. 수식(Equation) 처리

**문제**:
공공·학술 HWP 문서에는 수식이 빈번하게 등장한다. 수식은 HWP 내부에서 `eqed` 컨트롤로 저장되며,
기존 파서는 이를 완전히 무시하거나 빈 문자열로 처리한다.

RAG에서 "이 계획서의 예산 산정 공식은 무엇인가" 같은 질문에 답하려면
수식이 의미 있는 형태로 청크에 포함되어야 한다.

**rhwp 현황**:
- `Equation.script`: HWP 수식 스크립트 텍스트 파싱 완료 (예: `"1 over {x+y}"`)
- `Equation.font_name`, `font_size`: 수식 폰트 정보 파싱 완료
- MathML / LaTeX 변환: 미구현

**요구사항**:
- 수식 청크에 `script` (HWP 수식 스크립트 원문) 포함 — 즉시 구현 가능
- `"type": "equation"` 메타데이터로 수식 청크 식별 가능하게
- (장기) `script` → LaTeX 변환: HWP 수식 문법을 LaTeX로 변환하여 LLM이 직접 이해 가능하도록

**기대 출력**:
```json
{
  "source": "docs/연구보고서.hwp",
  "section": 2, "para": 34,
  "text": "",
  "controls": [
    {
      "type": "equation",
      "script": "SIGMA_{i=1}^{n} x_i over n",
      "latex": "\\frac{\\sum_{i=1}^{n} x_i}{n}",
      "font_name": "수식",
      "font_size": 1000
    }
  ],
  "metadata": { "type": "paragraph", "has_equation": true }
}
```

> **구현 우선순위**: `script` 원문 포함은 즉시 구현 가능 (High). LaTeX 변환은 HWP 수식 문법 파서가 별도로 필요하여 장기 과제 (Low).

---

### R8. 그림(Picture) 처리

**문제**:
HWP 문서에는 조직도, 차트, 스캔 이미지, 서명 이미지 등이 그림 컨트롤로 삽입된다.
단순 텍스트 추출 시 그림은 완전히 소실되어 RAG가 그림의 내용을 전혀 인식하지 못한다.

특히 공공 문서에서 자주 등장하는 유형:
- **표 대신 그림으로 삽입된 데이터**: 스캔된 표나 캡처 이미지 → 텍스트 추출 불가
- **캡션이 있는 그림**: "그림 3. 사업 추진 체계" → 캡션 텍스트는 RAG에서 활용 가능
- **개체 설명문(대체 텍스트)**: 접근성을 위해 작성된 설명 → RAG에서 직접 활용 가능

**rhwp 현황**:
- `Picture.common.description`: 개체 설명문(대체 텍스트) 파싱 완료
- `Picture.caption`: 캡션 텍스트 파싱 완료
- `Picture.image_attr.bin_data_id` → `BinData.data`: 이미지 바이너리 데이터 접근 가능
- `BinDataContent.extension`: 이미지 포맷(jpg, png, emf 등) 확인 가능
- 이미지 base64 내보내기 API: 미구현

**요구사항**:
- 그림 청크에 `description` (개체 설명문), `caption` (캡션 텍스트) 포함 — 즉시 구현 가능
- `bin_data_id` + 이미지 포맷 참조 정보 포함
- `--embed-images` 옵션: 이미지 바이너리를 base64로 인코딩하여 청크에 포함 (VLM 파이프라인용)
- (장기) VLM 연동 힌트: 이미지 청크에 `"vlm_hint": true` 플래그로 OCR/VLM 후처리 필요 여부 표시

**기대 출력**:
```json
{
  "source": "docs/사업계획서.hwp",
  "section": 0, "para": 10,
  "text": "",
  "controls": [
    {
      "type": "picture",
      "description": "사업 추진 체계도",
      "caption": "그림 1. 사업 추진 체계",
      "image_format": "png",
      "bin_data_id": 3,
      "image_base64": null,
      "vlm_hint": true
    }
  ],
  "metadata": { "type": "paragraph", "has_picture": true }
}
```

`--embed-images` 옵션 사용 시 `"image_base64"` 필드에 base64 인코딩 데이터가 삽입되며,
VLM(GPT-4o Vision, Claude 3 등)에 직접 전달하여 이미지 내용을 텍스트화할 수 있다:

```python
import base64, subprocess, json
from openai import OpenAI

# rhwp에서 이미지 포함 청크 추출
proc = subprocess.Popen(
    ["rhwp", "export-chunks", "doc.hwp", "--format", "ndjson", "--embed-images"],
    stdout=subprocess.PIPE, text=True
)
client = OpenAI()
for line in proc.stdout:
    chunk = json.loads(line)
    for ctrl in chunk.get("controls", []):
        if ctrl["type"] == "picture" and ctrl.get("image_base64"):
            # VLM으로 이미지 내용 텍스트화
            resp = client.chat.completions.create(
                model="gpt-4o",
                messages=[{"role": "user", "content": [
                    {"type": "text", "text": "이 이미지의 내용을 한국어로 설명하세요."},
                    {"type": "image_url", "image_url": {
                        "url": f"data:image/png;base64,{ctrl['image_base64']}"
                    }}
                ]}]
            )
            ctrl["vlm_description"] = resp.choices[0].message.content
```

> **구현 우선순위**: `description` + `caption` 포함은 즉시 구현 가능 (High). `--embed-images` base64 내보내기는 Medium. VLM 연동은 외부 API 의존으로 rhwp 범위 밖 (사용자 구현).

---

### R6. RAG 프레임워크 직접 연동 (Python 인터페이스)

**문제**:
데이터 엔지니어는 Python 환경(LlamaIndex, LangChain, Haystack)에서 작업한다.
rhwp가 CLI만 제공하면, Python에서 `subprocess`를 통해 호출해야 하는 간접 방식이 강요된다.

**rhwp CLI 설치**:
```bash
# crates.io에서 설치 (패키지명: rhwp, 바이너리명: rhwp)
cargo install rhwp

# 소스 빌드
git clone https://github.com/edwardkim/rhwp.git
cd rhwp && cargo install --path .
```

> 참고: `rag_hwp_strategy.md` 등 일부 커뮤니티 문서에 `cargo install rhwp-cli`로 표기된 경우가 있으나,
> 실제 패키지명은 `rhwp`이다. `rhwp-cli`는 존재하지 않는다.

**요구사항 (우선순위 Medium)**:
- NDJSON stdout 스트리밍: Python `subprocess.Popen` + 한 줄씩 읽기 패턴 지원
- WASM `getDocumentChunks()` API: 브라우저/Node.js 환경에서 직접 호출
- (장기) Python 바인딩 (`PyO3`): `pip install rhwp` → `rhwp.iter_chunks("file.hwp")`

**Python 연동 예시 (NDJSON 스트리밍)**:
```python
import subprocess, json
from llama_index.core import Document

proc = subprocess.Popen(
    ["rhwp", "export-chunks", "./docs/", "--format", "ndjson"],
    stdout=subprocess.PIPE, text=True
)
docs = []
for line in proc.stdout:
    chunk = json.loads(line)
    if "_file_start" in chunk or "_file_end" in chunk:
        continue  # 파일 경계 레코드 건너뜀
    if chunk.get("text"):
        docs.append(Document(
            text=chunk["text"],
            metadata={
                "source": chunk["source"],
                "section": chunk["section"],
                "para": chunk["para"],
                "has_temporal": chunk.get("metadata", {}).get("has_temporal", False),
            }
        ))
```

---

## 4. 요구사항 → 이슈 매핑

| 이슈 | 요구사항 | 기능명 | 우선순위 | 예상 공수 |
|------|---------|--------|---------|---------|
| A | R1 + R2 기반 | `export-chunks` CLI — 단일 파일, source 메타 포함 JSON 청크 출력 | **High** | 3~5일 |
| F | R1 + R2 확장 | 배치 처리 — 폴더/글로브 처리, 파일 경계 NDJSON, 오류 건너뜀 | **High** | 2~3일 |
| B | R3 | 표 OLAP JSON 직렬화 — 병합 셀 좌표·행/열 헤더 계층 | **High** | 5~8일 |
| C | R4 | 책갈피·날짜 필드·교차 참조 엔티티 청크 | **High** | 2~3일 |
| G | R7 | 수식 청크 — script 원문 + has_equation 메타 포함 | **High** | 1~2일 |
| H | R8 | 그림 청크 — description·caption 포함, --embed-images base64 옵션 | **High** (description/caption) / Medium (base64) | 2~3일 |
| E | R5 | `--flatten` 옵션 — 표 셀 독립 청크 평탄화 | Medium | 1~2일 |
| D | R6 | WASM `getDocumentChunks()` API | Medium | 1~2일 |

### 구현 의존 관계

```
A (export-chunks 기본 골격, 단일 파일 + source 메타)
  ├─→ F (배치 처리 래핑 — A 위에 폴더/글로브 처리 추가)
  ├─→ B (표 OLAP 확장 — A의 controls 배열에 통합)
  ├─→ C (엔티티 메타데이터 확장 — A의 controls 배열에 통합)
  ├─→ G (수식 청크 — A의 controls 배열에 통합, script 원문 출력)
  ├─→ H (그림 청크 — A의 controls 배열에 통합, description/caption/base64)
  ├─→ E (--flatten 옵션 — B 완료 후 구현)
  └─→ D (WASM getDocumentChunks() 래핑 — A 완료 후 래핑)
```

---

## 5. 경쟁 도구 대비 차별점

| 요구사항 | 라마인덱스 기본 HWP 로더 | PDF 변환 | 한컴 Data Loader | **rhwp (구현 후)** |
|---------|------------------------|---------|-----------------|------------------|
| R1 배치 처리 | ✅ (Python 루프) | ✅ (Python 루프) | ✅ | ✅ 네이티브 지원 |
| R2 source 메타 | ❌ 수동 주입 필요 | ❌ 수동 주입 필요 | ✅ | ✅ 자동 삽입 |
| R3 표 OLAP | ❌ 병합 셀 파괴 | ❌ 병합 셀 파괴 | ✅ (유료) | ✅ **무료 오픈소스 유일** |
| R4 엔티티 메타 | ❌ 소실 | ❌ 소실 | 부분 | ✅ |
| R5 평탄화 옵션 | N/A | N/A | N/A | ✅ |
| R6 NDJSON 스트리밍 | ❌ | ❌ | ❌ | ✅ |
| R7 수식 script 원문 | ❌ 소실 | ❌ 소실 | 부분 | ✅ (script 원문) / 장기 (LaTeX 변환) |
| R8 그림 메타 + base64 | ❌ 소실 | ❌ 소실 (이미지만) | ✅ (유료) | ✅ description/caption + base64 옵션 |

**표 OLAP JSON(이슈 B)**은 무료 오픈소스 HWP 파서 중 rhwp만이 제공 가능한 기능으로,
공공·기업 문서 AI 시장에서의 핵심 차별점이다.

---

## 6. 마일스톤 제안

| 방안 | 내용 | 비고 |
|------|------|------|
| **방안 1**: M100 통합 | 이슈 A, F, B, C를 M100(v1.0.0)에 포함 | 릴리즈 범위 확대 |
| **방안 2**: M110 신설 | RAG 전용 마일스톤 M110(v1.1.0) 신설 (이슈 A~F 전체) | M100 범위 유지, RAG는 차기 릴리즈 |

커뮤니티 요구 강도와 M100 일정을 감안하여 작업지시자가 결정.

---

## 7. 결론

데이터 엔지니어가 HWP RAG 파이프라인을 구축할 때 겪는 핵심 문제는 다섯 가지다:

1. **폴더 단위 배치 처리 부재** → CLI 한 줄로 해결해야 한다 (이슈 A + F)
2. **표 구조 파괴** → 병합 셀 좌표·헤더 계층이 보존된 OLAP JSON이 필요하다 (이슈 B)
3. **엔티티 메타데이터 소실** → 날짜 필드·책갈피가 `MetadataFilter`에서 활용 가능한 형태로 출력되어야 한다 (이슈 C)
4. **수식 소실** → 수식 스크립트 원문이 청크에 포함되어야 LLM이 수식 관련 질의에 응답할 수 있다 (이슈 G)
5. **그림 소실** → 개체 설명문·캡션을 청크에 포함하고, base64 옵션으로 VLM 파이프라인을 지원해야 한다 (이슈 H)

이 다섯 문제를 해결하면 rhwp는 단순 HWP 파서에서
**한국 공공·기업 문서 RAG의 표준 전처리 도구**로 포지셔닝된다.

---

## 승인 요청

위 분석 내용을 검토 후 승인해주시면:
1. 이슈 A, F, B, C, G, H를 GitHub에 등록하고
2. 마일스톤 배정(M100 통합 또는 M110 신설)을 진행하겠습니다.
