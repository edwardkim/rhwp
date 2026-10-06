# 최종 보고 — Task M100-1271

## 이슈

- GitHub: [#1271](https://github.com/edwardkim/rhwp/issues/1271)
- 제목: HWPX 글뒤로 표 분할로 인한 바탕쪽 홀짝 밀림 수정

## 원인

대상 샘플 `samples/hwpx/[2027] 온새미로 1 본교재.hwpx` 의 앞부분에서
`Paper` 기준으로 배치된 `BehindText` 표가 본문 흐름 표처럼 분할되어 PDF 기준본에 없는
`PartialTable` 페이지를 만들었다.

그 결과 2쪽 MEMO, 3쪽 1주차 간지, 4쪽 본문 시작 대응이 한 쪽씩 밀릴 수 있었고,
구역 간 쪽번호 carry 전에 바탕쪽을 선택하던 기존 순서와 결합해 홀수/짝수 바탕쪽 선택이
최종 쪽번호와 달라질 수 있었다.

## 변경

- `src/renderer/typeset.rs`
  - `Paper` 기준의 배경성 `BehindText`/`InFrontOfText` 표는 본문 흐름을 밀지 않는
    shape 경로로 유지하도록 보정했다.
  - #992 성격의 oversized multirow table 분할 정책은 paper-anchored overlay table 을
    제외한 나머지에 유지했다.

- `src/document_core/queries/rendering.rs`
  - 바탕쪽 선택 로직을 `assign_master_pages_for_section` helper 로 분리했다.
  - 구역 간 page number carry 보정 이후 최종 `page_number` 기준으로 `Odd`/`Even`
    바탕쪽을 선택하도록 호출 시점을 변경했다.
  - 기존 첫 쪽 바탕쪽 감추기, 마지막 쪽 확장 바탕쪽, `overlap`/`replace_base` 처리는 유지했다.

- `tests/issue_1271_hwpx_behind_text_table.rs`
  - 대상 HWPX 샘플의 앞부분 페이지 대응을 고정하는 통합 테스트를 추가했다.

## 검증

통과:

```text
cargo fmt --check
cargo test --lib
cargo test --tests
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
```

주요 확인:

- 기준 PDF와 HWPX 렌더가 모두 46쪽이다.
- 2쪽은 `MEMO`, 3쪽은 `1주차`, 4쪽은 `section=1, page_num=4` 본문 시작이다.
- 4쪽은 짝수쪽 바탕쪽, 5쪽은 홀수쪽 바탕쪽 방향과 일치한다.
- `issue_703`, `issue_775`, `issue_1156_rowbreak_fragment_fit`, `issue_1197_svg_object_zorder`
  등 관련 회귀 테스트가 통과했다.

## rhwp-studio 시각검증

현재 수정본을 반영한 WASM 을 빌드했고, rhwp-studio 서버를 실행했다.

```text
http://127.0.0.1:7700/
```

대상 샘플 자동 로드 URL:

```text
http://127.0.0.1:7700/?url=/samples/hwpx/%5B2027%5D%20%EC%98%A8%EC%83%88%EB%AF%B8%EB%A1%9C%201%20%EB%B3%B8%EA%B5%90%EC%9E%AC.hwpx&filename=%5B2027%5D%20%EC%98%A8%EC%83%88%EB%AF%B8%EB%A1%9C%201%20%EB%B3%B8%EA%B5%90%EC%9E%AC.hwpx
```

## 남은 판단

- 로컬 테스트와 대상 샘플 검증 기준에서는 수정이 유효하다.
- Docker daemon 이 실행 중이 아니어서 Docker 기반 WASM 빌드는 수행하지 못했고,
  시각검증용 WASM 은 로컬 `wasm-pack build --target web` 으로 갱신했다.
- 작업지시자 시각검증 후 PR 작성/푸시 단계로 진행하면 된다.
