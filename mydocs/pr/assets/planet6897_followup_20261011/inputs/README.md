# planet6897 후속 검토 입력 보존

검토 브랜치의 9개 PR·19개 source commit 범위에서 실제 사용한 입력이다.
원래 파일을 남긴 채 저장소 안에 바이트 단위로 동일하게 보존했다.
활성 회귀 검사·golden·baseline을 등록한 것이 아니다.

- `owned-normal-picture-table.hwp`: 그림과 표의 정상 대조군.
- `plain-note-mun.hwp`, `plain-note-gana.hwp`: 미주 본문 접두어와 소유 관계 대조군.
- `pr7672-*.hwp` 8개: 기존 실물 문서에서 미주 본문을 실제 편집한 재현 입력.
  독립 한컴 Print와 전체 180쪽을 별도로 대조한다.

원본 경로·보존 경로·SHA-256과 PDF 대응은 ignored
`output/pr-review/planet6897-followup-20261011/c18-validation-input-preservation.json`에 있다.
정상 대조군 PDF 3개는 `pdf/planet6897_followup_20261011/`에 보존했고,
실물 편집본 Print 8개는 기존 `pdf/pr7672-edited-note-prefix/` 경로를 유지했다.

파일 보존은 시각 검증 통과나 PR 수용 판정이 아니다. 현재 HEAD의 Native/fresh WASM
전쪽 TSV·쪽수 일치·확대 판독과 입력 커밋 일치를 확인한 뒤 최종 검토 기록에 연결한다.
90% 미달 입력에서 기대값을 낮추거나 부적합한 배치를 새 회귀로 고정하지 않는다.
