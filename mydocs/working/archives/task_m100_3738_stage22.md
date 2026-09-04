---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 22 — Square 그림의 다음 physical-page owner 복원

## 출발 근거

[Stage 21 visual sweep](task_m100_3738_stage21_visual_sweep.md)는 p154의 RowBreak tail과 p157–158의 clipped
table을 복원했지만, p155 rhwp에는 기준 PDF에 없는 분홍 흐름도 그림이 표 우하단부터 본문·각주 위로 겹쳐
나오는 것을 확인했다. 같은 source contract를 전체 HWP에 대조하자 p126의 그림 56도 같은 방식으로
각주 170–172 위에 겹치는 잠재 결함임을 확인했다.

기준 PDF p155에는 `(3) 평가 절차` 본문·BTS/OPTN 표·각주 211만 있고, 그림 64는 PDF p156의 우상단에
caption과 함께 있다. source `pi=1692 / ci=1`는 `Square`, `vert=Para(off=518)`,
`horz=Column(off=25139)`, `treat_as_char=false` 그림 64와 본문을 함께 가진 문단이다. native rhwp는 이
그림을 p155의 anchor 문단에 그대로 올려 표와 본문을 덮고 있었다.

그림 56(`pi=1355 / ci=0`)도 `Square`, `vert=Para(off=3566)`, `horz=Column(off=23057)`인 같은 형태다.
기준 PDF p126은 anchor 본문과 각주만, PDF p127은 그림 56·caption과 좁은 본문 wrap을 가진다. 변경 전
rhwp p126에서는 image `x=401.9, y=652.6, w=253.1, h=340.7`의 하단과 caption이 FootnoteArea
`y=898.3..1039.4`를 침범했다.

## 분석 계약

1. 그림 64가 `pi=1692`의 Square anchor일 때 reference PDF가 p156으로 보내는 정확한 physical-owner 조건을
   dump/render-tree/shape placement 경로로 확인한다.
2. 동일 조건을 source 전체에서 대조해 p1355 그림 56도 같은 결함임을 확인하고, 유사해도 next-page owner
   contract가 없는 그림은 제외한다.
3. anchor paragraph가 현재 쪽에 남아도 그림만 다음 page에 배치하되, out-of-flow 그림이 후속 본문의
   typeset height/vpos fit을 바꾸지 않도록 한다.
4. source contract에 한정된 최소 보정과 HWP5 focused regression을 만든다. 다른 Square 그림의 같은-page
   배치나 p154–158 RowBreak 표 fix를 넓게 바꾸지 않는다.
5. 수정 뒤 p126–127 및 p155–156 PDF direct visual sweep으로 그림 owner와 caption·본문·각주 흐름을 다시
   판정한다.

## Source contract와 제외 근거

모든 non-TAC `Picture + Square + Para + Column + Top/Left + flowWithText + bottom caption`를 대조하면 세
후보만 있다.

| anchor | 다음 문단 저장 계약 | 판정 |
| --- | --- | --- |
| `pi=384` | `pi=385`가 `vpos=12000`부터 시작 | next physical-page owner가 아니므로 제외 |
| `pi=1355 / ci=0` 그림 56 | `pi=1356` 첫 줄부터 `vpos=0`, `cs=0`, `sw=23057 == horizontalOffset` | p127 owner |
| `pi=1692 / ci=1` 그림 64 | `pi=1693`의 전폭 2줄 뒤 `vpos=0`, `cs=0`, `sw=25139 == horizontalOffset` | p156 owner |

즉, 다음 문단에 source가 저장한 narrow band가 (a) 즉시 시작하거나 (b) 전폭 tail 뒤 reset되는 경우만
후보로 삼는다. (a)는 다음 문단의 저장 advance 전체가 현 page의 각주 예약 뒤 남은 높이를 넘어야만
허용해, 현재 page에 그대로 들어갈 일반 side-wrap을 나중의 무관한 page break로 보내지 않는다.

