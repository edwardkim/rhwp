---
kind: guide
status: active
canonical: mydocs/manual/consumer_edit_api_guide.md
last_verified: 2026-09-13
---

# 템플릿 복제·채우기 API (#3587)

고정 양식의 명시 영역에 값을 쓰거나 원형을 복제한 뒤 복사본별로 값을 채운다.
Gym 없이 native·WASM·CLI `run`·MCP `hwp_run_plan`에서 같은 문서 코어를 사용한다.
한컴 조판 결과를 맞추려고 표 높이·글꼴·빈 문단을 바꾸는 기능이 아니다.

## 연산과 주소

| action | native 요청 | 동작 |
| --- | --- | --- |
| `fill_template` | `FillTemplateRequest` | 기존 ID와 문단 구조를 유지하며 지정 범위만 채움 |
| `repeat_and_fill_paragraph_block` | `TemplateFillRequest` | `[sourceStart, sourceEnd)` 문단 묶음을 복제하고 복사본별로 채움 |
| `repeat_and_fill_table_rows` | `RepeatTableRowsRequest` | 본문 최상위 표의 완결 행 묶음 `[startRow, endRow)`을 복제·채움 |

각 action은 `{action, request}`로 구성한다. 요청의 필드명은 camelCase이며,
정확한 스키마는 `rhwp export-plan-schema --bare`에서 얻는다. 요청의 알 수 없는 필드나
잘못된 타입은 오류다. 계획 스키마 판번호는 `1.3`, 계획의 `planVersion`과 응답 봉투의
`schemaVersion`은 계속 `1.0`이다.

모든 인덱스는 **실행 전 문서의 0 기준**이다. 페이지 번호나 화면 좌표는 주소가 아니다.
본문 문단 블록의 binding은 원형 시작 문단에 상대적인 경로를 사용한다. 고정 양식도
`scope.start`에 상대적이다. 예: `[{"kind":"paragraph","index":0}]`.
하위 소유 경로는 `control`, `shape`, `cell`, `textBox`, `caption`, `groupChild`로
구성하며 단일 노드인 shape/textBox/caption에는 index를 쓰지 않는다.

행 복제의 binding은 `paragraph(0)/control(0)/cell(n)/paragraph(p)`로 시작한다.
`n`은 **선택된 셀을 행·열 앵커 순으로 정렬한 인덱스**이고 원본 표 전체의 셀 번호가 아니다.
결과의 행 복제 `copies[].mappings`는 원본·목적지 모두 실제 절대 소유 경로다.
문단 복제 mapping의 source는 원형 상대 경로, destination은 삽입 직후 절대 경로다.
이 대응표는 영구 ID가 아니며 후속 편집 뒤 그대로 재사용하지 않는다.

## 값과 보존 계약

- `bindings`는 `{key, target}` 배열, `record`는 문자열 값 객체다. 복제 요청은
  `records` 배열을 받으며 문단 복제의 `block.count`는 그 길이와 같아야 한다.
- `target.kind: "textRange"`는 문단의 `path`, `start`, `end`를 받는다. offset은
  UTF-16 코드 유닛이 아닌 **Unicode scalar 인덱스**다. 끝은 포함하지 않는다.
- `target.kind: "field"`는 같은 문단 안에서 닫힌 ClickHere의 `fieldRangeIndex`를 받는다.
  문서 전체 필드 occurrence가 아니다. native의 범위 제한 이름/셀 편의 selector로
  명시 binding을 만들 수 있다.
- 누락/여분 key, 중복 binding, 겹친 범위, 그림·필드 등 다른 컨트롤 경계 침범은 거부한다.
  빈 문자열은 범위의 값만 비운다. 셀 전체를 평탄화하거나 빈 문단을 삭제하지 않는다.
- 행 복제는 source·삽입 경계를 가로지르는 병합/zone, 제목 셀 복제와 제목 블록 앞/내부
  삽입을 거부한다. 중첩 표 내부 행 증가는 초기 지원 범위가 아니다.
- 복제는 **멱등 연산이 아니다**. 같은 요청을 두 번 실행하면 다시 추가한다.
  CLI의 `preconditions.inputSha256`은 입력 버전 보호이며 영구 요청 중복 제거가 아니다.

블록의 구조 보존 지원과 값 채우기 대상은 다르다. 기본 글상자와 일반 하이퍼링크 필드는
블록 복제/가져오기에서 보존하지만, `target.kind: "field"`로 값을 채우는 대상은 계속 ClickHere다.
링크 주소는 데이터로 복사하며 복제 중 접속하거나 실행하지 않는다. HWPX의 단일 `Command`
매개변수는 기존 링크 문자열 및 표준 표현과 일치할 때만 허용하며, 미지 매개변수·양식/이름을
가진 글상자 확장은 명시적으로 거부한다. 다른 문서 가져오기의 공개 WASM/CLI 연결은 아직 후속 단계다.

## 먼저 dry-run, 이어서 실행

