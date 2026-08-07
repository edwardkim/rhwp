# test-data-gen — 검증된 build-from-ingest 경로 기반 HWPX 테스트 데이터 생성기 (#4044 후속)

템플릿을 ingest JSON 으로 펼친 뒤 **rhwp 의 표준 `build-from-ingest` 경로로 실제
HWPX 를 산출**하고, 산출물마다 `rhwp info --json` 재검증을 파이프라인에 내장한
테스트 데이터 생성기다. 표준 라이브러리만 사용한다.

```
템플릿(config_templates.json)
  → ingest JSON (ingest_schema_v1 범위 내 결정적 생성)
  → rhwp build-from-ingest <ingest.json> -o <out.hwpx> --json
  → rhwp info <out.hwpx> --json   ← 산출물마다 자동 검증, 실패 시 비 0 종료
```

#4044 리뷰 2번 반영: 이전 구현은 zip 을 손으로 조립해 `Contents/header.xml` 이
없는 비정합 HWPX 를 만들었고 `rhwp info` 가 필수 파일 누락으로 거부했다. 이
재작성본은 메인테이너 권고대로 생성을 검증된 `build-from-ingest` 경로에
위임한다 — OWPML 구조 정합은 rhwp 본체가 보장하고, 이 도구는 템플릿 → ingest
변환과 산출물 검증만 책임진다.

## 요구사항

- Python 3 (표준 라이브러리만)
- `rhwp` 실행 파일 — 해석 순서: **`--rhwp-bin` 인자 > `RHWP_BIN` 환경변수 > PATH**
  - 명시한 경로는 실행 가능한 일반 파일이어야 한다. 없는 파일·디렉터리·실행 권한 없는 파일은
    환경 오류(exit 2)로 거부한다.

```bash
cargo build --bin rhwp        # target/debug/rhwp
export RHWP_BIN=target/debug/rhwp
```

## 사용법

```bash
# 전체 템플릿 생성 + 검증
python tools/test-data-gen/hwp_test_data_generator.py --output-dir out/testdata

# 특정 템플릿만, 기계 판독 요약
python tools/test-data-gen/hwp_test_data_generator.py \
  --output-dir out/testdata --template minimal --template large --json

# 템플릿 목록
python tools/test-data-gen/hwp_test_data_generator.py --list

# 중간 ingest JSON 을 산출물 옆에 보존 (디버깅용)
python tools/test-data-gen/hwp_test_data_generator.py --output-dir out --keep-ingest
```

같은 `(템플릿, --seed)` 조합은 항상 같은 ingest JSON 을 만든다(기본 시드 42).

### 종료 코드

| 코드 | 의미 |
|------|------|
| 0 | 전 템플릿 생성 + `rhwp info --json` 검증 성공 |
| 1 | `build-from-ingest` 실패 또는 산출물 검증 실패 (rhwp 원인 메시지 표면화) |
| 2 | 사용법·설정·환경 오류 (미지 템플릿 키, 범위 밖 값, rhwp 바이너리 부재 등) |

### `--json` 요약 출력

```json
{
  "schemaVersion": "1.0",
  "generator": "hwp_test_data_generator",
  "seed": 42,
  "count": 5,
  "documents": [
    {
      "name": "large",
      "output": "out/testdata/large.hwpx",
      "bytes": 8773,
      "questionCount": 30,
      "paragraphCount": 281,
      "pageCount": 7,
      "verified": true
    }
  ]
}
```

`verified` 는 해당 산출물이 `rhwp info --json` 을 통과했다는 뜻이다(하나라도
실패하면 요약 없이 exit 1).

## 기본 템플릿

| 이름 | 내용 |
|------|------|
| `minimal` | 문항 1개, 선택지 2개 — 가장 작은 유효 문서 |
| `simple` | 3문항 5지선다, 문항당 지문 2문단 |
| `structured` | 머리말·꼬리말·형식 라벨 + 공유 지문 2개 + `<보기>` 박스 |
| `media` | 이미지 블록 포함 — 실물 파일 없이 placeholder 로 처리됨 |
| `large` | 30문항·지문 3문단·`<보기>` 6개 — 다중 페이지(실측 7쪽) 스트레스 |

## 템플릿 키 (ingest_schema_v1 이 표현 가능한 범위)

| 키 | 의미 |
|----|------|
| `questions` | 문항 수 (≥1) |
| `choices_per_question` | 문항당 선택지 수 (1~5, 라벨 ①~⑤) |
| `stem_paragraphs` | 문항당 지문 텍스트 블록 수 (≥1) |
| `boxed_every` | N번째 문항마다 `<보기>` 박스 블록 (0=끔) |
| `media_every` | N번째 문항마다 이미지 블록 + media 항목 (0=끔) |
| `passages` / `questions_per_passage` | 공유 지문 수 / 지문당 문항 수 (앞 문항부터 묶음) |
| `header_text` / `footer_text` / `form_label` | 머리말 / 꼬리말 / 형식 라벨 |
| `default_font` / `page_size` | ingest 최상위 필드로 그대로 전달 |

고급: `raw_ingest` 키에 ingest JSON 전체를 통짜로 넣으면 변환 없이 그대로
`build-from-ingest` 에 전달한다(경계 사례 fixture 제작용 — 실패 회귀 테스트가
이 경로로 스키마 위반 입력을 주입한다).

템플릿 이름은 산출물 파일명으로도 사용하므로 비어 있거나 `.`·`..`, `/`·`\\` 같은
경로 구분자가 들어갈 수 없다. 모든 산출물은 반드시 `--output-dir` 바로 아래에 만든다.

### 이전 config 에서 제거된 항목 (README 정정)

이전 구현의 `num_tables`·`num_images`(실제 픽셀 임베드)·`use_styles`·
`use_colors`·`include_complex_shapes` 는 **ingest_schema_v1 이 표현할 수 없는
항목**이라 제거했다. 스키마 v1 이 지원하는 것은 문항/선택지/텍스트 블록/`<보기>`
박스/공유 지문/이미지 참조(placeholder)/머리말·꼬리말·형식 라벨/페이지 크기·기본
폰트까지다. 표·글자 스타일이 필요한 fixture 는 ingest 스키마 확장 후에 추가할
수 있다.

## 테스트

```bash
RHWP_BIN=target/debug/rhwp python tools/test-data-gen/test_hwp_test_data_generator.py
# 또는
python -m unittest discover -s tools/test-data-gen -p "test_*.py"
```

회귀 고정 내용:

- **성공 케이스**: 전 템플릿 산출물이 `rhwp info --json` 통과 — 생성기 내장
  검증과 별도로 테스트가 독립 재실행해 이중 확인, `large` 는 pageCount > 1
- **실패 케이스**: 스키마 위반 ingest(boxed 블록에 `text`, `choices` 누락)는
  rhwp 원인 메시지를 표면화하며 exit 1
- 설정 오류(미지 키, `choices_per_question` 6, 없는 바이너리, 없는 템플릿)는 exit 2
- 경로형 템플릿 이름과 실행 권한 없는 rhwp 경로도 exit 2이며 traceback을 출력하지 않음
- 같은 시드 결정성, 템플릿 knob → ingest 구조 대응(박스/미디어/공유 지문 위치)

rhwp 바이너리가 없으면 파이프라인 테스트는 skip 되고 순수 파이썬 테스트만 돈다.
