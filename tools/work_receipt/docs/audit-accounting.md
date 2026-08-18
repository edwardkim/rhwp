# audit — 폴더 재현율 회계

```
rhwp audit <캡슐 폴더> [--json]
```

대상은 폴더 **직속** `*.capsule.json` (비재귀, 이름 정렬).
0개면 exit 2 + stdout 0바이트. 없는 폴더는 exit 1.

## 회계

```
reproducedRate = reproduced / total
```

빈 폴더에 0.0 을 주지 않는다 — 그 경로는 usage 다.

`failed[]` 가 하나라도 있으면 exit 3. 회계는 봉투로 읽고, 실패
캡슐만 replay verify 로 개별 추적한다.

## 실패 종류

| 종류 | 픽스처 | 봉투 |
| --- | --- | --- |
| 출력 해시 불일치 | `mixed`, `tamper_output` | `expected`/`actual` |
| 입력 해시 불일치 | `tamper_input` | `kind=inputSha256` |
| steps 길이 | `steps-tamper` | `kind=steps` 또는 error 바늘 |
| plan≠planText | `plan-vs-text` | error 바늘 |
| planText 해시 | `plan-text-sha` | error 바늘 |
| kind | `wrong-kind` | error 바늘 |
| JSON | `invalid-json` | 파싱 실패 |
| 기형 hex | `bad-output-sha` | 가드 |

audit 는 체인을 따라가지 않는다. 같은 폴더의 부모·자식은 각각
재실행된다 (`same-folder-chain`). 연대기는 `lineage` 의 일이다.

확장자 필터는 `ends_with(".capsule.json")`. `.bak` / `.txt` /
중간 `.json` 은 무시 (`mixed-ext`).