dry-run은 실행과 같은 경로로 detached 사본의 채우기·ID/참조·작업량 검증까지 수행하고
반영 직전에 버린다. 원본·raw cache·이벤트·클립보드·조판·파일은 바꾸지 않는다.
기존 native `validate_template_fill_native`는 더 가벼운 선검증으로, 이 전체 준비와 구별한다.
실제 반영용 용량 확보가 환경 메모리 부족으로 실패할 가능성과 OOM/process abort 복구까지
dry-run 성공이 보증하지는 않는다.

독립적으로 검사 가능한 잘못된 target은 최대 16개까지 함께 보고한다. 구조·key·예산
선검증 실패는 먼저 중단하며, 선행 구조가 없어 검사할 수 없는 대상을 추측해 오류로 만들지 않는다.
CLI 실행 모드는 선검증과 반영에서 준비 경로를 각각 한 번 호출한다. 이 비용은 별도
C 종료 계측 대상이며 `workload`가 그 실행 시간이나 메모리 비용을 대신하지 않는다.

WASM은 `HwpDocument.applyTemplateOperation(optionsJson)`을 사용한다.

```javascript
const operation = {
  action: "fill_template",
  request: {
    scope: { sectionIndex: 0, start: 1, end: 2 },
    bindings: [{ key: "title", target: {
      kind: "textRange", path: [{ kind: "paragraph", index: 0 }],
      start: 0, end: 0
    } }],
    record: { title: "실험 기록" }
  }
};
// 예시 주소다. 실제 입력에서 대상과 범위를 먼저 확인해야 한다.
const preview = JSON.parse(doc.applyTemplateOperation(JSON.stringify({operation, dryRun: true})));
const result = JSON.parse(doc.applyTemplateOperation(JSON.stringify({operation})));
```

생략한 `dryRun`은 false이고 문자열 `"false"` 같은 잘못된 타입은 거부한다.
오류는 JS 예외로 전달한다. 성공 응답에는 `operationResult`의 적용/예정 대상·대응표,
`dryRun`, 출처 표지가 있으며 `changedPages`는 아직 확정하지 않으므로 null이다.
`workload`는 record 수·확장 target 수·대체 문자열 UTF-8 바이트 수다.
실제 메모리 사용량이나 조판 비용의 실측값은 아니며 CLI 저널에도 같은 항목을 싣는다.
이 API는 메모리 문서만 편집한다. 파일 저장은 기존 별도 export API로 수행한다.
브라우저에서는 이 binding을 포함하여 빌드한 WASM 패키지가 필요하다.

CLI 계획은 `steps: [operation]`으로 동일 요청을 넣는다.

```json
{
  "planVersion": "1.0",
  "input": "samples/rnote/labnote-001.hwp",
  "output": "output/labnote-rows.hwp",
  "steps": [{
    "action": "repeat_and_fill_table_rows",
    "request": {
      "sectionIndex": 0, "paragraphIndex": 12, "controlIndex": 1,
      "startRow": 5, "endRow": 6, "insertBefore": 6,
      "bindings": [], "records": [{}, {}]
    }
  }]
}
```

`rhwp run plan.json --dry-run --json`으로 먼저 확인하고, 승인 후 `--dry-run` 없이
실행한다. 기존 `run`은 **입력 형식을 보존**한다. HWP 입력에 `.hwpx` 이름만 지정해도
HWPX 변환이 되지는 않는다. 먼저 `rhwp export-hwpx`로 변환한 HWPX를 입력으로 사용한다.
기존 예외인 HWPX 입력의 `.hwp` 출력은 어댑터 경유 변환과 경고를 유지한다. `assertions.verify:true`는
기존 저장 자기검증을 추가하며 한컴 시각 판정을 대신하지 않는다. MCP는 위 계획을 그대로
`hwp_run_plan`의 `plan` 인자로 준다. 별도 MCP 전용 복제 엔진이나 세션 API는 없다.

새 템플릿 action은 **계획당 단독 step만 허용**한다. 여러 target/record는 한 요청에 묶고,
다음 연산은 저장 결과를 입력으로 하는 다음 계획으로 연결한다. 기존 네 action만 사용하는
다중 step 계획과 `if` 조건은 유지한다. 조건이 거짓이면 기존 계약대로 건너뛰며, 이를
실제로 채웠다는 성공으로 해석하지 않는다.

## 안전 상한과 검증 범위

요청 JSON은 최대 8 MiB이며 기본 serde 재귀 제한도 유지한다. native typed 요청도
실제 작업량 제한을 적용한다. 복사 1,000회, 추가 문단 10,000개, 추가 노드 100,000개,
깊이 64, 구조 32 MiB, 대응표 8 MiB, 원본 검사 노드 1,000,000개가 기본 상한이다.
`limits`를 지정하면 모든 필드를 명시하며 각 상한은 낮출 수만 있다. 상한은 파일 형식
제한이나 프로세스 전체 메모리 사용량 보장이 아니다.

출력 구조·저장 성공과 Studio의 재편집 조판은 별도 판정이다. 현재 알려진
[#7065](https://github.com/edwardkim/rhwp/issues/7065) 재편집 페이지네이션과
[#7084](https://github.com/edwardkim/rhwp/issues/7084) 이모티콘 폭 문제는 이 API로 해결했다고
간주하지 않는다. 다른 문서의 스타일·이미지를 가져오는 기능과 후속 Gym 평가는 별도 단계다.
