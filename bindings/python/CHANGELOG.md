# 변경 기록

이 패키지는 [rhwp](https://github.com/edwardkim/rhwp) 본체와 별개로 버전을 매긴다.
봉투 계약(`schemaVersion`)과 IR 계약(`irSchemaVersion`)은 각각 본체가 관리한다.

## 0.1.0 (미출시)

M18 1호 릴리스 (#3762). 로드맵 #3608 의 외부 바인딩 3계열 중 첫 번째.

### 추가

- **1층 무상태 API** — `info`·`export_text`·`export_structure`·`export_tables`·
  `fields`·`search`·`digest`·`capabilities`·`export_svg`·`export_pdf`·
  `export_markdown`·`export_hml`·`export_doclang`·`thumbnail`·`extract_pages`·
  `build_from_ingest`·`export_hwpx`·`convert`·`ir_diff`·`fill_fields`·
  `replace_text`·`set_cell`·`batch`
- **2층 세션 API** — `rhwp.open()` / `Document` / `Session`.
  `mcp-serve` stdio 클라이언트로 `hwp_doc_*` 를 그대로 노출한다.
- **3층 계획 API** — `Plan` 빌더와 `PlanResult`.
  `check()` 는 디스크 무변경 검사, `run()` 은 원자 실행.
- **IR 스키마 소비** — `ir_schema()` / `IrSchema` / `TypeDef` / `FieldDef`.
- **모델 생성기** — `tools/gen_models.py` 가 스키마에서 dataclass 를 만든다.
  `--check` 로 최신 여부를 CI 에서 검사한다.
- **예외 체계** — exit 코드를 파이썬 예외로 옮기되, **판정 실패(3/4)는 예외가
  아니라 반환값**이다.
- **바이너리 탐색** — `RHWP_BIN` → 패키지 동봉 → `PATH`.

### 계약

- 런타임 의존성 **없음** (표준 라이브러리만). 바인딩이 무거우면 "재포장"이 아니다.
- 봉투 키 ↔ 파이썬 속성은 **기계 변환**. 수기 개명 금지.
- `None`(모름)과 `[]`(없음)을 구분한다.
- 없는 필드는 조용한 `None` 이 아니라 예외다.

### 알려진 제약

- 휠에 바이너리를 동봉하지 않는다 — `PATH` 또는 `RHWP_BIN` 이 필요하다.
- 계획 `--dry-run` 은 rhwp 본체 #3759 머지 후에 동작한다
  (미지원 버전에서는 통합 테스트가 자기서술을 확인하고 건너뛴다).
