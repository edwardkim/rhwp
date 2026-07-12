# 구현 계획서 — Task M100 #1891 (축 B: 외부 참조 BinData 왕복)

## 근본 원인 (3중 결함 체인)

73504 는 manifest 에 `isEmbeded="0"` 외부 파일 참조
(`<opf:item id="image1" href="D:\다운로드\" media-type="image/" isEmbeded="0"/>`,
부처 작성자 로컬 경로 잔재)를 갖고 본문 `<hp:pic>` 4곳이 이를 참조한다.

1. **직렬화기 bin_data_map 이 임베디드 전용** — `SerializeContext::collect_from_document`
   가 `doc.bin_data_content`(실바이트)만 등록. Link 참조 pic 은
   `resolve_bin_id` 실패 → `write_img` Err → section.rs 호출자가 pic 컨트롤을
   통째로 드롭 → 레이아웃 앵커 소실 → TextRun 재배치 (STRUCT 167px).
2. **manifest 순번 명명의 숫자 불변식 위반** — HWPX 파서(section.rs)는
   `binaryItemIDRef="imageN"` 의 숫자 N 을 **그대로 bin_data_id 로 파싱**한다
   (숫자 불변식). 순번(i+1) 명명은 Link 로 id 에 구멍이 생기면 이름과 id 가
   어긋나 재파스 그림 참조 전체가 시프트.
3. **파서 bin_data_items 필터의 media-type 의존** — `image/*` 또는 `BinData/`
   경로만 수집. 외부 참조가 `application/octet-stream` 등으로 방출되면 항목
   누락 → 후속 항목 인덱스(=bin_data_id) 전부 밀림 (2-round 불안정 r2=1).

## 수정 단계

1. `context.rs` — `BinDataEntry.is_embedded` 신설. 임베디드 등록 후
   `doc_info.bin_data_list` 의 `BinDataType::Link` 항목(storage_id≠0,
   #1567 placeholder 센티널 회피)을 manifest 에 등록. 명명은 숫자 불변식
   `image{bin_data_id}` 로 통일 (임베디드·링크 공통).
2. `mod.rs` — ZIP 쓰기 루프에서 링크 항목 스킵, 3-way 단언도 임베디드 한정.
3. `content.rs` — `isEmbeded` 를 항목별 실값("1"/"0")으로 방출.
4. `package_check.rs` — `isEmbeded="0"` 항목 href 는 ZIP 실재 검사 제외.
5. `parser/hwpx/content.rs` — 외부 참조 항목은 media-type 무관 수집.
6. 픽스처 + 핀: 렌더 자기정합 + Link/pic 수 2-round 보존 + content (id,크기) 보존.

## 계약 정제

- `img_uses_manifest_id`(lib test): 종전 "id=5 → image1"(순번) 기대는 숫자
  불변식 위반을 고정하고 있었음 → "image5" 로 갱신.

## 검증 계획

- 73504: hwpx-roundtrip PASS(diff 0, r2 0) + render-diff PASS
- cargo test 전 스위트 (hwpx_roundtrip_baseline 에 신규 픽스처 자동 포함)
- big_hwpx 2,500 A/B (rd_big_hwpx_1892fix 기준 — hwpx 경로는 현 devel 과 동등)

## 후속 (축 A — 별도 설계)

HWP5-in-.hwpx 10건: LINE_SEG 부재 문서의 빈 문단 높이 시멘틱.
`document.rs:78` `include_empty` 게이트의 소스 분기 + reflow 높이의 한컴 보정
(80250 한컴 17쪽 오라클 확보, output/poc/task1891/*_hancom2022.pdf).
