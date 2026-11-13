# #9018 최종 검토 증적

코드 `cc6c60381dfeff9b7892e836b8bf01803d0a5c1a`, base
`fa32a1633b0677f3dc48ea5cc99a4cab309361b6`에서 검증했다.
원 head는 `be67439b02f42779b50361ace7b8a47dd0d1439a`다.
[producer](producer.json)·[최종 검증](final-validation.json)·[리베이스 근거](rebase-scope.json)를 구분했다.

| 검증 | 결과 |
| --- | --- |
| 최소 글꼴 집중 회귀 | 보정 전 12 PASS/1 FAIL → 보정 후 13 PASS |
| 최소 글꼴 전체 회귀 | 13,446 PASS / 0 FAIL / 45 skip |
| Native Skia | 4,034 PASS, 기존 12 ignore |
| Native PNG / 직접 PDF | 2 PASS / 36 PASS |
| fmt·Native/WASM/all-target Clippy·workspace build·manifest·unit-tier | 모두 통과 |
| Python Visual Sweep / manifest 계약 | 89 PASS / 24 PASS |
| Windows 주석 검증 | 9개 라벨 검사·전체 89 PASS, 동일 스크립트 해시로 재사용 |
| 원본·편집본·실제 생성 저장본 전쪽 | Native 6쪽/fresh WASM 6쪽, 각각 최저 99.99049%/99.99050% |
| 최종 Native 편람 HWPX | 383쪽, raw 최저 99.96%(130쪽), 공식 보호 표시값 하락 0쪽 |

Visual Sweep은 일반 원본 글꼴 환경이다. 최소 글꼴 격리는 Rust 회귀에 적용했으며 개인 글꼴을 변경하지 않았다.
WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
Docker 최적화 검증으로 표시하지 않는다.

## 같은 입력·쪽의 직접 증거

각 `cases/{original,edited}/{native,wasm}/`에 두 쪽의 raw/PDF/review/overlay와
기준·후보 TSV를 보존했다. [전쪽 지표·해시](page-evidence.json), [직접 판독](direct-reading.json).
공식 두 자리 표시 100.00은 정확한 RGB 일치를 뜻하지 않는다.

- [편집본 Native1쪽 review](cases/edited/native/review-p001.png)·[overlay](cases/edited/native/overlay-p001.png)
- [편집본 Native2쪽 review](cases/edited/native/review-p002.png)·[overlay](cases/edited/native/overlay-p002.png)
- [편집본 WASM1쪽 review](cases/edited/wasm/review-p001.png)·[overlay](cases/edited/wasm/overlay-p001.png)
- [편집본 WASM2쪽 review](cases/edited/wasm/review-p002.png)·[overlay](cases/edited/wasm/overlay-p002.png)

## 실제 사용과 보정

[실제 편집·HWP/HWPX 두 번 저장·재열기](actual-edit-save-results.json)는 모두 2쪽·본문 보존이다.
생성 저장본은 실행 결과이며 별도 정적 입력 사본으로 중복 추가하지 않았다.
Native/fresh WASM의 [생성본 TSV](actual-generated/)와 PNG는 기존 커밋 편집본의 대응 출력과 같다.
[기존 표준 CFB 스트림 대조](candidate-generated-stream-audit.json)는 새 생성 HWP의 전체 파일 해시가
이전 생성 결과와 같은 것을 확인해 재사용했다. 추가 private `RhwpLineSegTags`는 구분했다.

[누름틀 끝 원인·범위](field-end-repair/cause.md), [집중 회귀](field-end-repair/focused-validation.json),
[새 WASM 저장 파일의 독립 레코드 확인](field-end-repair/actual-wasm-validation.json)을 연결했다.
END 뒤 그림 추가에도 앞선 줄 시작 56을 보존하고 char_count만 65→73으로 증가했다.
이 합성 반례는 메타데이터 관계 검사이며 실물 px 좌표/시각 수용으로 주장하지 않는다.

[실제 HTTP 초기화 응답](browser-runtime-identity.json)은 Root pkg·Studio public·응답 JS/WASM의 해시 일치를 확인했다.
Studio 전체 UI E2E나 CanvasKit 수용을 새로 실행했다고 주장하지 않는다.

Windows는 SSH→PowerShell5.1→venv Python3.12에서 주석 도구를 실행했다.
[실행 근거](windows-execution.json)·[두부문자 출처](reported-overlay-audit.json)·
[보정 전](windows-label-before-overlay.png)·[보정 후 review](windows-label-after-review.png)·
[보정 후 overlay](windows-label-after-overlay.png). 문서 raw PNG와 수치는 그대로이며
한글 주석 누락 30자가 0자로 줄었다. Windows Rust/HWP 전체 렌더링 검사로 표시하지 않는다.

## 보호 결과의 재사용

[재사용 자격](protected-reuse-audit.json)은 18조건 Native/WASM 각 1,771쪽의 원 producer·입력·PDF·
글꼴·TSV 해시를 보존한다. 새 head에서 전쪽 재캡처한 결과로 바꾸어 쓰지 않는다.
리베이스의 #9012 적용 범위를 실제 선언/사용 글꼴과 18문서의 외부 그림 목록으로 확인했다.
딸기 Bold는 해당 HFT 자격을 충족하지 않고, sample10의 연결 그림 3개는 같은 커밋 원본을 공급해
loaded 상태를 확인했다. 그 외 보호 입력에 외부 그림 참조는 없다.
현재 수정은 고정 입력의 읽기 전용 출력이 아니라 편집 명령의 좌표 이동에만 적용된다.
최종 head의 [Native383쪽 필수 gate](handbook-current/gate.json)와
[전체 TSV](handbook-current/silhouette.tsv)는 별도로 실제 다시 실행했다.

기준 PDF는 기존 공식 2020 자료 및 원 작성자가 확인한 데스크톱 한컴 2022 12.0.0.535의
Print→Microsoft Print to PDF다. 이전 3쪽 PDF-save 결과는 정량 기준으로 사용하지 않았다.
원본·독립 PDF는 기존 커밋을 재사용한다. font binary·실행 로그·임시 스크립트는 포함하지 않았다.
