# 수용 기준 (조각 단위)

플레이북 §3. 기계 카드: [`../fixtures/add_surface/acceptance.json`](../fixtures/add_surface/acceptance.json).

조각을 더하는 PR 의 Definition of Done 이다. 스킬 신설 PR 은 이 목록을
**구현에 적용하지 않고** 안내만 한다.

## 체크리스트

- [ ] **stdout 순수성.** `--json` 모드에서 stdout 에 JSON 하나(배치는 NDJSON)만.
      진단·진행 메시지는 stderr.
- [ ] **실패 경로.** 런타임 실패 시 stdout 비움(부분 매니페스트 금지), exit 1.
      조립 오류는 exit 2. **미지 옵션 침묵 무시 금지**.
- [ ] **`schemaVersion`.** 필드 추가는 허용, 변경/삭제는 계약 테스트가 잡는 구조.
- [ ] **출처 표지.** `untrustedContent`·`untrustedFields` 를 **모든 모드에서**
      (dry-run 포함) 싣고, `export-provenance-map` 에 항목 추가.
      문서를 열지 않는 명령도 `untrustedContent:false` 를 **명시**.
      키 부재는 옛 바이너리와 구별되지 않는다.
- [ ] **무상태 도구.** `inputSchema.required` 와 `cli.args` 자리표시자가 1:1.
      선택 인자를 자리표시자로 쓰지 않는다.
- [ ] **세션 도구.** 닫힌 핸들 `isError`, 디스크 기록은 `hwp_doc_save` 만,
      판정 어휘는 무상태 대응 도구와 동형.
- [ ] **`nextCall`.** 실패 응답에 `{name,arguments,why}` — 에이전트가 다음 수를 안다.
- [ ] **문서.** `cli_commands.md` 해당 절 현행화, 지식 지도 §1-1·§2·§5·§6·§8 에 행 추가.

## 아직 미충족인 표지 (실측, 플레이북 §3-1)

다음 6개 봉투에는 `untrustedContent`·`untrustedFields` 키가 없다.

| 봉투 | 문서 파생 값을 싣는가 |
|---|---|
| `edit redact --json` | 예 — `findings[].raw` |
| `edit sanitize --json` | 예 — `removed[].before` |
| `run --dry-run --json` | 예 — `preview[].targets[].name` |
| `edit insert-image --json` | 아니오 |
| `export-ir-schema --json` | 아니오 |
| `export-capabilities-schema --json` | 아니오 |

소비자: 키 부재 = **미표기**. `false` 가 아니다.
구현자: 새 봉투는 이 목록에 이름을 더하지 마라.

## 증적

증적은 재현 가능해야 한다. 이미지와 함께 재현 명령을 남긴다.
가짜/합성 화면은 그렇게 표기한다. 다쪽 문서 편집은 건드리지 않은 쪽의
불변(픽셀 대조)까지 포함한다.
