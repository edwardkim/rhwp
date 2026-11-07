# e4ffabea 리베이스와 충돌 해소

- 기존 base: b8dd67c5001dfb0410e9e192528676396cdd6a4e
- 새 base: e4ffabea38062e716f6c333cc950fa31b42b4dd7
- 리베이스 전 head: 656d5d4e68112a21367958ac4d66db5926959ce9
- 리베이스 후 code head: be919a1f22be74ea5826034721d1d801352dfe6c

로컬 커밋은 전후 각각 79개이며 range-diff로 대조했다.

tests/cases/skia_native_glyph_replay.rs에서 새 devel의 glyph proof·cache·vertical 검사 추가와 이 브랜치의 #7640 시스템 프로그램 모듈 추가가 같은 파일 끝에서 충돌했다.
upstream의 기존 함수 35개를 모두 유지하고 로컬 모듈의 함수 4개(검사 2개 포함)를 추가했다. 양쪽 변경을 보존했으며 기대값·허용폭·CI 진입점을 변경하지 않았다. 상단 설명에도 두 검사 범위를 함께 기술했다.
함수 소유 대조는 충돌 보존 기록 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7375/rebase-e4ffabea-20261107/conflict-preservation.json`)에 있다.

진행 중 Git 상태 조회와 겹칠 수 있는 index.lock 경합이 한 차례 발생했다. 잠금이 사라진 뒤 파일 삭제 없이 재개했고 커밋 누락·중복은 없다. 이후 변경 중 상태 조회를 겹치지 않는다.

수정 전 Native·WASM 실행 파일과 source manifest는 output의 screen-dash-and-font-v3에 고정했다. 새 code head의 빌드·실행·전쪽 TSV와 새 devel 기능 보존 검증은 이어서 수행한다. 리베이스 완료 자체를 검증 통과로 판정하지 않는다.
