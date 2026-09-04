# 구현계획서 — #3486 HWP3 기준 PDF 대비 구조·기하 보정

## 현재 구현 경계

Stage 2는 확인된 원본 구조 손실만 다룬다. HWP3의 추가 정보 블록 #6(쪽 배경), HWP3 조합형의
회사명 PUA, 문단 간격 저장 단위, 인라인 개체의 UTF-16 stream 점유 길이가 공통 IR 계약과 달라진
경우를 복원한다. HWPX 비교 문서에서 같은 의미가 보이는지 확인하되, HWPX parser 자체를 이 단계에서
변경하지 않는다.

| 파일 | 변경 계약 |
| --- | --- |
| `src/parser/hwp3/mod.rs` | 추가 정보 블록 #6의 내장 이미지와 효과값을 안전하게 읽어 공통 `BorderFill` 이미지 채우기로 정규화한다. 문단 앞뒤 간격과 인라인 개체의 stream 좌표는 HWP5 저장 계약과 맞춘다. |
| `src/parser/hwp3/johab.rs` | 한글 97 원본에서 실제 확인한 회사명 조합형 코드 여섯 개만 공통 Hancom PUA로 보존한다. |
| `src/renderer/hancom_pua.rs` | 확인 근거가 있는 PUA의 paint-time 표준 문자 투영 표를 한 곳에 둔다. 미확인 PUA와 원문 IR은 바꾸지 않는다. |
| `src/renderer/composer.rs` | 공통 PUA 표를 통해 fallback text를 만든다. |
| `src/renderer/layout/paragraph_layout.rs` | 확인된 `DISTRIBUTE_SPACE` 회사명+inline logo 줄만 trailing space에 남은 폭을 배분한다. 일반 나눔 정렬에는 적용하지 않는다. |
| `tests/hwp3_password_fixture.rs` | 실제 HWP3 fixture의 회사명 PUA·쪽 배경·열기 계약을 보호한다. |
| `tests/issue_3486_hancom_pua_display.rs` | PUA 투영 표의 확인된 값과 미확인 인접값 보존을 보호한다. |

## Stage 3 후보와 배제 기준

중앙 그림이 기준 PDF보다 옅거나 위치가 다른 현상은 Stage 2의 쪽 배경 보존만으로 수용하지 않는다.
`src/renderer/svg.rs`, `src/renderer/web_canvas.rs`, `src/renderer/render_tree.rs`의 이미지 효과 적용을
다음 순서로 대조한다.

1. parser가 읽은 brightness/contrast/effect/fill mode가 HWPX 비교 문서와 같은 의미인지 확인한다.
2. SVG와 Canvas가 해당 IR 값을 같은 순서와 범위로 적용하는지 확인한다.
3. 원인을 하나로 좁힌 경우에만 공통 renderer 보정을 한다. 문서별 watermark 상수나 화면 캡처만 맞추는
   예외는 추가하지 않는다.
4. 전용 HFT·폰트 glyph가 없는 데서 비롯된 글자 모양 차이는 renderer 색조 결함과 섞지 않는다.

### 확인된 원인과 다음 구현 경계

HWPX `Contents/header.xml`의 `bright="50" contrast="-15"`는 HWP5 legacy `ImageFill` raw 저장
순서에 맞춰 `(brightness=-15, contrast=50)`으로 모델에 보존한다. 이는 구조/round-trip의 올바른
값이다. 그러나 현재 SVG와 Web Canvas는 그 raw 순서를 화면 의미로 그대로 넘겨 두 포맷 모두
`rhwp-img-bc-b-15c50` 필터를 생성한다. 따라서 실제 화면 의미는 `(50, -15)`이어야 한다.

다음 구현은 `PageBackgroundImage`의 명시적 display 변환을 한 곳에 두고 SVG·Web Canvas·Skia가 그
값을 공통으로 사용하게 한다. `ImageNode`의 일반 그림 속성은 다른 parser 계약을 가지므로 이 변경에
포함하지 않는다. `is_watermark()`와 REAL_PIC preset 판정은 raw 계약에 의존하므로, 색조 변환과
함께 임의로 바꾸지 않고 기준 PDF 대조로 opacity를 별도 판정한다.

## 회귀·시각 검증 계약

- 구조 변경마다 가장 가까운 HWP3 parser/renderer focused test를 먼저 추가하거나 갱신한다.
- 한컴 PDF↔rhwp 비교는 자동 정확도 수치를 품질 후보로만 쓴다. 실제 수용 증적에는 페이지 번호가
  표시된 before/after/overlay PNG와 남은 차이의 설명을 포함한다.
- password·MCP token·환경 파일 내용은 테스트, 단계 보고, commit, PR 본문에 기록하지 않는다.
