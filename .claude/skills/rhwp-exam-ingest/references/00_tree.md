# 00 — 판단 트리

이 장은 에이전트가 시험지 입력을 받고 **첫 30초**에 어느 helper·어느 필드를
고르는지 고정한다. 새 CLI 를 만들지 않는다. `exam_paper` 빌더를 고치지 않는다.

## 한 줄

입력 종류를 고르고 → 페이지 PNG 또는 텍스트를 만들고 → Vision 으로
`ingest.json` 을 쓰고 → bbox 를 자르고 → `rhwp build-from-ingest --media-dir -o`
로 조립하고 → `dump`/`export-text` 로 텍스트가 들어갔는지 확인한다.

## 살아 있는 동사

아래 상자 순서만 실제 명령이다.

```
check_deps.sh [--json]
pdf_to_pngs.sh [--json] [--dry-run] <pdf> <dir> [dpi]
extract_docx.py [--json] [--dry-run] <docx> <dir>
crop_image.sh [--json] [--dry-run] <src> <x> <y> <w> <h> <out>
rhwp build-from-ingest <ingest.json> --media-dir <dir> -o <out.hwpx>
rhwp dump <out.hwpx>
rhwp export-text <out.hwpx> -o <dir>
unzip -l <out.hwpx>
```

없는 명령 (발명 금지):

- `rhwp exam-from-pdf` — 발명 금지
- `rhwp ingest-exam` — 발명 금지
- `rhwp build-exam` — 발명 금지
- `hwp_doc_exam` — 발명 금지
- `rhwp crop-image` — 발명 금지
- `rhwp pdf-to-png` — 발명 금지
- `rhwp import-md` — 발명 금지

이미지 패스스루와 MD 이미지 ref 는 helper 가 아니라 **에이전트가 파일을 읽고
경로를 media id 로 옮기는 규약**이다.

## 입력 → 첫 동작

| 확장자/단서 | 첫 동작 | 다음 | 정지 |
| --- | --- | --- | --- |
| `.pdf` | `check_deps` 후 `pdf_to_pngs.sh` | Vision 각 `page_NNN.png` | F03/F05 |
| `.png` `.jpg` `.jpeg` `.webp` | 패스스루. Read 로 직접 | crop 만 | F06 |
| `.md` `.markdown` | 본문 Read + `![alt](path)` | media 경로 확인 | F07 |
| `.docx` | `extract_docx.py` | text.txt + img/ | F04/F08 |
| `.hwp` `.hwpx` | 이 스킬이 아니다 | `rhwp-doc-triage` 또는 편집 스킬 | — |
| 폴더에 PDF 200개 | 이 스킬의 단건 루프 | `rhwp-bulk-pipeline` 이 아님 (그건 HWP 배치) | — |

## Vision 분기

페이지 PNG 를 읽은 뒤:

1. 문제 번호 마커 (`1.` `1)` `①` `문1.`) 위치를 적는다.
2. 공유 지문 지시문 (`[1~3] 다음 글을 읽고`) 이 있으면 `passages[]` 후보.
3. `<보기>` `[보기]` 테두리 박스는 `stem_blocks` `boxed`.
4. 그래프/그림/표/수식은 bbox 를 픽셀로 적고 placement 를 고른다.
5. 선택지 ①–⑤ (또는 ①–④) 를 `choices[]` 로.

한 페이지에 문항이 너무 많으면 사분면으로 나눠 다시 읽는다 (F09).
번호가 안 읽히면 원본을 다시 받는다 (F10). 추측으로 문항을 만들지 않는다.

## ingest 분기

| 관찰 | 필드 |
| --- | --- |
| 여러 문항이 같은 글 | `passages[].id` + `passage_ref` |
| 보기 박스 | `{"type":"boxed","title":"<보기>","blocks":[...]}` |
| 그림이 질문과 선택지 사이 | `placement: "between"` |
| 그림이 질문보다 위 | `placement: "above"` |
| 그림이 선택지 다음 | `placement: "below"` |
| 그림이 문장 한가운데 | `placement: "inline"` |
| stem 이 `"3. 밑줄 친"` 으로 시작 | `auto_number: false` |
| stem 이 번호 없음 | `auto_number: true` (기본) |

스키마에 없는 키 (`answer`, `score`, `latex`, `table_html`) 를 넣지 않는다.
`deny_unknown_fields` 가 즉시 실패한다 (#3358).

## 빌드 분기

```
ingest.json 준비됨
  ├─ media[] 비어 있음 ──▶ build-from-ingest -o  ( --media-dir 생략 가능 )
  └─ media[] 있음
       ├─ crop 아직 안 함 ──▶ crop_image.sh (F14)
       └─ 파일 존재 ──▶ build-from-ingest --media-dir -o (F15)
            ├─ unknown field ──▶ JSON 수정 (F11)
            ├─ boxed.text ──▶ blocks 로 (F12)
            └─ 성공 ──▶ dump / export-text / unzip -l
```

## 한계 분기

| 관찰 | 하는 일 | 하지 않는 일 |
| --- | --- | --- |
| 적분·행렬 수식 | bbox crop → image 블록 | LaTeX 필드를 ingest 에 추가 |
| 3단 병합 표 | 표 전체를 Picture | Table IR 발명 |
| 손글씨 메모 | 무시하거나 이미지 | OCR 엔진 호출 |
| Picture 가 한컴에서 안 보임 | 한계 고지 (#182) | writer 를 이 스킬에서 고침 |

## 이 트리가 고르지 않는 것

- gym 과제·채점기
- `exam_paper.rs` 의 문단/테두리 구현
- 새 `rhwp` 서브커맨드
- 이웃 스킬 본문 재작성
