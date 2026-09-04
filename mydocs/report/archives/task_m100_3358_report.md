# task_m100_3358 처리결과 보고서 — ingest 미지 필드 거부 (침묵 유실 차단)

- **이슈**: [#3358](https://github.com/edwardkim/rhwp/issues/3358)
- **브랜치**: `pr/fix-issue-3358-ingest-deny-unknown` (upstream/devel `4a39f7cc0` 직분기)
- **범위**: `src/parser/ingest/schema.rs`(deny_unknown_fields 6곳 + StemBlock 수동
  Deserialize + 단위 테스트 7종), `tests/issue_3358_ingest_unknown_fields.rs`(신규)
- **분류**: 버그 수정 (에이전트 입력 계약 — 실패의 계약)

## 1. 배경

ingest JSON 은 Vision/에이전트가 기계 생성하는 중간 표현인데, serde 기본 동작이 미지
필드를 조용히 무시해 **필드명 오타·구조 착오가 "성공"으로 위장**됐다. 실측(v0.8.0):
`boxed` 블록에 (올바른 `blocks[]` 대신) `text` 를 주면 exit 0 으로 빈 박스가 생성되고
내용이 소리 없이 사라진다. 시험지 파이프라인 기준 문항·보기 내용이 통째로 증발할 수
있는 부류다.

## 2. 설계 결정

- **거부가 기본** — ingest 는 기계 생성 입력이라 관용(lenient) 파싱의 이득이 없고,
  실패는 빠를수록 싸다. 미지 필드는 파싱 오류 → 기존 계약대로 exit 1(런타임, #2707),
  출력 파일 미생성. (이슈 본문의 exit 2 제안은 #2707 재확인 결과 1 이 맞다 —
  CLI 인자 오류가 아니라 입력 파일 결함이므로 기존 "ingest JSON 파싱 실패" 경로와 동일.)
- **plain struct 6종은 `#[serde(deny_unknown_fields)]`** — IngestDocument·PageSize·
  Passage·Question·Choice·Media.
- **`StemBlock` 은 수동 Deserialize** — serde 의 internally-tagged enum 은
  `deny_unknown_fields` 를 지원하지 않는다(적용해도 무효). 전 필드 합집합
  (`RawStemBlock`, deny_unknown_fields)으로 받은 뒤 type 별 허용 필드를 검증한다.
  덕분에 오류가 위치·힌트를 갖는다:
  `boxed 블록에 허용되지 않는 필드 'text' — 박스 내용은 'blocks' 배열의 text 블록으로 넣으세요`
- **Serialize 는 무변경** — 직렬화 출력(태그형)은 종전과 동일, 라운드트립 테스트 유지.

## 3. 검증

- **단위 테스트 7종** (schema.rs, red→green): boxed+text 거부(+힌트) / 미지 블록 필드 /
  미지 블록 type / 최상위 오타(defaul_font) / Question 오타(choice) / 정상 3형 파싱 /
  기존 라운드트립·기본값 테스트 무회귀
- **CLI 계약 테스트 3종** (`tests/issue_3358_ingest_unknown_fields.rs`):
  boxed+text → exit 1 + 힌트 오류 + **출력 파일 미생성** / 최상위 오타 → exit 1 /
  **공식 예제 2종(sample_minimal·sample_structured) 종전대로 exit 0** (무회귀 —
  전 키 집합을 사전 검증함)
- `cargo fmt` clean, clippy `-D warnings` 0건 (release-test 프로필)
- **전/후 스크린샷**: `assets/task_m100_3358/before.png·after.png` — 에이전트 관점
  시나리오(같은 잘못된 입력이 전: 성공 위장+내용 증발 / 후: 즉시 실패+수정 힌트)

## 4. 남긴 것

- `--lenient` 옵트인(현행 무시 동작 복원)은 요구가 생기면 조각으로. 현재는 공식 예제·
  실전 스킬 산출물이 모두 엄격 파싱을 통과하므로 불필요하다고 판단.
- JSON-Schema(`ingest_schema_v1.json`)와 serde 모델의 자동 동기화 검사는 별도 주제.
