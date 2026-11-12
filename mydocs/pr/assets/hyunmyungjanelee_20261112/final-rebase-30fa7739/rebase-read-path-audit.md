# 리베이스 후 소비 경로 확인

- 후보: `30fa7739f0340f2a248b22542c7dd36d9354f056`
- 새 base: `1a3ed4a7cd76701085a24ac81d30bc5fb31942c1` (#8956)
- 기존 시각 산출 source: `d66568ef247859a9c184546cad3ba42b06e806e1`
- source 충돌 없음. `src/renderer`, parser, serializer, crates, scripts, assets, ttfs, samples, pdf, Studio src의 Git tree가 전후 동일하다.

## 새 기능

`json_usize_opt` → `parse_path_entries(last_step_is_table=true)` → `parse_table_path` → 두 표 조회 API(`get_table_dimensions_by_path_native`, `get_table_cell_bboxes_by_path_native`) → 기존 WASM binding이다. 표 자체를 가리키는 마지막 단계만 `controlIndex`를 요구하며, 앞 단계의 칸 경로는 세 필드를 계속 요구한다. 일반 `parse_cell_path`는 false를 전달하므로 칸 조회·편집 경로의 기존 계약을 유지한다. 조회는 `&self`로 수행하며 파서·저장·조판 결과를 수정하지 않는다. WASM source 변경은 설명 주석이며 binding 이름은 같다.

새 Native API는 후보 4건·base 4건을 같은 최소 글꼴 조건에서 실행해 모두 통과했다. 실제 브라우저 바깥/중첩 표 조회·오류 경계 11건도 통과했다. `incoming-base-validation.json`과 `browser-runtime-result.json`에 명령·결과를 보존했다. 이후 #9017 리베이스의 별도 재사용 감사는 `rebase-a85d3265/README.md`를 따른다.

## 시각 증적

기존 전쪽 증적은 원 생산 SHA를 유지한다. 리베이스 후 새 Native/fresh WASM의 공개 입력 34종·46쪽과 Native 편람 gate를 다시 산출해 기존 출력과 대조한다. 정본 18종·1771쪽은 기존 source의 실제 전쪽 측정과 renderer/입력/글꼴/캡처 소비 경로 동일성 및 새 런타임 검증을 별도로 연결한다. 이전 TSV를 새 source에서 재측정했다고 표시하지 않는다.

sample10의 마지막 캡처 완료가 리베이스 시작 뒤였으나, 동결 JS/WASM을 사용했다. exporter가 HTTP로 공급하는 scripts와 Studio core JS 7개를 모든 replay commit에서 대조해 동일함을 확인했다. CanvasKit TS 모듈은 이 직접 Canvas2D exporter의 HTTP module map에 포함되지 않는다. `direct-exporter-rebase-overlap-audit.json`에 Git blob을 남겼다.
