---
kind: working
status: active
issue: 6868
---

# HWPX→HWP5 저장이 누름틀 필드의 끝 표시를 잃는다 (#6868)

작업 브랜치: `fix/6868-hwp5-orphan-field-end`
대상: `src/parser/hwpx/section.rs` · `src/document_core/helpers.rs`

## 한 줄

다단락 누름틀이 **중첩 문단 목록**(글상자 subList·표 칸·머리말·각주) 안에서 열리고
닫히면, 그 종료 마커가 짝을 못 찾아 `begin_ctrl_id = 0` 으로 남고 HWP5 저장기가
**끝 표시를 통째로 버린다.** 끝이 없는 누름틀은 문단 나머지를 필드 안으로 삼킨다.

## 이슈가 요구한 것

- `x2h` 산출에서 사라지는 필드 끝 표시를 되살린다(11문서 19개).
- `x2x` 는 이미 무결하므로 그 경로를 깨지 않는다.

## 원인 — 직렬화기가 아니라 파서의 링크 단계

이슈는 "잃는 것은 **HWP5 직렬화기 한 곳**"으로 결론했다. `x2x` 가 무결하고 `x2h` 만
잃는다는 관찰은 정확하지만, 실제 결손은 그 앞단이다.

`src/parser/hwpx/section.rs`:

```rust
    link_orphan_field_ends(&mut section.paragraphs);   // ← 최상위 목록에만 걸린다
```

`link_orphan_field_ends` 는 **구역 최상위 `section.paragraphs`** 에만 걸리고 중첩
문단 목록에는 내려가지 않는다. 그 목록의 종료 마커는 `begin_ctrl_id` 가 0 으로 남는다.

HWP5 저장기(`src/serializer/body_text.rs`)의 방출 지점은 **둘 다** 그 값을 요구한다.

```rust
.filter(|o| o.char_idx == i && o.begin_ctrl_id != 0)          // 본문 중간
.filter(|o| o.char_idx >= text_chars.len() && o.begin_ctrl_id != 0)  // 마지막 문자 뒤
```

주석이 밝히듯 "`begin_ctrl_id` 가 0 이면 필드 종류를 모른다는 뜻이라 내지 않는다" —
저장기의 판단은 옳고, **0 을 남긴 파서가 원인**이다. `x2x` 가 무결한 것도 같은 이유다:
HWPX writer 는 `begin_id_ref` 만 쓰면 되고 `begin_ctrl_id` 를 요구하지 않는다.

### 최소 픽스처로 확증

`36414761_결재문서본문.hwpx`(누름틀 2개: '제목'·'본문').

```text
  SRC        fieldBegin=2 fieldEnd=2   end refs=[1267903547, 1553095123]
  x2x        fieldBegin=2 fieldEnd=2   (파서·HWPX writer 무결)
  x2h→back   fieldBegin=2 fieldEnd=1   end refs=[1553095123]      ← '제목' 이 사라진다
```

임시 프로브(커밋 안 함)로 IR 을 찍으니 **'제목' 필드가 최상위 문단 목록에 아예 없었다** —
글상자 subList 안에서 열리고 닫힌다. 최상위에만 있는 '본문' 은 살아남는다. 두 필드의
차이는 종류·정렬·위치가 아니라 **어느 문단 목록에 사는가** 하나다.

## 수정

목록마다 **독립적으로** 잇는 재귀를 넣었다. 컨테이너 목록은 `injection_scan` 의 방문자와
같은 것을 본다 — 표 칸·표 캡션·글상자·도형 캡션·그림 캡션·각주·미주·머리말·꼬리말·숨은 설명.

```rust
fn link_orphan_field_ends_recursive(paragraphs: &mut [Paragraph])
fn link_orphan_field_ends_in_control(control: &mut Control)
```

