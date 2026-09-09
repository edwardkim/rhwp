# task_m100_3381 처리결과 보고서 — `edit set-cell` (Stage 3 세 번째 조각, 커버리지 G2)

- **이슈**: [#3381](https://github.com/edwardkim/rhwp/issues/3381)
- **브랜치**: `pr/task-edit-set-cell` (**#3345 → #3374 위 적층**)
- **범위**: `src/main.rs`(edit_set_cell + 디스패치 + help + capabilities/MCP 등재),
  `tests/edit_set_cell_contract.rs`(신규), `mydocs/manual/cli_commands.md`(신설 항목)
- **분류**: 기능 추가 (편집 — 로드맵 §7.3, 커버리지 #3370 G2 / 7행)

## 1. 배경 — 실물 양식의 표준형은 "표 양식"이다

실측: 실제 배포되는 정부 양식(기부·답례품 실적 보고서)은 **누름틀 0개·표 53개** —
사람이 표 칸에 직접 타이핑하는 구조다. 누름틀 서식(fill-fields, #3345)만으로는 세상의
양식 대부분이 채움 범위 밖이었다. set-cell 이 실물 양식 채움의 관문이다.

## 2. 설계 결정

- **좌표계 = export-tables 격자** — 발견(`export-tables`)→편집(`set-cell`)→재독
  검증(`export-tables`)이 같은 주소(index/row/col)로 닫힌다.
- **새 편집 로직 0줄** — 스튜디오가 쓰는 코어 셀 편집 경로(delete/insert_text_in_cell,
  전체 주소 지정)를 그대로 부른다.
- **격자→모델 매핑은 모델 순회로** — 격자 배열 위치를 cell_idx 로 쓰지 않는다(손상 방어
  필터로 어긋날 수 있음). (row,col) 앵커를 `table.cells` 순회로 직접 찾아 모델 인덱스를
  얻는다.
- **병합으로 덮인 칸은 앵커 좌표를 안내하며 exit 2** — 에이전트가 다음 호출을 스스로
  고칠 수 있는 오류. 격자 밖 좌표·인자 누락도 exit 2.
- **편집 계약 준수(#3329/#3373 동일)** — dry-run 은 파일 미생성(old→new 예고),
  실패 시 원본 불변, 재독 검증이 계약.
- v1 범위: 본문 최상위 표, 셀 첫 문단 교체(`--text` 에 줄바꿈·탭 불가). 중첩 표·다문단
  셀은 후속 조각.

## 3. 검증

- **계약 테스트 5종** (`tests/edit_set_cell_contract.rs`, red→green — 시험대가 **실물
  배포 양식 그 자체**): 기록+재독 대조(같은 좌표) / dry-run 파일 부재+old·new 보고 /
  **병합 덮인 칸 exit 2 + 앵커 안내**(실물 양식에서 동적 탐색) / 인자 누락 4형 exit 2 /
  격자 밖 좌표 exit 2
- 무회귀: `edit_fill_fields_contract`(7)·`edit_replace_text_contract`(5)·
  `cli_json_contract`(22) green (release-test)
- `cargo fmt` clean, clippy `-D warnings` 0건
- **실전 데모**: 실물 기부·답례품 양식의 보고 칸들을 set-cell 로 채워 렌더 — 공식 PDF
  (동일 양식의 정부 배포본)와 나란히 비교 시트 동봉

## 4. 남긴 것

- 중첩 표(`containerPath`)·다문단 셀 — 같은 규약의 후속 조각.
- `batch set-cell`(다건 좌표 일괄) — 대량 취합 갱신용, #3346 배치 골격 위 후보.
- 이로써 로드맵 §7.3 의 edit 3종(fill-fields/replace-text/set-cell)이 전부 조각으로
  존재한다 — 커버리지 7행(표 수치 갱신)·실물 양식 채움이 ✕→△(머지 시 ○).