## 구현

`src/renderer/typeset.rs`에 다음을 추가했다.

1. native HWP5·1단·non-TAC·`Square`·`Para/Column/Top/Left`·positive horizontal offset·bottom caption·기존
   각주·narrow reset을 모두 확인하는 `native_hwp5_square_picture_uses_next_page_owner` predicate.
2. 그림 frame에 image outer margin뿐 아니라 실제 caption height와 spacing도 포함해 현재 각주 영역에
   들어갈 수 있는지를 판정.
3. 다음 page가 만들어질 때 그림을 `page_start_square_pictures`에 보관한다. 즉시 `current_items`에 넣지 않아
   out-of-flow 그림이 p1356 뒤 문단의 height/vpos fit을 바꾸는 +1 page 회귀를 막는다.
4. 해당 page의 column flush에서만 이 그림을 첫 `PageItem::Shape`로 materialize한다. 따라서 layout은
   그림→caption→narrow text 순서로 렌더하지만, typeset 흐름은 기존과 동일하다.

초기 구현은 새 page의 `current_items`에 Shape를 바로 넣어 그림 56을 옮기기는 했으나 p1361의 허용 tail을
두 줄만 남기고 분할해 전체 페이지가 `219→220`으로 증가했다. 이 결과는 받아들이지 않고, 위의
flush-time materialization으로 수정했다.

## 코드 회귀 검증

다음 focused file을 새 review target에서 실행해 13/13 통과했다.

```text
CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment
```

- `native_hwp5_square_picture_uses_the_next_page_wrap_owner`: p155에는 그림 64 Image/caption이 없고, p156에
  `pi=1692/ci=1` Image 하나와 caption·p1693 continuation이 남는지 고정.
- `native_hwp5_square_picture_figure_56_uses_the_same_next_page_owner_contract`: p126에는 그림 56이 없고,
  p127에 `pi=1355/ci=0` Image 하나와 좁은 본문·caption이 남는지 고정.
- 기존 Stage 9–21 회귀 11개도 함께 통과했고 native page count는 다시 `219`다.

## 결과와 증적

코드 revision `a5612fd2534dbe5ccf4b85e330769213ce30f93c`를 고정한 뒤 p126·127·155·156을 한컴 PDF와
직접 대조했다. 그림 56·64는 각각 p127·p156에서만 확인되고, p126·p155에서는 표·본문·각주를 덮지
않는다. p155의 본문/각주 211 및 p156의 그림 64 caption·뒤 표의 physical owner도 기준 PDF와 같다.

### 후속 정정 — p127 그림 주변 본문 wrap

위 결과는 **그림 physical owner**에 한정한다. 이후 유지보수자 직접 비교에서 p127의 그림 56은 올바른
쪽에 있으나, 다음 문단 `pi=1356`의 저장 narrow wrap band(`cs=0`, `sw=23057HU`)가 다음-page 그림의
wrap anchor로 복원되지 않아 본문이 PDF와 다른 비정상 세로 열로 흐르는 결함이 확인됐다. 따라서 이
Stage는 그림 owner만 완료로 유지하고, p127 본문 flow는 Stage 23에서 독립 P0 결함으로 이월한다. 이
정정은 Stage 22가 p127 전체를 정상이라고 해석될 수 있는 표현을 제한한다.

완전한 실행 명령, PNG, 입력 해시, 자동 후보의 사람 판정은
[Stage 22 visual sweep](task_m100_3738_stage22_visual_sweep.md)에 고정했다. focused regression은 13/13
통과했고 native HWP page count는 219로 유지됐다.

## 다음 단계 이월

이 Stage는 두 Square 그림의 next-page owner만 해소한다. 기준 PDF 215쪽과 native HWP 219쪽의 전체
page-map 차이와 p43의 본문/각주 겹침을 포함한 독립 잔여 후보는 이 증적 commit 뒤 다음 Stage에서
현재 `devel` 기준으로 재현·원인 분석한다.
