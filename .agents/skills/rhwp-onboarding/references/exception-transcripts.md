# 예외 경로 실물 리포트 — 세 갈래 트랜스크립트

에이전트가 `exceptions[].kind` 만 보고 추측하지 않도록, 세 예외의 **리포트 모양**을
필드 단위로 적는다. 값은 예시이며, 통과를 위조한 실행 결과가 아니다.
바이너리 없는 CI 와 불량 픽스처로 같은 갈래를 재현할 수 있다.

관련 구현: `tools/agent_onboarding/rhwp_doctor.py` 의 `make_exception` /
`classify_sample` / `probe_network` / `aggregate`.

## 1. missing_binary — 아직 빌드하지 않음

재현:

```bash
python tools/agent_onboarding/rhwp_doctor.py --json --offline --repo-root /tmp/empty-tree
```

프로세스 종료 코드: **3**.

stdout 요지:

```json
{
  "schemaVersion": "1.1",
  "tool": "rhwp_doctor",
  "ok": false,
  "exitCode": 3,
  "binary": {
    "found": false,
    "path": null,
    "source": "(미발견)",
    "onPath": false,
    "version": null
  },
  "sample": null,
  "checks": [
    {"id": "python", "status": "PASS", "critical": false},
    {"id": "network", "status": "SKIP", "critical": false, "exception": "no_network"}
  ],
  "exceptions": [
    {
      "kind": "missing_binary",
      "title": "바이너리 미발견",
      "detail": "rhwp 를 찾지 못했다",
      "nextSteps": [
        "저장소 루트에서 `cargo build --release --bin rhwp` 를 한 번 실행한다."
      ]
    }
  ],
  "mcpJson": {
    "mcpServers": {
      "rhwp": {"command": "rhwp", "args": ["mcp-serve"]}
    }
  },
  "buildCommand": "cargo build --release --bin rhwp"
}
```

읽는 법:

1. `ok` 가 false 여도 MCP 스니펫은 나온다. **지금 붙이면 호스트가 실행에 실패**한다.
2. `checks` 에 `version` / `selftest-*` 가 없다. 바이너리가 없어 돌리지 않았다.
3. `--offline` 이면 `no_network` 가 정보로 같이 실릴 수 있다. 종료 코드 3 의 원인은
   네트워크가 아니다.
4. `binaryInventory[]` 의 모든 `exists` 가 false 인지 본다. 한 자리라도 hit 인데
   `found==false` 면 버그로 보고한다.

하지 말 것:

- `cargo build` 를 백그라운드로 걸고 온보딩을 성공이라고 쓰기.
- gym 바이너리나 다른 크레이트 exe 를 `--rhwp` 로 속이기.
- `mcpJson` 을 호스트에 붙인 뒤 "온보딩 완료"로 보고.

다음 문서: [exception-missing-binary.md](exception-missing-binary.md),
[binary-discovery.md](binary-discovery.md).

## 2. bad_sample — 파일은 있으나 문서가 아님

재현 (바이너리가 있는 워크트리):

```bash
python tools/agent_onboarding/rhwp_doctor.py --json --offline \
  --sample tools/agent_onboarding/fixtures/samples/text_named_hwp.hwp
```

프로세스 종료 코드: **1** (바이너리는 있음).

stdout 요지:

```json
{
  "ok": false,
  "exitCode": 1,
  "binary": {"found": true, "source": "target/release"},
  "sample": "…/fixtures/samples/text_named_hwp.hwp",
  "sampleClassification": {
    "ok": false,
    "kind": "not_document",
    "reason": "OLE/ZIP/HWP3 시그니처가 없다. 텍스트·잘린 파일일 가능성이 크다.",
    "sizeBytes": 188,
    "magicHex": "5468697320697320"
  },
  "checks": [
    {"id": "version", "status": "PASS", "critical": true},
    {
      "id": "selftest-info",
      "status": "FAIL",
      "critical": true,
      "exception": "bad_sample",
      "command": "rhwp info <샘플> --json"
    },
    {
      "id": "selftest-export-text",
      "status": "FAIL",
      "critical": true,
      "exception": "bad_sample"
    }
  ],
  "exceptions": [
    {
      "kind": "bad_sample",
      "title": "불량·부재 샘플",
      "nextSteps": [
        "samples/basic/english.hwp 같은 평범한 번들 문서를 쓴다."
      ]
    }
  ]
}
```

빈 파일:

```json
{
  "sampleClassification": {
    "ok": false,
    "kind": "empty",
    "sizeBytes": 0,
    "magicHex": ""
  }
}
```

너무 작은 파일 (`tiny.hwp`):

```json
{
  "sampleClassification": {
    "ok": false,
    "kind": "too_small",
    "sizeBytes": 5
  }
}
```

읽는 법:

1. `binary.found==true` 인데 `ok==false` 다. 도구가 없는 것이 아니라 입력이 나쁘다.
2. `selftest-info` 를 실제로 돌리지 않았을 수 있다. 매직이 거절하면 명령 문자열만
   남고 파서에 넣지 않는다.
3. `magicHex` 가 `d0cf11e0` 으로 시작하는데 FAIL 이면 잘린 OLE 이거나 파서 거절.
   그때는 같은 명령을 손으로 실행해 stderr 를 본다.

하지 말 것:

- 픽스처를 정상 샘플로 바꿔 테스트를 초록으로 만들기.
- `kind==avoid` 인 gym/output 경로를 `--sample` 로 강제해 성공 시연.
- 확장자만 `.hwp` 인 메모장을 사용자에게 "열렸다"고 보고.

다음 문서: [exception-bad-sample.md](exception-bad-sample.md),
[sample-selftest.md](sample-selftest.md).

## 3. no_network — 오프라인은 실패가 아님

재현:

```bash
python tools/agent_onboarding/rhwp_doctor.py --json --offline
```

프로브를 돌리되 외부망이 없으면:

```bash
python tools/agent_onboarding/rhwp_doctor.py --json
```

`network` 칸 요지 (`--offline`):

```json
{
  "network": {
    "probed": false,
    "reachable": null,
    "offline": true,
    "targets": [],
    "reason": "--offline"
  },
  "checks": [
    {
      "id": "network",
      "status": "SKIP",
      "critical": false,
      "exception": "no_network",
      "detail": "프로브 생략(--offline)"
    }
  ]
}
```

프로브가 실패한 경우:

```json
{
  "network": {
    "probed": true,
    "reachable": false,
    "offline": true,
    "targets": [
      {"host": "1.1.1.1", "port": 443, "ok": false, "error": "timed out"},
      {"host": "8.8.8.8", "port": 443, "ok": false, "error": "timed out"}
    ]
  }
}
```

읽는 법:

1. `network` 검사의 `critical` 은 항상 false. 이것만으로 `ok` 가 false 가 되지 않는다.
2. 바이너리와 정상 샘플이 있으면 **exit 0 + exceptions 에 no_network** 가 동시에 있을 수 있다.
3. MCP 는 stdio. 오프라인이라고 `mcp-serve` 를 건너뛰지 않는다.
4. `cargo build` 의 네트워크 필요는 별 문제다. 닥터가 crate 를 받지 않는다.

하지 말 것:

- `no_network` 를 보고 온보딩을 중단.
- crates.io 에서 샘플 HWP 를 받으려 하기.
- 프로브를 HTTP GET 으로 "개선"하려 하기. 하지 않는다.

다음 문서: [exception-no-network.md](exception-no-network.md).

## 4. 세 갈래를 한 표로

| 관찰 | kind | exit | 다음 한 줄 |
|---|---|---:|---|
| `binary.found==false` | missing_binary | 3 | `cargo build --release --bin rhwp` |
| `sampleClassification.ok==false` | bad_sample | 1 | `--sample samples/basic/english.hwp` |
| `network.offline==true` 만 | no_network | 0 가능 | 로컬 레시피 계속 |
| `--write` 기존 파일 | write_exists | 2 | `--force` 또는 병합 |
| 알 수 없는 `--host` | (exceptions 없음) | 2 | `--list-hosts` |

## 5. 사람이 보는 stderr 머리

`--json` 이어도 stderr 에 같은 판정이 한글 리포트로 반복된다. 에이전트는
stdout JSON 을 이긴다. 사람이 콘솔만 볼 때는 stderr 의 `판정:` 줄을 본다.

```
rhwp doctor — 에이전트 제로프릭션 온보딩 점검
repo: C:\Users\…\rhwp-agent-onboarding-doctor

[1] 바이너리 위치·버전
  [FAIL] rhwp 미발견 — 아직 빌드 안 됨. 저장소 루트에서 실행:
           cargo build --release --bin rhwp
…
판정: 미완 — 위 FAIL/빌드 안내를 먼저 처리하세요  (exit=3)
```

Windows cp949 콘솔에서 한글이 깨지면 JSON 파일을 UTF-8 로 연다. 닥터는
스트림을 UTF-8 로 맞추려 하지만 콘솔 폰트/코드페이지가 남을 수 있다.

## 6. 테스트가 잠그는 것

`tools/agent_onboarding/test_rhwp_doctor.py`:

| 테스트 | 잠그는 갈래 |
|---|---|
| `TestMainCli.test_missing_binary_json_exit_three` | exit 3 + kind + stderr 한글 |
| `TestMainCli.test_bad_sample_json_exit_one` | 매직 거절 + FAIL + kind |
| `TestNetwork.test_check_network_offline_is_noncritical_skip` | 오프라인 SKIP |
| `TestAggregate.test_network_skip_does_not_force_exit_one` | 오프라인이 건강을 안 뒤집음 |
| `TestSampleClassification.*` | empty/too_small/not_document/ole/zip/hwp3/avoid |

이 테스트는 rhwp 바이너리 없이 돈다. CI 가 네이티브 빌드를 안 해도 예외 갈래는 산다.

## 7. 픽스처 경로

```
tools/agent_onboarding/fixtures/samples/empty.hwp
tools/agent_onboarding/fixtures/samples/tiny.hwp
tools/agent_onboarding/fixtures/samples/text_named_hwp.hwp
tools/agent_onboarding/fixtures/samples/truncated_ole.hwp
tools/agent_onboarding/fixtures/samples/zeros.hwp
tools/agent_onboarding/fixtures/samples/not_hwp.txt
tools/agent_onboarding/fixtures/reports/missing_binary.shape.json
tools/agent_onboarding/fixtures/reports/bad_sample.shape.json
tools/agent_onboarding/fixtures/reports/no_network.shape.json
tools/agent_onboarding/fixtures/reports/healthy.shape.json
```

정상 HWP 는 여기 없다. `samples/basic/english.hwp` 가 성공 경로의 정본 후보다.
