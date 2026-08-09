# task_m100_4388 Stage 1 — HWPX 직렬화의 조용한 컨트롤 손실

- **이슈**: [#4388](https://github.com/edwardkim/rhwp/issues/4388)
- **브랜치**: `fix/issue-4388-hwpx-silent-drops`
- **분기 기준**: `upstream/devel` (0 behind)
- **상태**: 게이트 전부 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 이슈 진술의 정정

원 이슈는 도형 곡선 속성을 `arcType` 으로 적었으나 OWPML 상 실제 속성명은 **`type`** 이다
(`ParaList XML schema.xml`). 또 "HWP5 는 왕복이 된다"고 적었는데 확인 결과 HWP5 축도 별개 결함이
있었다 — 아래 4절.

## 2. 고친 것

1. **`hp:arc` 의 `type` 속성** — 파서가 읽지 않고 직렬화기가 쓰지 않아 곡선/부채꼴 구분이
   HWPX 왕복에서 사라졌다. `parser/hwpx/section.rs` 와 `serializer/hwpx/section.rs` 양쪽에 추가.
2. **조용한 손실에 경고** — 직렬화기가 표현할 수 없는 컨트롤을 만나면 아무 말 없이 버렸다.
   `warn_if_unrepresentable_in_hwpx` 헬퍼를 만들어 두 지점에서 호출한다.

## 3. 리뷰가 잡은 것 — 슬롯 등록과 경고를 분리해야 했다

첫 수정은 `Control::Hyperlink`/`Control::Unknown` 을 `is_hwpx_inline_slot` 에 등록해 손실을
드러내려 했다. **이것이 무관한 회귀 테스트 `bookmarks_survive_saving_to_hwp5` 를 깨뜨렸다.**

`is_hwpx_inline_slot` 은 소비처가 둘이다 — `render_runs`(HWPX 슬롯 위치 축)와
`roundtrip::diff_documents`(**포맷 비종속** IR 비교로, `convert`(HWP5 대상) `--verify` 도 그대로
재사용한다). Hyperlink 를 등록하자 `diff_documents` 가 HWP5 경로의 **기존** 하이퍼링크 손실을
새로 검출하기 시작했다. HWPX 축 수정이 HWP5 축 테스트를 깬 것이다.

등록을 되돌리고, 경고는 `is_hwpx_inline_slot` 과 무관하게 두 분기
(`render_control_slot` 의 catch-all, `render_runs` 의 필터 배제 지점) 모두에서 직접 호출하도록
바꿨다. 재등록을 막는 회귀 가드를 **반대 방향 단언**으로 넣었다 —
`issue4388_diff_documents_{hyperlink,unknown}_not_compared_as_control` 는 "diff 가 검출하면
안 된다"를 확인해 누가 다시 등록하면 즉시 RED 다.

## 4. 이 작업에서 고치지 않은 것

`hwp3-sample16.hwp` 의 손실 4건(`s0` 문단 27/32/196/657)은 덤프 결과 **전부
`Control::Hyperlink`** 였다. `Control::Unknown` 은 이 문서에 하나도 없었다.

원인은 `serializer/body_text.rs:850` 의 `Control::Hyperlink(_) => (0x000B, 0)` 다. `ctrl_id` 가
0 이라 `serializer/control.rs:177` 이 CTRL_HEADER 를 아예 안 만드는데 본문에는 0x000B 가
들어간다 — 뒤따르는 컨트롤 짝짓기가 한 칸씩 밀린다.

**HWP5 직렬화기의 기존 결함이고 이 이슈 범위(HWPX 축) 밖이라 손대지 않았다.**
[#4424](https://github.com/edwardkim/rhwp/issues/4424) 로 따로 열었다. 같은 arm 의
`Control::Ruby` 는 #4397 이다.

## 5. 검증 (완료)

- `bookmarks_survive_saving_to_hwp5` 2/2 통과.
- `issue4388_*` 4종(신규 2 + 기존 2 축소) 전부 통과, 경고 stderr 출력 확인.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests --no-fail-fast` —
  바이너리 498개 전부 `test result: ok`, **FAILED 0건**.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.

## 6. 미처리

GitHub Actions, 작업지시자 승인, merge.
