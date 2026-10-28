# Task M100 #1387 구현계획서 — HWPX serializer 표 캡션(hp:caption) 직렬화

- 수행계획서: `mydocs/plans/task_m100_1387.md` (승인 완료)
- 브랜치: `local/task1387`
- 작성일: 2026-06-12
- 단계: 4단계

## 0. 사전 조사 확정 사항

### 0.1 원본 XML 실물 (ta-pic-001-r — 한컴 생산)

```xml
<hp:caption side="BOTTOM" fullSz="0" width="8504" gap="850" lastWidth="47624">
  <hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" …>
    <hp:p …>…&lt;그림 <hp:ctrl><hp:autoNum …/></hp:ctrl>&gt; 의정활동 …</hp:p>
  </hp:subList>
</hp:caption>
```

- 속성 순서: `side, fullSz, width, gap, lastWidth` (한컴 실물 기준)
- `hp:caption` 자체에는 vert_align 속성이 **없다** — IR `Caption.vert_align`은 HWP5
  바이너리 유래 필드. HWPX 파서가 읽지 않는 것은 공백이 아니라 포맷 차이 (1단계에서
  대비 샘플로 재확인 후 종결)
- subList 속성(textDirection/lineWrap/vertAlign 등)은 파서가 caption 경로에서
  적재하지 않음 → 실물 값(HORIZONTAL/BREAK/TOP) 고정 방출, 한계로 기재

### 0.2 serializer 구조 (serializer/hwpx/table.rs — 코드 확인 완료)

- 방출 위치: 모듈 doc 자식 순서 `sz, pos, outMargin, (caption …), inMargin` —
  `write_table`의 `write_out_margin`(`:94`)과 `write_in_margin`(`:95`) 사이.
- `write_sub_list`(`:220`)는 Cell 전용 (text_direction/vertical_align/paragraphs 참조).
  문단 방출 본체(`:252~268`: render_paragraph_parts + render_hp_p_open + para/style
  ID reference + sub_list_depth 증감)를 **공유 헬퍼로 추출**해 cell과 caption이 함께 사용.

### 0.3 게이트 구조 (serializer/hwpx/roundtrip.rs — 코드 확인 완료)

- Table arm 3곳이 `cells` 문단만 순회: linesegs(`:543`), char_shapes(`:659`),
  controls(동일 패턴) — caption.paragraphs 미방문.
- path 표기: 기존 `/ctrl[{ci}]tbl.cell[{j}].p[{k}]` 에 맞춰 caption 은
  `/ctrl[{ci}]tbl.caption.p[{k}]`.

## 1단계 — 전수 측정 + vert_align 조사

코드 수정 없음 (조사 전용).

### 1.1 caption 분포 측정

- `samples/hwpx` 전수에서 `hp:caption` 출현 카운트 (원본 XML, 표 하위 vs 그림/도형
  하위 구분) → RT(`output/poc/task1388/`) 카운트와 대조해 소실 정량화.
