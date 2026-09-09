# task_m100_4408 Stage 1 — 중첩 표 borderFillIDRef 사전 등록 누락

- **이슈**: [#4408](https://github.com/edwardkim/rhwp/issues/4408)
- **브랜치**: `test/roundtrip-hwp-hwpx-hwp`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 게이트 통과(타이밍 flake 제외), PR 게시
- **기록일**: 2026-08-10 KST

## 1. 발견 경위 — HWP → HWPX → HWP 왕복 스윕

`samples/` 400개 + `~/hwpdocs_10k/` 6,582개 = **6,982건** 전수 스윕에서 나왔다.
46건 실패 중 **1건이 크래시**였다 — 중첩 표를 가진 정부 보고서에서 `export-hwpx` 가 산출물 없이
전체 실패했다(`borderFillIDRef: [0]` 미등록).

나머지 45건은 빈 파일 24, HTML 오저장 12, DRM 6, 암호 3 이다.

스윕 자체의 결함도 하나 잡아 정정했다 — 임시 파일명에 원본 이름을 그대로 써서 macOS 255바이트
제한을 넘는 17개 문서가 "저장 실패"로 오분류됐다. 해시 경로로 재실행해 16 PASS + 1 IR_DIFF 로
정정했다.

## 2. 결함 — 최상위 표만 훑었다

`SerializeContext::collect_from_document` 가 `doc.sections[].paragraphs[]` 의 최상위
`Control::Table` 만 훑어, 셀 안에 중첩된 표나 글상자·머리말/꼬리말·각주/미주 안의 표의
`border_fill_id` 가 등록에서 빠졌다. 그 표가 실제 직렬화에서 참조되면
`assert_all_refs_resolved` 가 하드 실패해 **문서 전체의 export 가 산출물 없이 죽는다.**

`table_extract::collect_from_paragraph` 와 같은 위상의 재귀
(`register_border_fills_in_paragraphs`)로 교체했다. 깊이 상한은 `table_extract::MAX_NEST_DEPTH`
와 같은 값이다.

## 3. 리뷰가 잡은 것 — 문단 리스트 소유자 4종 누락

첫 수정은 소유자 **6개**만 돌았다(Table 셀·Shape 텍스트박스·Header·Footer·Footnote·Endnote).
OWPML 상 8개인데 빠진 것:

- **`Caption`**(`model/shape.rs:674`) — 표·도형·그림·차트·OLE 이 각각 갖는다
- **`Field.memo_paragraphs`**(`model/control.rs:276`)
- **`HiddenComment`**(`model/control.rs:235`)
- **`Control::Picture`** — 자체 `caption`(`model/image.rs:44`)

**이건 이번 세션에 이미 나온 패턴이다.** #4321(PR #4365)이 정확히 같은 누락을 injection_scan 에서
고쳤는데, 새 코드가 같은 자리를 또 빠뜨렸다.

캡션이 변형마다 다른 필드에 있다는 것도 반영했다 — 기본 6종은 `drawing.caption`,
`Group`/`Picture` 는 자기 `caption`, **`Chart`/`Ole` 는 자기 `caption`**(파서가 `.take()` 로 옮겨
`.drawing().caption` 이 항상 `None`, `parser/control/shape.rs:213`·`:222`). 방어적으로 양쪽 다
확인한다.

요청 범위 밖이지만 `GroupShape.children` 재귀도 함께 닫았다 — 안 닫으면 묶음에 담긴
Picture/Chart 의 캡션이 여전히 사각지대다.

## 4. 재현 — 실 코퍼스는 실패했다

전체 코퍼스를 `<hp:caption>…<hp:tbl` 패턴으로 스캔해 실제 문서 2건을 찾았다
(`samples/issue1891_external_bindata_link.hwpx`,
`hwpdocs_10k/.../36421964_고시문.hwpx`). **둘 다 첫 수정만으로 이미 통과한다** —
`border_fill_id` 가 우연히 전역 1-based 등록 범위 안에 들어서다.

**코드 경로상 누락은 실재하지만(단위 테스트가 직접 증명) 실 코퍼스 크래시 재현은 실패했다.**

## 5. 검증 (완료)

- 소유자별 단위 테스트 9건 + 실제 `serialize_hwpx` 를 물리는 end-to-end 1건.
  **전부 수정 전 코드로 되돌려 FAILED 를 확인**했다(e2e 는
  `XmlError("미등록 ID 참조 발견: borderFillIDRef: [121]")`).
- 최소 IR 픽스처에 기본 char_shape/para_shape/style 을 채워 border_fill_id 축과 무관한
  잡음(paraPrIDRef/styleIDRef 미등록)을 제거했다.
- `cargo test --profile release-test --tests` 2회 완주 — 타이밍 flake 1건 외 전부 통과.
- `cargo fmt --all -- --check`, `cargo clippy -- -D warnings` 통과.

## 6. 미처리

`src/serializer/hwpx/context.rs` 는 #4395 수정(PR #4409, `ctx.style_ids.register(0)` 한 줄)과
같은 파일이다. 논리 충돌은 없지만 머지 시 텍스트 충돌 가능성이 있다 — 나중에 머지되는 쪽이
리베이스한다.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
