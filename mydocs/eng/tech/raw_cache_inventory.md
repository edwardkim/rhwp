# raw 캐시 인벤토리 — 필드별 해석 상태

관련 이슈: [#5890](https://github.com/edwardkim/rhwp/issues/5890) (역공학 이중 장부의 구조적 undo 비용)

## 왜 이 문서가 있나

rhwp 는 폐쇄 포맷(HWP)을 부분 역공학하면서 저장 fidelity 를 약속한다. 그래서 파싱된 IR 과
**미해석 원본 바이트**를 둘 다 진실로 보관하는 **이중 장부** 상태다. MS Word 는 포맷=모델,
LibreOffice·OnlyOffice 는 완전 해석 후 export 재생성이라 이 비용 클래스가 없다 — rhwp 고유의,
역공학 과정에서 필연적인 비용이다.

비용의 실체는 크기가 아니라 **정합성**이다. raw 캐시 하나하나는 수십~수백 바이트지만,
setter 가 이것을 파괴하면 되돌릴 원본이 사라져 그 조작은 **스냅샷으로만** undo 된다.
스냅샷은 문서 전체를 담으므로 undo 예산을 소모하고, 실효 undo 깊이를 깎는다(#3230 → #5769).

#5890 의 완료 정의는 두 가지다. (1) 신규 objectProps 계열 조작이 스냅샷 없이 저장 바이트
왕복 동일성을 충족하고, (2) **남아 있는 raw 캐시의 목록(필드별 해석 상태)이 문서화된다.**
이 문서가 (2)다.

## 읽는 법 — 세 축

| 축 | 뜻 | #5890 비용 |
|---|---|---|
| **장치** | 캐시가 아니라 캐시를 지키는 봉인·저널·더티 플래그 | 해당 없음 |
| **순수 passthrough** | 파싱 때 채워지고 편집 계층이 **쓰지 않는다** | 없음 — 저장까지 그대로 흐른다 |
| **편집이 씀** | 편집 계층에 대입·`clear`·`push` 가 있다 | 여기서만 발생한다 |

"편집이 씀"이 곧 결함은 아니다. 값이 실제로 바뀌어 파생 상태가 무효가 된 경우의 초기화는
정당하다. 결함은 **값 변화 없이 파괴**하는 경우다 — 그때 한컴 원본 바이트가 근거 없이
rhwp 재생성본으로 바뀐다.

## 복원 장치

| 대상 | 장치 | 계약 |
|---|---|---|
| DocInfo `raw_stream` | `DocInfoSeal` (#4493) | 모델·raw 다이제스트 쌍이 **둘 다** 맞을 때만 통과. 공개 필드를 직접 바꿔도 승인되지 않는다 |
| Section `raw_stream` | `SectionSeal` (#4488) + `SectionRawCapture` 저널 (#5951) | capture→new→old→restore 로 저장 바이트가 완전 수렴한다 |
| Control / Table `raw_ctrl_data` | `raw_ctrl_seal` (#4495) | 봉인 불일치면 저장기가 IR 합성 경로로 내려간다 |
| OLE `raw_tag_data` | `raw_tag_seal` (#4495) | 동일 |
| 선택 삭제 조각 | `delete_fragment` 저널 | 구역 raw + 봉인을 함께 캡처 |

계약 전문은 `src/model/raw_provenance.rs` 모듈 주석에 있다.

## 값 변화 없는 파괴 — 알려진 사례

| 대상 | 사이트 | 상태 |
|---|---|---|
| 수식 `raw_ctrl_data` | `object_ops/equation.rs` 무조건 `clear()` | **해결** [#6350](https://github.com/edwardkim/rhwp/issues/6350) — #4495 봉인 판정으로 대체 |
| 그림 `raw_rendering` | `TRANSFORM_KEYS` 텍스트 스캔 | **해결** [#6355](https://github.com/edwardkim/rhwp/issues/6355) — 값 변화 지문으로 전환 |
| 그림 회전 저장 비트 | `refresh_picture_rotation_layout_for_save` | **해결** [#6373](https://github.com/edwardkim/rhwp/issues/6373) — 한컴 오라클로 파싱값 보존이 정답임을 확인 |
| 표 `raw_ctrl_data` | `table_ops.rs` 0 확장 3곳 | **해결** [#6388](https://github.com/edwardkim/rhwp/issues/6388) — 길이 가드 덧쓰기 |
| 묶음 `raw_rendering` | `set_shape_properties_native` / `apply_shape_props_inner` | **해결** [#6740](https://github.com/edwardkim/rhwp/issues/6740) — 지문 판정을 묶음에도 |
| 커넥터 `raw_rendering` | `update_connectors_in_section` | **미판정** — 코드상 값 비교가 없으나 표본에서 재현되지 않았다(아래) |

### 커넥터 경로가 미판정인 이유

`update_connectors_in_section` 은 재계산한 bbox 를 기존 값과 비교하지 않고 `raw_rendering` 을
비운다. 그러나 `2025 행정업무운영 편람(최종).hwp` 의 커넥터 4개로 무변경 호출을 재현하면
`raw_rendering` 이 그대로고 저장 바이트도 동일하다 — 이 표본의 커넥터는 subject id 가 어떤
도형의 `inst_id` 와도 매칭되지 않아 clear 분기 전에 `continue` 된다. 분기가 실제로 발동하는
문서를 확보하기 전에는 결함으로 단정하지 않는다.

### 파괴의 대가가 문서마다 다르다 — 측정 시 주의

`raw_rendering` 이 비면 저장기는 파싱된 `render_*` 필드로 행렬을 **재생성**한다
(`serializer/control.rs` 의 rendering 블록). 그 결과가 원본과 바이트 동일한 문서도 있다.
표본 스윕에서 `raw_rendering` 파괴가 저장 바이트를 실제로 바꾼 개체는 **300건 중 75건**이고,
그중 묶음은 4건이었다(`group-box.hwp`, `shape-group-02.hwp`, `kps-ai.hwp`, `hwp-3.0-HWPML.hwp`).
따라서 **판정력 없는 표본을 고르면 결함이 없는 것처럼 보인다.**

또 하나 — 저장 바이트 차이를 raw 캐시에 귀속하려면 **대조군**이 필요하다. 편집이라면 무엇이든
`section.raw_stream = None` 으로 구역 패스스루를 깨므로, 그것만으로도 저장본이 원본과 갈린다.
비교 기준은 "패스스루만 무효화한 저장본"이어야 한다.

## 전체 인벤토리

`src/model/**` 의 `pub raw_*` 필드 전수다. 편집 계층 = `src/document_core/**`.

| 필드 | 위치 | 담는 것 | 축 | 복원 장치 |
|---|---|---|---|---|
| `raw_data` | `bin_data.rs:32` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_ctrl_data` | `control.rs:160` | 라운드트립용 원본 ctrl_data | 편집이 씀 | `raw_ctrl_seal`(control.rs:164) — #4495 |
| `raw_ctrl_seal` | `control.rs:164` | `raw_ctrl_data` 무결성 봉인 | —(장치) | 없음 |
| `raw_type` | `control.rs:852` | HWPX `fieldBegin type` 원문 — IR 이 모르는 종류일 때만 | 순수 passthrough | 없음 |
| `raw_parameters_xml` | `control.rs:880` | HWPX `hp:parameters` 원문 verbatim | 순수 passthrough | 없음 |
| `raw_data` | `document.rs:144` | 원본 FileHeader 256바이트 (직렬화 시 원본 복원용) | 편집이 씀 | 없음 |
| `raw_data` | `document.rs:160` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_stream` | `document.rs:220` | 원본 DocInfo 레코드 스트림 바이트 (직렬화 시 원본 복원용) | 편집이 씀 | `DocInfoSeal`(document.rs:248) — #4493 |
| `raw_stream_dirty` | `document.rs:234` | raw_stream 이 모델과 어긋났음을 표시 | —(장치) | 없음 |
| `raw_provenance` | `document.rs:248` | DocInfo raw 출처 봉인(모델·raw 다이제스트 쌍) | —(장치) | 없음 |
| `raw_stream` | `document.rs:282` | 원본 BodyText 레코드 스트림 바이트 (직렬화 시 원본 복원용) 편집 시 None으로 초기화하여 재직렬화 유도 | 편집이 씀 | `SectionSeal`(document.rs:286) + 구역 raw 저널 — #4488 / #5951 |
| `raw_provenance` | `document.rs:286` | Section raw 출처 봉인 | —(장치) | 없음 |
| `raw_ctrl_extra` | `document.rs:349` | CTRL_HEADER 데이터의 파싱된 필드 이후 추가 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_unknown` | `footnote.rs:91` | HWP5 미문서화 2바이트 | 편집이 씀 | 없음 |
| `raw_attr` | `header_footer.rs:13` | 원본 attr u32 전체 (라운드트립 보존용) | 순수 passthrough | 없음 |
| `raw_ctrl_extra` | `header_footer.rs:15` | CTRL_HEADER ctrl_data의 4바이트(attr) 이후 추가 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_attr` | `header_footer.rs:36` | 원본 attr u32 전체 (라운드트립 보존용) | 순수 passthrough | 없음 |
| `raw_ctrl_extra` | `header_footer.rs:38` | CTRL_HEADER ctrl_data의 4바이트(attr) 이후 추가 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_list_header` | `header_footer.rs:106` | LIST_HEADER raw data (라운드트립 보존용) | 순수 passthrough | 없음 |
| `raw_picture_extra` | `image.rs:40` | SHAPE_PICTURE 레코드의 파싱된 필드 이후 추가 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_attr` | `page.rs:146` | 원본 attr u16 전체 (라운드트립 보존용, 0이면 재구성) | 순수 passthrough | 없음 |
| `raw_break_type` | `paragraph.rs:20` | 원본 break_type 바이트 (라운드트립 보존용, 0이면 column_type에서 재구성) | 편집이 씀 | 없음 |
| `raw_header_extra` | `paragraph.rs:95` | PARA_HEADER tail 보존용 바이트 | 편집이 씀 | 없음 |
| `raw_break_type` | `paragraph.rs:134` | 원본 break_type 바이트 (셀 문단) | 편집이 씀 | 없음 |
| `raw_header_extra` | `paragraph.rs:137` | PARA_HEADER tail — instanceId 및 변경추적 suffix라 문단마다 고유하다 | 편집이 씀 | 없음 |
| `raw_digest` | `raw_provenance.rs:53` | 봉인 다이제스트 | —(장치) | 없음 |
| `raw_digest` | `raw_provenance.rs:250` | 봉인 다이제스트 | —(장치) | 없음 |
| `raw_extra` | `shape.rs:106` | 파싱된 필드 이후 추가 바이트 (라운드트립 보존용) | 순수 passthrough | 없음 |
| `raw_rendering` | `shape.rs:265` | 렌더링 정보 원본 바이트 (변환 행렬 등, 라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_list_header_extra` | `shape.rs:363` | LIST_HEADER 레코드의 파싱된 필드 이후 추가 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_trailing` | `shape.rs:607` | SC_LINE 끝 패딩/추가 바이트 (라운드트립 보존) | 순수 passthrough | 없음 |
| `raw_trailing` | `shape.rs:677` | SHAPE_POLYGON 끝 패딩/추가 바이트 (라운드트립 보존) | 순수 passthrough | 없음 |
| `raw_chart_data` | `shape.rs:825` | CHART_DATA 레코드 원본 바이트(라운드트립 보존용, 하위 태그 전체 병합) | 순수 passthrough | 없음 |
| `raw_tag_data` | `shape.rs:880` | OLE 레코드 원본 바이트 (라운드트립 보존) | 순수 passthrough | `raw_tag_seal`(shape.rs:885) — #4495 |
| `raw_tag_seal` | `shape.rs:885` | OLE `raw_tag_data` 무결성 봉인 | —(장치) | 없음 |
| `raw_data` | `style.rs:54` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_data` | `style.rs:105` | 원본 레코드 바이트 (라운드트립 보존용, 있으면 직렬화 시 우선 사용) | 편집이 씀 | 없음 |
| `raw_data` | `style.rs:323` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_data` | `style.rs:403` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_para_heads` | `style.rs:417` | HWPX `hh:paraHead` 영역 원본 XML — 10수준·전용 속성 보존 | 순수 passthrough | 없음 |
| `raw_data` | `style.rs:439` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_para_head` | `style.rs:459` | HWPX `hh:bullet` 자식 원본 구간 — 7수준으로 못 담는 속성 보존 | 순수 passthrough | 없음 |
| `raw_data` | `style.rs:488` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_data` | `style.rs:532` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_data` | `style.rs:560` | 원본 레코드 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
| `raw_ctrl_data` | `table.rs:69` | CTRL_HEADER ctrl_data의 4바이트(attr) 이후 추가 바이트 (라운드트립 보존용) | 편집이 씀 | `raw_ctrl_seal`(table.rs:75) — #4495 |
| `raw_ctrl_seal` | `table.rs:75` | `raw_ctrl_data` 무결성 봉인 | —(장치) | 없음 |
| `raw_table_record_attr` | `table.rs:77` | HWPTAG_TABLE 레코드의 원본 속성 값 (라운드트립 보존용, 0이면 재구성) | 편집이 씀 | 없음 |
| `raw_table_record_extra` | `table.rs:79` | HWPTAG_TABLE 레코드의 border_fill_id 이후 추가 바이트 (라운드트립 보존용) | 순수 passthrough | 없음 |
| `raw_list_extra` | `table.rs:151` | LIST_HEADER 레코드의 34바이트 이후 추가 바이트 (라운드트립 보존용) | 편집이 씀 | 없음 |
## 갱신 규약

이 표는 `tests/cases/issue_5890_raw_cache_inventory.rs` 가 강제한다 — `src/model/**` 에
`pub raw_*` 필드를 추가하면서 이 문서에 줄을 넣지 않으면 시험이 깨진다. 반대로 사라진 필드가
표에 남아 있어도 깨진다. 문서가 조용히 낡는 것을 막는 장치다.

필드를 추가·삭제할 때는 위치(`파일.rs:라인`)까지 함께 맞춘다.

## 소멸 경로

raw 필드가 하나씩 해석될 때마다 캐시는 하나씩 사라진다. 미해석 비트의 의미는 **추측하지 말고
한컴을 정답지로 잰다** — #6373 에서 "회전 0 이면 비트를 내린다"는 그럴듯한 가설이 오라클로
정반대(파싱값 보존이 정답)임이 드러난 전례가 있다. 도구는 `tools/hangul_rotation_oracle/`
(#6371)이고 `--survey` 는 한글 설치 없이 돈다.