- 캡션 subList 내 컨트롤 분포 (autoNum 등 — #1382 간섭 후보) 측정.

### 1.2 vert_align 조사 종결

- `hp:caption` 속성에 수직 정렬이 실재하는지 대비 샘플 + 한컴 OWPML 모델
  (hancom-io/hwpx-owpml-model CaptionType) 교차 확인 → 파서 보강 불요/필요 판정.

### 1.3 그림/도형 캡션 범위 판정

- `ShapeComponent.caption` 경로 분포 확인 — 표 외 캡션 소실이 실재하면 별도 이슈
  분리 제안 (본 타스크는 표 한정).

### 1.4 게이트 동승 영향 사전 판정 + 보고

- caption 소실은 현재 전 샘플 공통이므로 2단계 수정 후 동승 시 xfail 0 예상 — 측정으로 확정.
- `mydocs/working/task_m100_1387_stage1.md` → 승인 요청

## 2단계 — serializer 수정 (`write_table` caption 방출)

### 2.1 공유 헬퍼 추출

- `write_sub_list`의 문단 방출 본체를 `write_sub_list_paragraphs(w, paragraphs, ctx)`
  로 추출 — cell 경로 동작 무변경 (기존 테스트로 보증).

### 2.2 `write_caption` 신설

- `write_out_margin` 직후 `if let Some(caption) = &table.caption` 방출:
  - `hp:caption` 속성 5종 역매핑: side(direction 4종) / fullSz(include_margin) /
    width / gap(spacing) / lastWidth(max_width) — 0.1 실물 속성 순서 준수
  - `hp:subList` 래퍼 (0.1 실물 고정 속성) + 공유 헬퍼로 문단 방출

### 2.3 단위 테스트 (table.rs)

- caption 속성 역매핑 (direction 4종 포함)
- caption 문단 + 내부 autoNum 컨트롤 방출
- caption 없는 표 → `hp:caption` 미방출 (기존 동작 무변화)
- 실샘플(ta-pic-001-r) serialize → `hp:caption` 존재 + 텍스트 보존

### 2.4 보고 + 승인 요청

- spot 배치 재실행으로 caption 소실 해소 수치 포함, `_stage2.md`

## 3단계 — 게이트 동승 (Table caption 비교)

### 3.1 caption 구조 비교

- `IrDifference::TableCaption { section, paragraph, path, detail }` 추가 —
  존재 비대칭(Some/None), 속성 5종, 문단 수 불일치를 detail 로 보고.

### 3.2 문단 재귀 동승

- Table arm 3곳(char_shapes/controls/linesegs)에 `caption.paragraphs` 대응 순회 추가
  — path `…tbl.caption.p[{k}]`.

### 3.3 단위 테스트 (roundtrip.rs)

- caption 소실 주입(Some vs None) → `TableCaption` 검출
- caption 문단 char_shapes 차이 주입 → 재귀 검출 (path 확인)
- 실샘플(ta-pic-001-r) roundtrip → 게이트 0 (2단계 수정 후 기대치)

### 3.4 보고 + 승인 요청

- baseline 전수 결과(xfail 변동 포함), `_stage3.md`

## 4단계 — 전수 검증 + 문서 + 한컴 판정 요청

1. `hwpx-roundtrip --batch samples/hwpx` 전수 → `output/poc/task1387/`
2. `cargo test --test hwpx_roundtrip_baseline` — 신규 실패 0
3. ta-pic-001-r SVG 좌표 대조 — **완전 일치** 확인 (#1388 4단계에서 잔존 18좌표가
   캡션뿐임을 실증했으므로 본 타스크의 결정적 판정 기준)
4. 매뉴얼 `mydocs/manual/hwpx_roundtrip_baseline.md` 갱신 (#1387 해소 처리 + 게이트 항목)
5. CI급: `cargo test --profile release-test --tests` + `cargo fmt --check` + clippy
6. 최종 보고서 + 한컴 판정 요청 (ta-pic-001-r.rt — 캡션 복원 범위 명시, #1389 그림
   크기 증상은 기지 이슈로 안내)

## 위험 관리 (수행계획서 5절 보강)

| 위험 | 단계 | 대응 |
|------|------|------|
| 요소 순서 어긋남 → 한컴 열기 실패 | 2 | 0.1 실물 + 모듈 doc 순서 준수, 한컴 판정으로 최종 확인 |
| 공유 헬퍼 추출이 cell 경로 회귀 유발 | 2 | 추출은 무변경 리팩터링 — 기존 cell 테스트 + baseline으로 보증 |
| 캡션 내 autoNum이 #1382 비일관 간섭 | 1·3 | 1단계 분포 측정, 발생 시 #1382 귀속 (신규 xfail 판정) |
| subList 고정 속성이 실물과 다른 샘플 존재 | 1 | 1단계 전수에서 caption subList 속성 변종 유무 확인 |
