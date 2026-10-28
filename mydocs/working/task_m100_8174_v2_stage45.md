# #8174 v2-S45 — 그림·도형과 내부 글상자의 저장 예약

Issue: #8174. 2026-10-26, 기준 `aa791c48bb`.

PDF가 생산하는 Picture와 Rectangle/Line/Polygon의 HWP 저장 payload를 같은 요청 Storage에
연결했다. Ellipse/Arc/Curve/Chart도 같은 primitive producer를 소비한다. 일반 저장은 같은
encoder를 Unbounded로 호출한다. **T04 전체와 PR 준비 완료는 아니다.**

## 실제 경로와 유지한 계약

`BodyText → try_serialize_control → try_serialize_picture_control/try_serialize_shape_control`
경로에서 공통 header 결합, SHAPE_COMPONENT·종류별 payload, CTRL_DATA 사본과 record 배열을
실제 생성·증가 전에 예약한다. 캡션과 글상자의 LIST_HEADER뿐 아니라 내부 문단·표에도 같은
Storage를 재귀 전달한다. counting 단계에는 문단 저장 같은 부수효과를 실행하지 않는다.

- 그림의 밝기/명암 순서, border attr, crop·imgDim·원본 extra와 투명도 바이트의 기존 의미를
  유지한다. 원본 extra 전체를 복사한 뒤 마지막 바이트를 고치는 대신 borrowed span과 수정
  바이트를 같은 순서로 기록한다. 짧은 extra에 투명도 확장을 붙이는 기존 분기도 유지한다.
- SHAPE_COMPONENT의 raw/생성/명시 행렬과 group depth, 선·연결선·다각형/곡선의 trailing,
  단색·이미지·그라데이션 채우기와 alpha를 같은 순수 writer로 내보낸다. 행렬의 중간 Vec도
  제거했다. 좌표·형상 규칙을 바꾸지 않았다.
- CTRL_DATA의 그림·도형 자식 level+2 소유, 자동 생성 사각형의 top-level level+1 및 자식
  이중 기록, 캡션의 SHAPE_COMPONENT 선행 순서를 보존한다. 일반 control의 별도 CTRL_DATA
  삽입 분기에 다시 넣지 않는다. 원본 글상자 tail과 기본 13바이트 tail도 유지한다.
- Group/OLE의 기존 저장·내부 scratch는 이번 절편에서 전체 예약을 입증하지 않았다. S44의
  실제 PDF constructor 대조와 census에서 생산되지 않은 종류이며 일반 API의 지원은 유지한다.
  이 경로의 `bytes(0)`을 payload 예약으로 세거나 전체 allocation proof로 발표하지 않는다.

## 검증

생산 source와 제품 hash, 실행 명령·로그는 ignored
`output/pr-review/issue8174/v2-s45-20261026/`의 `production-freeze.json`,
`native-producer.json`, `checks.json`에 보존한다.

| 항목 | 실제 결과 |
| --- | --- |
| 그림·도형 wire 대조 | **1,264개 조합 bytes 동일**. S44의 실제 librhwp와 후보의 공개 serializer 호출을 비교 |
| 조합 범위 | 명시/자동 CTRL_DATA, 캡션·글상자, 원본 extra 길이·투명도, 6개 채우기/6개 행렬 조건과 primitive 종류 |
| 기존 serializer unit | **589 PASS**, 기대값 변경 없음 |
| 공개 요청/저장/wire 계약 | **16 PASS** |
| 실물 BodyText 재생성 | raw stream 재사용을 끈 9개·**34,600개 레코드 bytes 동일** |
| PDF 실제 저장·재열기 | 17개·**34개 저장/98쪽 재열기**, 상세 진단 3개·선택 쪽 추가 4개 왕복 유지 |
| 실제 저장 파일 | 일반/상세 **37개 bytes가 S44와 동일** |
| CLI 한도 실패·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 기존 파일·stdout 보존 및 새 요청 정상 복구 |
| 새 Native 시각 | CFF **100.00000%**, Type3 11쪽 **99.97900%**. 새 review/overlay가 S44와 bytes 동일 |
| Rust 검사 | Native/WASM Clippy `-D warnings`, feature-off 통과. 이번 절편의 unit 정책은 **4,134 tests / 304 modules**, 증가 없음 |
| 문서 | 변경 문서 3개 상대 링크·diff 검사 통과 |

새 review를 직접 열어 CFF 글자·기호·간격과 Type3의 26–30번 행, 그림 9·10의 셀 내부 표시·
캡션·앞뒤 본문·괘선 관계를 대조했다. 독립 PDF와의 미세 래스터 차이는 기존과 같으며 점수로
배치 누락을 면제하지 않았다. 1,264개 wire 대조는 합성 metadata 호환성 검사이지 한컴 시각
정답이나 수정 전 실패의 증거가 아니다. 새 px/golden·source-side 테스트는 추가하지 않았다.

37개 실제 CLI 보고의 decoded 논리 작업량도 대조했다. 17개 HWP 저장에서 추가 계상이
관측됐고 출력 bytes는 같았다. CFF는 1,669,057 → 1,671,886, 전체 최대 증가분은
28,383,144바이트다. `decoded-work-delta.json`에 보존하며 이를 heap/RSS 증가량이나 성능
측정으로 해석하지 않는다. 프로필·명시적 host cap은 올리지 않았다.

초기 후보를 검토하던 중 일반 저장 wrapper가 사각형의 자동 생성 CTRL_DATA를 이미 제공된
값으로 넘겨 top-level 기록을 누락할 수 있음을 발견했다. 진행 중인 해당 제품 빌드를 중단하고
원래 Option을 공유 producer에 넘기도록 보정한 뒤 다시 빌드했다. 중단 로그·이유를 보존했고,
위 1,264개 조합과 실제 결과는 보정 후 제품의 실행이다.

전체 fmt 검사에서는 이전 source unit 이관 뒤 남은 `font_metrics_data.rs`·`mini_cfb.rs`·
`record_writer.rs`의 테스트 모듈 빈 줄이 검출됐다. 제품·회귀 뒤 해당 빈 줄만 정리하고 fmt를
재실행했다. `test-format-only-after-product.json`은 세 파일의 모든 비공백 줄과 production
prefix가 그대로임을 검증한다. 제품 생산 시점의 full-file source hash와 이 테스트 공백 차이를
구분하며 새 생산 코드 실행으로 표시하지 않는다. 전체 PR의 기존 unit 이관 차단은 유지한다.

## 통합 잔여

Core pagination/중첩 composition, adapter/HWPX XML/context와 나머지 실제 할당 경계,
T05 복사/재열기, T06 의존성·JPEG 및 T07–T09 최종 포장·전체 회귀·시각/성능·base/PR 준비는
남는다. 현재 source의 최적화 fresh WASM/Studio 실행은 아직 없으며 S37 결과로 대체하지
않는다. `parserAllocationBoundsProven=false`와 host 책임을 유지한다.
