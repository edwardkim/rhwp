---
kind: report
status: active
canonical: mydocs/report/task_m100_2550_report.md
last_verified: 2026-08-05
---

# Task #2550 처리 결과 — BinData 무제한 압축 해제 (deflate bomb)

- Issue: [#2550](https://github.com/edwardkim/rhwp/issues/2550) — `[보안] BinData 무제한 압축 해제`
- 브랜치: `task_m100_2550` (기준 `upstream/devel` `d3fb9de7c`)
- 방향: 이슈 코멘트에서 메인테이너가 권고한 **C안(경로별 차등)** 을 그대로 구현

## 1. 근인

상한 기구(`decompress_stream_limited`·`read_bin_data_limited`·`resolve_limited`·
`load_limited`)는 이미 있었으나 **프로덕션 호출부가 폰트 1곳뿐**이었다
(`queries/rendering.rs`). 저장·클립보드·렌더·질의는 전부 무제한 `load()` → `resolve()` →
`decompress_stream`(`read_to_end`, 상한 없음) 을 탔다.

파싱은 지연 등록이라 저렴하게 성공한다(`parser/mod.rs` 가 `has_stream` 만 확인하고
`BinDataBytes::Lazy` 등록). 폭발은 소비 시점에 일어난다.

## 2. 수정

### 상한값

`MAX_BIN_DATA_BYTES = 256MB` (`model/bin_data.rs`). 폰트 상한(32MB)보다 큰 실문서 임베드가
있어 별도 값이며, HWP3 레코드 상한 `HWP3_MAX_RECORD_SIZE` 와 같은 값으로 정합했다.

### 경로별 동작

| 경로 | 상한 초과 시 |
|------|--------------|
| HWP5 저장 (`serializer/cfb_writer.rs`) | 해제 없이 **원본 저장 바이트 통과** (무손실) |
| HWPX 저장 (`serializer/hwpx/mod.rs`) | 빈 엔트리 placeholder — [#1917] 로드 실패 계약 준수 |
| 렌더 (`renderer/layout*`) | placeholder (이미지 누락과 같은 경로) |
| 클립보드 (`commands/clipboard.rs`) | `RenderError("… 상한 … 초과")` |
| 질의 (`get_bin_data`, doclang) | `None` |
| 길이·빈 판정 (`len`/`is_empty`) | 출력 미적재 계수(`decompressed_len_capped`), 초과 시 0 |

### 저장 경로 무손실의 근거와 가드

저장 fallback 을 위해 `BinDataResolver::resolve_raw` 를 추가했다 — IR 이 원본 압축 바이트를
노출하지 않아 한 줄 수정이 불가능했던 지점이다. 반환형 `StoredBinData { bytes, compressed }`
에 **압축 상태를 함께 싣는다**: 저장 형태를 그대로 쓰려면 그 상태가 이 문서에서 기대되는
상태(`should_compress`)와 같아야 하고, 다르면 읽는 쪽이 압축 바이트를 원본으로 오해해
조용히 깨진다. 불일치 시에는 통과시키지 않고 경고 + 빈 스트림으로 접는다(암호 저장의 압축
강제 등 아주 좁은 corner).

암호 문서는 복호화(크기 1:1, `decrypt_hwp5_stream` 이 입력 길이로 truncate)만 수행하고
압축 해제는 하지 않으므로 폭탄 위험이 없다.

### 렌더 경로 정리 (동승)

`find_bin_data(...).map(|c| c.data.load())` 반복을 `find_bin_data_bytes()` 로 모았고,
OLE 분기가 같은 항목을 3번 해제하던 것을 1회 로드로 합쳤다(`shape_layout.rs`).

## 3. 검증

### 회귀 시험 (신규)

`tests/issue_2550_bin_data_decompression_bomb.rs` — 공격 문서는 커밋하지 않고
**시험 시점에 합성**한다(`tests/security_corpus_regression.rs` 와 같은 방침).
숙주 `samples/143E433F503322BD33.hwp` 의 첫 `/BinData` 스트림을 zeros deflate(해제 시 1GB)로
교체한다.

| 시험 | 수정 전(devel `d3fb9de7c`) | 수정 후 |
|------|------|------|
| `parsing_a_bomb_document_stays_cheap` | 통과 (공격 전제) | 통과 |
| `saving_…_preserves_original_stream_without_decompressing` | **실패** (전량 해제 후 재압축 → 바이트 불일치) | 통과 |
| `render_and_query_paths_fold_an_oversized_entry_to_placeholder` | **실패** | 통과 |
| `clipboard_image_queries_reject_an_oversized_entry` | **실패** | 통과 |
| `normal_documents_round_trip_unchanged_under_the_limit` | 통과 (손실 회귀 가드) | 통과 |

red→green 게이트를 실제로 확인했다 — devel 워크트리에 같은 파일을 넣어 3건 실패를 관측한
뒤 수정본에서 5건 전건 통과.

### 메모리 실측 (저장 경로, 이 PC)

| 폭탄 크기 | devel 최대 작업집합 | 수정본 최대 작업집합 |
|---|---|---|
| 1GB | **2058.8MB** | 522.7MB |
| 4GB | (미실행 — 기기 위험) | **535.0MB** |

수정본의 최대치는 **폭탄 크기가 아니라 상한에 묶인다**(1GB→4GB 에서 522.7→535.0MB, +2%).
잔여 ~520MB 는 bounded read 가 `max_bytes+1` 까지 읽어 보는 기존 `decompress_stream_limited`
의 전이 할당이며, 폰트 경로가 이미 쓰는 방식과 같다.

### 게이트

- `cargo clippy --all-targets -- -D warnings` — 통과
- `rustfmt --check` (변경 파일) — 통과 (이 PC 의 CRLF 잡음 제외)
- `cargo test --profile release-test --lib` — 3253 통과 / 0 실패
- `cargo test --profile release-test --tests` — 제출 시점 실행 중, CI 결과로 확인

## 4. 남은 사항

- **HWPX 저장의 상한 초과는 무손실이 아니다.** ZIP 엔트리는 해제된 바이트의 재압축이 필수라
  원본 통과 경로가 없다. [#1917] 이 정한 "로드 실패 = 빈 엔트리 placeholder + pic 컨트롤 보존"
  계약을 따랐다(오류로 중단하면 폭탄 문서 하나가 저장 자체를 막는다). ZIP 멤버를 해제 없이
  복사하는 경로를 열면 HWPX 도 무손실이 되며, 별도 이슈 감이다.
- 256MB 초과 정상 임베드가 실제로 존재하는 실문서를 확보하면 왕복 무손실 회귀를 표본으로도
  고정할 수 있다. 현재는 합성 폭탄과 상한 이내 표본으로만 양쪽을 눌렀다.
