---
kind: report
status: done
---

# 처리 결과: refList memoProperties hwpx→hwpx 라운드트립 소실

Issue: #3184

## 문제

HWPX 헤더 `<hh:refList>`의 마지막 자식인 `<hh:memoProperties>`(메모 모양 정의:
테두리 두께/색상/채우기색/타입)가 hwpx→hwpx 라운드트립(parse 후 재직렬화)에서
통째로 사라진다.

## 근거 (실물 샘플)

`samples/hwpx/aift.hwpx`의 `Contents/header.xml` `refList` 마지막 자식:

```xml
<hh:memoProperties itemCnt="1">
  <hh:memoPr id="1" width="15591" lineWidth="3" lineType="SOLID"
    lineColor="#A9A9A9" fillColor="#CBFF99" activeColor="#FDBCDD" memoType="NOMAL"/>
</hh:memoProperties>
```

## 원인

`src/parser/hwpx/header.rs::parse_memo_shape()`는 `<hh:memoPr>`을 읽어
`DocInfo.extra_records`에 `HWPTAG_MEMO_SHAPE` 바이너리 레코드로만 쌓는다. 이
레코드는 `src/serializer/doc_info.rs`(HWP5 DocInfo 스트림 직렬화, hwpx→hwp5
변환 전용)에서만 소비된다. HWPX 재직렬화 경로인
`src/serializer/hwpx/header.rs::write_header()`는 `extra_records`를 전혀
참조하지 않고, `fontfaces`~`styles`까지만 `refList` 자식을 재구성해 방출한 뒤
바로 `</hh:refList>`를 닫는다 — `memoProperties`를 방출하는 코드 자체가
없었다.

## 3중 중복확인

- upstream log: `git log --oneline -- src/serializer/hwpx/header.rs`에
  memoProperties 관련 커밋 없음.
- gh 이슈 검색: `memoProperties`/`memoShape` 검색 결과 기존 이슈 없음(관련
  이슈 #2779는 `secPr memoShapeIDRef` 참조 보존이고, #2978은 `parse_memo_shape`
  내부 lineType enum 매핑 오류로 별개 지점).
- git grep: `write_header.rs` 전체에 `memoProperties`/`memoPr` 토큰이 전무함을
  확인.

## 재현 (red)

`src/serializer/hwpx/header.rs`에 테스트 추가 후, 직렬화기 splice 로직을
임시로 비활성화(`if false { ... }`)하고 확인:

```
test serializer::hwpx::header::tests::write_header_preserves_memo_properties_roundtrip ... FAILED
재직렬화된 header.xml에 memoProperties가 있어야 함
```

## 수정

`hwpx_head_tail`(refList 뒤 compatibleDocument/docOption/trackchageConfig
splice)과 동일한 verbatim splice 패턴 적용:

1. `src/model/document.rs`: `DocInfo.memo_properties_xml: Option<String>` 필드
   추가.
2. `src/parser/hwpx/header.rs`: `extract_memo_properties()` 추가 — 헤더 원문에서
   `<hh:memoProperties>...</hh:memoProperties>`(또는 자기닫힘) 블록을 그대로
   추출해 저장. `parse_hwpx_header()`에서 `hwpx_head_tail`과 나란히 호출.
3. `src/serializer/hwpx/header.rs`: `write_styles()` 이후, `</hh:refList>` 닫기
   전에 `doc.doc_info.memo_properties_xml`이 있으면 그대로 splice.

기존 `parse_memo_shape()`/`extra_records`(hwpx→hwp5 변환용)는 그대로 유지.

## 검증 (green)

```
test serializer::hwpx::header::tests::write_header_preserves_memo_properties_roundtrip ... ok
```

`cargo test --lib`: 전체 2553개 통과, 실패 1건은 이번 변경과 무관한
`renderer::font_paths::tests::env_font_paths_parses_and_filters`
(환경변수/tmp 경로 의존, 손대지 않은 `font_paths.rs`에서 발생 — 사전 존재하는
환경 이슈로 판단).

## 파일

- `src/model/document.rs`
- `src/parser/hwpx/header.rs`
- `src/serializer/hwpx/header.rs`
