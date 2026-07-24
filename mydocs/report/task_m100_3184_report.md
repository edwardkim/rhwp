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

## 후속: PR #3185 메인테이너 지적 — IR 필드 스윕 발산 +2건 (2026-07-24)

### 증상

메인테이너가 `ir_field_sweep_does_not_regress`(799샘플)에서 이 PR 단독으로
5개 파일의 `sections[].paragraphs[].controls[].cells[].paragraphs[].raw_header_extra[]`
발산이 +2씩 늘었다고 지적(이분탐색으로 원인 커밋 확정, 예: aift.hwpx
1477→1479). "memoProperties 방출이 refList 구조를 바꾸며 표 셀 문단
raw_header_extra 왕복에 부수효과를 낸 것 같다"는 가설을 남김.

### 조사 (실측)

1. `cargo test --test ir_field_sweep_baseline ir_field_sweep_does_not_regress`
   로 재현: 동일 5개 파일에서 +2 확인.
2. 이 커밋(8b11e2218)만 되돌린 worktree(upstream devel 기준)와 적용본을 각각
   빌드해 `sweep_hwpx_roundtrip(aift.hwpx)` 를 직접 호출·비교.
3. 지적된 경로(`sections[2].paragraphs[134].controls[0].cells[57].paragraphs[0]`)
   의 `raw_header_extra` 실제 바이트를 두 빌드에서 직접 추출: **완전히
   동일**(원본 `[0,0,0,0,0,0,0,0,0,0]` vs 재파싱 `[0,0,0,0,0,0,54,3,0,0]`,
   두 빌드 모두 동일값). 즉 이 특정 문단의 직렬화 결과 자체는 이 PR로
   **전혀 바뀌지 않았다.**
4. 그런데 되돌린 빌드에서 `sweep_documents()`를 직접 호출하면 이 경로가
   발산 목록에 **아예 없다**. 원인 추적 결과
   `src/diagnostics/ir_field_sweep.rs:70`의 `MAX_DIVERGENCES: usize = 2000`
   전역 캡 확인. `sweep_documents()`는 `sweep_doc_info()` → `sections`
   순으로 같은 `out: Vec<FieldDivergence>`에 계속 적재하며, 캡에 도달하면
   이후 모든 재귀 호출이 즉시 `break`한다(388~391행 등 각 재귀 지점마다
   동일 체크).
5. aift.hwpx는 이미 발산 총량이 캡(2000)에 근접/도달한 파일이다. 확인 결과:
   - 되돌린 빌드(버그 있는 상태): `doc_info` 레벨 발산 2건
     (`memo_shape_count: 1→0`, `extra_records.len: 6→5` — 바로 #3184
     버그 그 자체).
   - 이 PR 적용본: `doc_info` 레벨 발산 **0건**(정확히 수정됨).
   - 두 빌드 모두 `sweep_documents()` 총 발산 = **2000**(캡 도달).
   `sweep_doc_info()`가 섹션보다 먼저 실행되므로, doc_info 레벨 발산 2건이
   사라지면 캡 도달 시점이 딱 2건만큼 뒤로 밀려, 이미 존재했지만 캡에
   가려 기록되지 못했던 표 셀 `raw_header_extra` 발산(한컴 원본이 `id="0"`
   /`0x80000000` 같은 sentinel을 쓰고 우리 직렬화기는 항상 순차 id를
   새로 매기는, **이 PR과 무관한 기존 현상**)이 2건 더 드러난 것이다.

### 결론

**실제 직렬화 회귀가 아니다.** 표 셀 문단의 `raw_header_extra`(instanceId)는
이 PR 전후로 바이트 단위까지 동일하며, 이 PR이 고치는 `doc_info.memo_shape_count`
/`extra_records.len` 회귀 수정이 스윕 도구의 `MAX_DIVERGENCES=2000` 전역
캡 안에서 "예산"을 2건 절약해, 원래부터 있었던(그러나 캡에 가려 안 보이던)
표 셀 id 불일치 backlog 2건을 더 드러낸 것뿐이다. `baseline.tsv` 재생성
결과(`RHWP_IR_SWEEP_DUMP=... cargo test --profile release-test --test
ir_field_sweep_baseline`)에서도 이 5개 파일 모두 **`doc_info.memo_shape_count`
/`doc_info.extra_records.len` 행이 사라지고(수정 확인) 그 대가로 같은
5개 파일의 `raw_header_extra[]` 카운트가 +2**로 정확히 대응됨을 확인했다
— 우연이 아니라 캡 메커니즘의 직접적 산술 결과.

### 조치

- 소스 코드 변경 없음(고칠 실제 결함이 없음).
- `tests/fixtures/ir_field_sweep_baseline.tsv` 재생성(래칫 절차 그대로
  따름) → 5개 파일의 `raw_header_extra[]` 행 +2 갱신 + 9개 파일의
  `doc_info.memo_shape_count`/`doc_info.extra_records.len` 행 제거(진짜
  개선, #3184 수정이 aift.hwpx 외 8개 파일에서도 추가로 효과가 있었음을
  확인).
- 리베이스로 인해 hwp5rb 레인 일부(`chars[]`) 발산도 함께 줄었는데, 이는
  이 PR과 무관하게 upstream devel에 먼저 반영된 개선분이 재생성 시점에
  자연 반영된 것.
- `cargo test --test ir_field_sweep_baseline`(799샘플) green,
  `write_header_preserves_memo_properties_roundtrip` 여전히 PASS 확인.

### 참고: MAX_DIVERGENCES 캡의 한계 (별도 이슈 후보, 이번 PR 범위 밖)

캡이 낮은 파일에서는 "실제로는 무해한" doc_info 레벨 수정이 하위 트리의
드러나는 발산 개수를 흔들 수 있다는 뜻이라, 향후 유사한 헷갈림을 줄이려면
캡을 (a) 최상위에서만 걸거나 (b) 레인/경로별로 독립적으로 적용하는 개선을
고려할 만하다. 이번 PR 범위에서는 손대지 않음.