바깥 목록의 열린 필드를 중첩 목록으로 **물려주지 않는다** — 필드는 컨테이너 경계를 넘지
못하므로, 물려주면 글상자 안 종료 마커가 바깥 필드를 닫는 짝으로 잘못 묶인다. 회귀
가드 테스트로 고정했다.

`get_caption_from_shape_mut` 은 기존 `get_caption_from_shape` 의 가변 짝이다 — 변형별
캡션 자리 판정(#4321)이 미묘해 파서에 복제하지 않고 헬퍼 옆에 두었다.

## 검증 실측

### 코퍼스 스윕 (필드를 가진 HWPX 상위 120건)

`x2h` 왕복 뒤 `hp:fieldEnd` 수를 원본과 견줬다. 기준 바이너리는 같은 base commit.

| | 문서 | 마커 |
|---|---|---|
| 수정 전 손실 | 9건 | 16개 |
| 수정 후 손실 | 1건 | 1개 |
| 개선 | **8건** | **15개** |
| 악화 | **0건** | 0개 |

이슈 표의 문서와 필드 수·손실 수가 일치한다.

| 문서 | 필드 | 손실 | 이슈 표 |
|---|---|---|---|
| 위생용품 관리법 시행규칙 규제영향분석서 | 67 | 5→0 | `03755` 67→62 (5) |
| 농업기계화 촉진법 시행규칙 | 50 | 2→0 | `03627` 50→48 (2) |
| 화장품법 시행규칙 | 25 | 2→0 | `03756` 25→23 (2) |
| 먹는물관리법 시행규칙 | 18 | 2→0 | `03873` 18→16 (2) |
| 모자보건법 시행규칙 | 33 | 1→0 | `03708` 33→32 (1) |
| 공공재정 부정청구 금지법 | 31 | 1→0 | `03506` 31→30 (1) |
| 36414761 결재문서본문 | 2 | 1→0 | `08799` 2→1 (1) |
| 36310909 결재문서본문 | 2 | 1→0 | (표 밖) |

### 회귀 테스트

`src/parser/hwpx/section.rs` 테스트 모듈에 둘을 더했다 (`task1556_*` 옆).

- `issue6868_orphan_field_end_links_inside_nested_paragraph_lists` — 각주 subList 안
  다단락 필드의 종료 마커가 `begin_ctrl_id` 를 얻는다.
- `issue6868_nested_list_does_not_borrow_outer_open_field` — 바깥 목록의 열린 필드를
  중첩 목록이 짝으로 훔치지 않는다(`begin_ctrl_id == 0` 유지).

## 게이트

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | OK (수정 없음) |
| `node scripts/rust-unit-test-tiers.mjs --check` | 4207 tests / 298 modules |
| `cargo clippy --locked -- -D warnings` | OK |
| WASM32 Clippy · workspace build · workspace all-targets Clippy | OK |
| `node scripts/rust-test-suite-manifest.mjs --check` | 48/48 targets |
| `cargo test --release --workspace` | (본문에 실측 기재) |

## 남긴 것 — 구역 경계를 넘는 필드

스윕에서 `36455713_결재문서본문.hwpx` 1건이 수정 후에도 1개를 잃는다. **악화가 아니라
미해결**(전 1 → 후 1)이고, 원인이 다르다.

```text
  Contents/section0.xml   fieldBegin 1   fieldEnd 0
  Contents/section1.xml   fieldBegin 2   fieldEnd 3
```

필드가 **구역 경계**를 넘는다. `link_orphan_field_ends_recursive` 는 구역 파싱
(`parse_hwpx_section`) 안에서 구역별로 걸리므로 이 형상에는 닿지 않는다. 문서 단위
링크가 필요한 별도 축이라 이 PR 에서 열지 않았다.

HWP5 파서(`src/parser/body_text.rs`)의 `link_orphan_field_ends` 도 같은 최상위 한정
형상이다. `h2h` 경로의 동일 결함 여부는 이 이슈의 증거 범위 밖이라 확인만 남긴다.
