# 06 — ingest_schema_v1

정본 JSON Schema: `tools/rhwp-ingest/schema/ingest_schema_v1.json`.
Rust 모델: `src/parser/ingest/schema.rs`. 둘 다 `version: "1"` 만 허용하고
`deny_unknown_fields` 다 (#3358).

이 스킬은 스키마를 **소비**한다. 필드를 추가하거나 빌더를 고치지 않는다.

## 필수

| 경로 | 형 | 메모 |
| --- | --- | --- |
| `version` | `"1"` | 다른 문자열 거부 |
| `questions` | array | 비어 있으면 빌더는 빈 문서를 만들 수 있으나 시험지가 아니다. 최소 1 |
| `questions[].number` | integer ≥ 1 | |
| `questions[].stem` | string | `stem_blocks` 가 있으면 fallback |
| `questions[].choices` | array | `{label, text}` |

## 선택 (문서)

| 경로 | 기본 | 용도 |
| --- | --- | --- |
| `page_size.width_mm` | 210 | A4 |
| `page_size.height_mm` | 297 | A4 |
| `default_font` | `함초롬바탕` | 시험지에서 읽은 폰트 또는 이 값 |
| `header_text` | 없음 | 반복 머리말 |
| `footer_text` | 없음 | 쪽 번호 힌트 `"1/20"` |
| `form_label` | 없음 | `홀수형` / `짝수형` |
| `passages` | `[]` | 공유 지문 |

## 선택 (문항)

| 경로 | 기본 | 용도 |
| --- | --- | --- |
| `passage_ref` | 없음 | `passages[].id` |
| `stem_blocks` | `[]` | text / image / boxed |
| `media` | `[]` | `--media-dir` 상대 경로 |
| `auto_number` | `true` | 첫 stem 앞에 `{n}. ` |

## StemBlock 세 종류

`type` 이 태그다. 종류마다 허용 필드가 다르다. 섞으면 힌트와 함께 실패.

### text

```json
{"type": "text", "text": "다음 글의 주제로 가장 적절한 것은?"}
```

금지: `ref`, `placement`, `title`, `blocks`.

### image

```json
{"type": "image", "ref": "img/q1_passage.png", "placement": "between"}
```

필수: `ref` (`media[].id` 와 일치). `placement` 기본 `between`.
금지: `text`, `title`, `blocks`.

### boxed

```json
{
  "type": "boxed",
  "title": "<보기>",
  "blocks": [{"type": "text", "text": "보기 본문"}]
}
```

`title` · `blocks` 선택. 금지: `text`, `ref`, `placement`.
**사고 형태** (#3358): `{"type":"boxed","text":"소속: 성명:"}` →
`boxed 블록에 허용되지 않는 필드 'text'`. 본문은 반드시 `blocks[]`.

알 수 없는 type: `알 수 없는 블록 type 'X' (지원: text|image|boxed)`.

## Media

```json
{
  "id": "img/q1_passage.png",
  "natural_w": 800,
  "natural_h": 600,
  "target_w_mm": 80,
  "placement": "between"
}
```

필수: `id`, `natural_w` ≥ 1, `natural_h` ≥ 1.
`target_w_mm` 생략 시 본문폭 70%.
`placement`: `between` | `above` | `below` | `inline`.

## 미지 필드 — 즉시 실패

넣지 말 것 (관측된 유혹):

- `answer`, `correct`, `score`, `points`
- `latex`, `equation`, `omml`
- `table`, `rows`, `html`
- `comment`, `$comment`, `_debug`
- `page`, `source_pdf`, `bbox` (bbox 는 ingest 필드가 아니다. crop 인자다)

빌더가 무시해 주길 바라지 마라. 거절한다.

## 최소 유효 문서

`tools/rhwp-ingest/schema/sample_minimal.json` 과
`fixtures/schemas/valid_minimal.json` 이 같은 모양이다.

`version` + `questions[]` 만으로도 파싱된다. `page_size`/`default_font` 는
Rust 기본값이 채운다 (`test_parse_legacy_minimal_defaults`).

## 구조 샘플

`sample_structured.json`: `header_text`, `footer_text`, `form_label`,
`passages`, `passage_ref`, `boxed` 보기.

픽스처 카탈로그: `fixtures/schemas/catalog.json`.
invalid_* 파일은 **빌더에 넣지 않는** 거부 표본이다.
