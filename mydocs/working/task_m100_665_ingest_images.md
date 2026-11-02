# #665 재개 — 시험지 이미지 구현 기록

Issue: #665

## 착수

- 신규 이슈 중복 등록을 피하고 #665를 open으로 재개했다. 담당자에 edwardkim을 추가했다.
- 원 PR #668 → 실제 선별 반영 `80535a3be1` → #666 보정 `f708a3cb9f` → 현행 placeholder를 추적했다.
- 사용자 승인 범위는 기존 조사 결과에 따른 로컬 구현·검증이다. 원격 코드 push·PR·merge는 수행하지 않는다.
- 계획: [구현 계획](../plans/task_m100_665_ingest_images.md).
- 착수 기록 시점에는 구현·빌드·시각 검증 전이었다. 아래에 후속 실제 결과를 추가한다.

## 구현 진행

- 공통 BinData 등록을 model helper로 공유하고 기존 native/HTML 그림 생성의 치수·crop·imgDim 구성을 공통 Picture factory로 연결했다.
- 이미지 byte를 검증·공유하는 `IngestImage`, 한도와 새 `build_exam_paper_with_images` API를 작성했다. 기존 text/placeholder API는 유지한다.
- `paragraph.runs`의 엄격한 text/image 역직렬화와 JSON schema 확장을 작성했다. 실제 문자 좌표는 UTF-16 stream과 그림의 8-unit 자리를 반영한다.
- CLI 로컬 이미지 로더, JSON 디렉터리 기본 기준 경로, 명시적 media root·한도 옵션을 연결했다. 파일 경로 이탈과 손상·누락을 실패 처리한다.
- 1차 `cargo check --locked --lib` 및 `--bin rhwp`가 공용 target에서 통과했다. 이후의 세부 보정은 최종 검증 때 다시 확인한다.
- 실험용 배치 4종·문장 내 두 이미지·공유 지문/보기 사례를 ignored output에 준비했다. 아직 정식 렌더링 회귀로 등록하지 않았다.
- 기준 devel의 Native/fresh WASM producer를 별도 clean worktree에서 빌드했다. source와 producer hash는 ignored `output/issue665/base-producers.json`에 기록했다. 구현 완료·시각 통과·전체 회귀 통과를 아직 판정하지 않는다.

## 재접속 후 진행

- 로그인 재접속 뒤 branch/WIP·실행 프로세스·이슈 담당자를 재확인했다. 중단된 기준 Native 빌드를 재개해 clean base `ea86816b53`의 Native와 최적화 fresh WASM을 확보했다.
- 수정본의 `cargo check --locked --bin rhwp` 최종 확인이 통과했다(34.71초). source compile과 시각·전체 회귀 완료를 구분한다.
- 본문 소유와 UTF-16 stream·반복 이미지·문항 번호를 점검하고, 본문 크기 초과·중복 지문 ID를 진단하도록 보완했다. picture의 raw 속성은 기존 공통 packer에서 산출한다.
- CLI 매뉴얼·스킬·생성기와 산출 안내를 현행 구현 계약으로 갱신했다. 기존 시험지·route/catalog/author 계약 48개와 주요 문서 상대 링크 검사가 통과했다.
- 기준 문서는 새 빌더에서 복사하지 않고 base의 텍스트 빌더와 기존 `edit insert-picture`/`set-picture`로 따로 작성했다. 6종의 한컴 2024 Print job이 성공했다. 실제 제품 `13.0.0.3901`, `pdf_print_method: 0`, `frame_print_to_pdf_ex_one_up`을 확인했다.
- 같은 기준 문서의 base Native 전쪽 TSV는 배치 4종·runs에서 각 100.00%, shared-boxed에서 93.01%였다. shared-boxed의 이미지·문자 순서와 보기 소유를 직접 확대 확인했다. 이 값은 새 빌더 후보의 검증 결과가 아니다.
- 다음 실행: 후보 Native/fresh WASM producer → 실제 CLI/저장 재열기 → 동일 독립 Print의 전쪽 TSV·직접 판독 → 조건 충족 뒤 정식 회귀와 필수 lint/전체 회귀.
