# Task M100 #4969 W10-Q4 — vertical metrics·writing mode 인수 감사

- **이슈**: [#4969](https://github.com/edwardkim/rhwp/issues/4969)
- **상위 계획**: [`task_m100_4969.md`](../plans/archives/task_m100_4969.md)
- **기계 판독 결과**:
  [`w10_q4_handoff_audit.json`](../tech/investigations/issue-4969/w10_q4_handoff_audit.json)
- **수정 수행계획**: [`task_m100_4969_w10_q4.md`](../plans/archives/task_m100_4969_w10_q4.md)
- **기준 HEAD**: `4e0a3822c3e05c2df8b7c52740b3a0c79487053a`
- **최신 upstream/devel**: `a71ebf2141ae3bd1ca3b5d3ea438bbe3454edaf9` (`34 ahead / 0 behind`)
- **감사일**: 2026-08-30 KST
- **판정**: `revised-plan-required`, 제품 source 변경 0

## 결론

Q4는 blocked 상태는 아니다. 추적된 공개 TTF/OTF에 vertical table이 있고, standalone
shaper는 top-to-bottom 요청과 `vhea`·`vmtx`·`VORG` capability를 구분한다. 그러나 현재 테스트는
vertical table의 **존재**만 검증하고 세로 glyph id·cluster·advance·offset을 정답으로 고정하지 않았다.
반면 제품의 표 셀·글상자 세로쓰기는 font size 기반 advance, Unicode 세로형 치환, 90° 회전
휴리스틱을 사용한다. 두 경로를 즉시 연결하면 "horizontal advance를 회전해 근사하지 않는다"는
보호 불변식을 증명할 수 없다.

더 큰 선행 결손은 문서 의도의 의미 손실이다. HWP5 표 셀은 `text_direction=1`(영문 눕힘)과
`2`(영문 세움)를 구분하지만 HWPX 파서·직렬화기의 표 셀 경로는 현재 `VERTICAL`을 `1`로만
취급한다. HWPX 글상자는 `VERTICAL`/`VERTICALALL`을 직렬화용 `vertical_all` boolean으로는
보존하지만 layout에서는 둘 다 `list_attr=1`로 합쳐진다. 이 경계를 typed intent로 고정하지 않고
`vert`/`vrt2`를 적용하면 서로 다른 문서 의도를 같은 조판으로 만들 수 있다.

따라서 Q4는 exact vertical oracle, typed document intent, dormant layout transaction, 한 개 표면의
bounded activation 순서로 다시 나눈다. 공개 근거로 의미를 판정할 수 없을 때만 Hyper-V 한컴
oracle을 별도 승인 경계에서 사용한다.

## 1. exact source readiness

| face | bytes | SHA-256 | vertical capability | 판정 |
| --- | ---: | --- | --- | --- |
| `ttfs/opensource/NotoSansKR-Regular.ttf` | 2,519,996 | `6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74` | GPOS, GSUB, vhea, vmtx; VORG 없음 | Q4 exact TTF 사용 가능 |
| `ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf` | 456,688 | `2f86ef9a52acb6d1dad9d915843239123b635d97edd88fd0573a88ffcb4e16f1` | CFF, GPOS, GSUB, vhea, vmtx, VORG | origin 포함 exact OTF 사용 가능 |
| `ttfs/redistributable/happiness-sans/HappinessSansVF.ttf` | 1,503,064 | `3bbd254dcc5780f7524f9d07af4aa981ba5e3e84cf32d7d4e04301b3943e8694` | vhea/vmtx 없음 | 음성 control, fail-closed 필수 |

Q0 inventory에는 Source Han·Happiness의 초기 WOFF2 상태가 남아 있지만, Q1·Q3에서 공개
OTF/TTF source가 추가됐다. Q4-A에서 inventory의 source-readiness 항목을 현행화하되 초기
container 판정 이력은 삭제하지 않는다.

## 2. 이미 있는 capability와 없는 oracle

`src/renderer/shaping.rs`는 다음을 이미 보존한다.

- `ShapingDirection::TopToBottom`
- `ShapingWritingMode::{VerticalRl, VerticalLr}`
- top-to-bottom과 vertical writing mode의 상호 일치 검증
- `vhea`+`vmtx` 부재 시 `VerticalMetricsUnavailable`
- `has_vertical_metrics`, `has_vorg` capability
- rustybuzz 결과의 `x_advance`, `y_advance`, `x_offset`, `y_offset`

`tests/cases/issue_4969_shaping_request_contract.rs`는 Noto·Source Han의 capability acceptance와 Happiness
rejection을 검증한다. 그러나 top-to-bottom 결과의 glyph id·cluster·y advance·offset,
default `vert`/`vrt2` 적용 여부를 고정한 테스트는 없다. Q4-A는 이 빈칸을 oracle contract로 고정하고
제품 caller를 열지 않는다.

## 3. 문서 의도 계보

| 표면 | 입력 표현 | 현재 IR | 손실·주의점 |
| --- | --- | --- | --- |
| HWP5 표 셀 | LIST_HEADER bit 16..18 | `Cell.text_direction: u8` | 0/1/2 실제 값을 보존함 |
| HWPX 표 셀 | `HORIZONTAL`/`VERTICAL` | `Cell.text_direction: u8` | 파서는 `VERTICAL=1`, serializer도 `1`만 VERTICAL로 방출; HWP5 값 2의 x2x 의미 정책 미확정 |
| HWPX 글상자 | `HORIZONTAL`/`VERTICAL`/`VERTICALALL` | `list_attr`+`vertical_all` | roundtrip 문자열은 보존하지만 layout 코드는 두 vertical 값을 모두 1로 합침 |
| 구역·바탕쪽 | `HORIZONTAL`/`VERTICAL` | `text_direction: u8` | binary flag로 보존; 표 셀·글상자와 같은 activation으로 간주하면 안 됨 |

`samples/table-004.hwp`는 `text_direction=2` 표 셀 3건을 유지한다. 공개
`samples/issue6029/3200477_icao_procedure.hwpx`는 `VERTICAL` 표 셀 3건과 한 열 문자 보존 회귀를
제공한다. 후자의 문자열은 한글·괄호 위주이며 exact font source와 vertical metric 정답을 등재하지
않으므로, 현재 legacy 배치의 무회귀 control이지 exact shaping oracle은 아니다.

## 4. 현재 제품 경로의 한계

- `layout_vertical_cell_text`와 `layout_vertical_textbox_text_with_paras`는 문자를 직접 나누고
  `font_size` 또는 `font_size * 0.5`를 advance로 사용한다.
- 구두점은 `vertical_substitute_char`로 Unicode compatibility form으로 바꾸거나
  `is_vertical_rotate_char`로 90° 회전한다.
- Latin 회전 여부는 `text_direction==1` 비-CJK 문자를 기준으로 한다.
- horizontal shaping lowerer·text-v2는 nonzero y positioning을
  `VerticalPositioningAuthorityPending`으로 거부한다.

이 거부는 아직 제거할 장애물이 아니라 horizontal product lane을 지키는 보호 불변식이다. Q4에서는
vertical 전용 transaction과 좌표 계약이 증명된 최초 subset에서만 별도로 연다.

## 5. 수정 필수 항목

1. capability acceptance와 exact vertical output oracle을 분리한다.
2. raw `u8`을 바로 shaping mode로 넘기지 않고 표면별 typed intent adapter를 먼저 고정한다.
3. HWPX `VERTICALALL`의 영문 세움 의미를 공개 HWPX/HWP 쌍·PDF·스키마로 먼저 판정한다.
4. vertical glyph measurement, line/column progression, bbox, next origin, hit-test를 하나의 dormant
   transaction으로 검증한다.
5. 최초 activation은 exact source와 문서 의도가 둘 다 증명된 한 표면·한 subset만 허용한다.
6. 기존 표 셀·글상자 fixture는 legacy output을 조용히 바꾸지 않도록 control로 남겨 둔다.

## 다음 승인 경계

수정 수행계획은 2026-08-30 KST 메인테이너 승인을 받았다. 현재 경계는
인수·계획 checkpoint commit 승인이다. 해당 commit 전에는 Q4 product source·test source를
변경하지 않는다. commit 후 첫 절편은 Q4-A exact vertical oracle contract이며,
Hyper-V·private corpus·GitHub comment·push·PR을 사용하지 않는다.
