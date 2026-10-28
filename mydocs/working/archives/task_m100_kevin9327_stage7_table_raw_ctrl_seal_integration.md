# Stage 7 - 최신 Table provenance 필드 통합 보정

## 배경

누적 후보를 최신 `upstream/devel` 위에 재배치한 뒤 전체 integration 회귀를 실행하자, scaffold 표 생성자가 새 `Table::raw_ctrl_seal` 필드를 초기화하지 않아 컴파일이 중단됐다.

## 보정

합성 표는 원본 `raw_ctrl_data`를 기반으로 재사용할 provenance가 없으므로 `raw_ctrl_seal: None`으로 생성한다. 이는 `Table` 기본값과 raw provenance 계약에 맞으며, 이후 편집·저장 경로가 raw 데이터를 잘못 재사용하지 않도록 한다.

## 검증 계획

파생 suite를 한 번 준비한 현재 작업 트리에서 전체 `cargo nextest run --cargo-profile release-test --tests --no-fail-fast`를 다시 실행한다. 생성된 suite와 manifest는 검증 뒤 커밋하지 않는다.
