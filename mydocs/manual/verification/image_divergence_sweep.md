---
kind: guide
status: active
canonical: mydocs/manual/verification/image_divergence_sweep.md
last_verified: 2026-08-05
---

# SVG↔studio 이미지 발산 스윕 하네스

문서 안 그림이 **소비자가 디코드할 수 없는 mime** 으로 방출되는 문서를 코퍼스에서 찾는
스윕이다. #4057(WMF)·#4060(EMF)에서 "변환기는 있는데 방출 경로에 안 걸려 있는" 부류의
결함을 전수 검출하는 데 썼다. 하네스는 `examples/` 의 4종이며 회차 결과는
`mydocs/report/survey_image_divergence_r{N}_{YYYYMMDD}.md` 로 남긴다.

## 1. 판정 모델

`audit_studio_image_parity` 가 문서당 3개 방출 경로의 mime 을 집계해 JSONL 1줄을 낸다.

| 경로 | API | 소비자 |
| --- | --- | --- |
| `flow` | `get_page_flow_image_ops` | studio DOM `<img>` 좁은 질의 (#3315, `flow-image-url-cache.ts` Blob) |
| `layer` | `get_page_layer_tree_with_profile(page, "screen", omit_bytes)` | studio layer tree 폴백 (인라인 base64) |
| `svg` | `render_page_svg` | SVG 내보내기의 data URI |

합격 집합은 두 벌이다 — studio 경로는 `BROWSER_IMG_OK`(png·jpeg·gif·webp·**bmp**·svg+xml·
x-icon), SVG 경로는 `SVG_EMBED_OK`(png·jpeg·gif·webp·svg+xml). BMP 가 SVG 쪽에서 빠지는
이유는 SVG `<image>` 가 data URI BMP 를 표준 지원하지 않아서다(`image_resolver.rs` 주석).
셋 중 한 경로라도 합격 집합 밖 mime 이 나오면 `flagged:true`.

`flagged` 는 "그림이 안 보인다"의 근사다 — octet-stream·x-wmf 등이 나오면 브라우저/SVG
소비자가 그 바이트를 못 그린다. 반대로 flagged 아님이 시각 정합을 보증하지는 않는다
(mime 만 보고 픽셀은 안 본다).

## 2. 실행 레시피

release 로 빌드해야 10k 규모가 실용 시간에 끝난다(12갈래 병렬로 수 분).

~~~bash
cargo build --release --example audit_studio_image_parity --example audit_image_magic

find <코퍼스> -type f \( -name "*.hwp" -o -name "*.hwpx" \) | sort > /tmp/corpus_list.txt
RHWP_FONT_PATH=ttfs/hwp:ttfs/windows \
  xargs -a /tmp/corpus_list.txt -d '\n' -n 25 -P 12 \
  target/release/examples/audit_studio_image_parity > /tmp/sweep.jsonl 2>/tmp/sweep.err
~~~

- stderr 로 `LAYOUT_OVERFLOW` 등 렌더 로그가 대량으로 나오므로 반드시 분리한다.
- 문서당 1줄이므로 `wc -l` 로 진행률을 본다. 파서 실패·패닉은 `"ok":false` 로 남는다
  (10k 기준 52건 — 페이지네이션 서베이의 ERR 코호트와 같다).
- 집계: `flagged` 필터 후 `studioBadFlow`/`studioBadLayer`/`svgBad` 맵을 합산한다.

## 3. 2차 조사 — 정체 판별과 원문 덤프

flagged 문서는 3종으로 좁힌다.

~~~bash
# (1) octet-stream 의 정체: source image 바이트의 매직 직접 스니핑 + pageBackground base64 검사
xargs -a /tmp/flagged.txt -d '\n' target/release/examples/audit_image_magic > /tmp/magic.jsonl

# (2) 특정 문서의 source image op 바이트를 파일로 덤프 — 변환 실패 건 조사용
cargo run --release --example dump_source_images -- <file> <outdir>

# (3) 페이지 layer JSON 원문 덤프 — 스윕 anomaly 조사용 (profile 기본 screen)
cargo run --release --example dump_layer_json_page -- <file> <page> [profile]
~~~

`audit_image_magic` 의 `unknown:<hex12>` 는 선두 12바이트 hex 다. 자주 나오는 정체:

| 선두 바이트 | 정체 |
| --- | --- |
| `252150532d41646f62652d33` (`%!PS-Adobe-3`) | PostScript/EPS 텍스트 |
| `c5d0d3c6` | DOS EPS 바이너리 헤더 (MS-DOS EPS preamble) |
| `0a03` | PCX v2.8 (판별기는 v5 `0a05` 만 인식) |

## 4. 판정 이후

- 원인 부류가 "변환기는 있는데 경로에 안 걸림"이면 #4057/#4060 의 수정 자리
  (`image_resolver.rs` 의 `resolve_image_payload`·`emitted_image_bytes`·
  `detect_image_mime_type` 두 벌, `svg.rs`·`html.rs`·`web_canvas.rs` 의 방출 분기)를 본다.
- 판별기(`detect_image_mime_type`)에 매직이 없어 octet-stream 으로 새는 부류는 판별
  추가가 선행이다 — mime 이 잡혀야 변환 분기에 걸린다.
- 변환기 자체가 없는 부류(PostScript/EPS 등)는 별도 이슈로 승격한다.
- 회귀 고정은 합성 바이트 테스트(`emf_is_emitted_as_svg_not_raw_emf` 패턴)로 하고,
  외부 수집 코퍼스 문서는 fixture 로 커밋하지 않는다(비공개·용량).
