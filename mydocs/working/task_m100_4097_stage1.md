# Stage 1 — task_m100_4097 측정과 red 기준선

- **이슈**: [#4097](https://github.com/edwardkim/rhwp/issues/4097)
- **계획서**: [`mydocs/plans/task_m100_4097.md`](../plans/task_m100_4097.md)
- **브랜치**: `task_m100_4097` (`upstream/devel` `d634e608b` 기준)
- **작업 시각**: 2026-08-07 KST
- **프로덕션 코드 변경**: **0** — `src/` 최종 diff 없음

## 1. 측정 목적

계획 수립 시 남은 미확인 5건을 수치로 확정하고, AC② 확장분이 현행 코드에서 **red** 임을 확인한다.

## 2. 결과

### 2.1 코퍼스 56건 (28종 × 2포맷) 중첩 CFB 전수 측정

임시 프로브 `tests/task4097_stage1_probe.rs` 로 `samples/chart/` 전건을 `rhwp::parse_document` →
`bin_data_content` 에서 CFB 매직으로 중첩 CFB 를 뽑아 측정했다(측정 후 프로브 삭제).

```text
검사 파일 수          : 56 (목표 56)
중첩 CFB 없음         : []
루트 CLSID 그룹       : { "37a13d4c90dcb9479bed59dae352a280": 56 }
스트림 집합 그룹      : { ["/\u{2}OlePres000", "/Contents", "/OOXMLChartContents"]: 56 }
서브 스토리지         : []
현행 재포장 CLSID 손실: 56/56          <- AC② red 기준선
소요시간              : 223.4006ms
```

확정된 것:

| 항목 | 값 | 계획에 미치는 영향 |
|---|---|---|
| `.hwp` 축 중첩 CFB 접근 | `bin_data_content` 에서 **그대로** 얻힌다 (56/56 성공) | **`hwp_nested_cfb()` 헬퍼 불필요** — HWP5 파서가 `parser/mod.rs:877,1673,1718,1850` 에서 4바이트 prefix 를 `drain(..4)` 하므로 이미 raw CFB 다 |
| 루트 CLSID | **56건 전부** `37a13d4c90dcb9479bed59dae352a280` = `{4C3DA137-DC90-47B9-9BED-59DAE352A280}` | `.hwp` 축도 HWPX 와 동일. 분산 0 |
| 스트림 집합 | **56건 전부 3종** (`\x02OlePres000`, `Contents`, `OOXMLChartContents`) | `known` 목록(`probe:206-211`)에 **추가할 것 없음**. `\x01Ole10Native` 는 차트 코퍼스에 없다 |
| 서브 스토리지 | **0개** | 루트 CLSID 만 받는 API 설계가 정당함을 실측으로 확인 |
| AC② red 기준선 | **56/56 손실** | 현행 `build_cfb` 는 전건에서 CLSID 를 떨군다 |
| 소요시간 | **223 ms** | 56건 확장의 성능 우려 없음 — 완화책(테스트 분리 등) 불필요 |

### 2.2 SO-SUEOP OLE 서브 스토리지 CLSID (HWP3 축)

`extract_ole_payloads`(`src/parser/hwp3/ole.rs:174-179`)에 임시 계측을 넣고
`task3363_hwp3_embedded_ole_payload_extraction` 를 `--nocapture` 로 실행한 뒤 계측을 원복했다.

```text
[4097-probe] sub-storage "/00000000.OOO" clsid=1442040000000000c000000000000046
```

→ **`{00044214-0000-0000-C000-000000000046}`** — **비-0**이다.

의미: HWP3 축의 승격 재포장은 **실제로 유효한 CLSID 를 버리고 있다**. 이슈 영향 범위 표의 "같은 손실이
일어난다"가 실측으로 확인됐다. Stage 4 의 회귀 단언에 `assert_ne!(clsid, [0u8;16])` 를 붙일 근거가 된다.

> 이 값이 `is_hmapsi_ole_container`(`ole_container.rs:172`)가 판별하는 글맵시 개체의 서버 클래스다.
> 현행 `task3363_...` 테스트는 스트림 **이름** 기반 `parse_ole_container` 만 보므로 이 손실을 잡지 못한다
> — HWP3 축에도 새 단언이 필요한 이유다.

### 2.3 `uuid` 전이 의존 컴파일 확인

`e.clsid().to_bytes_le()` (반환 `[u8;16]`)가 `Cargo.toml` 에 `uuid` 를 **명시하지 않고 컴파일된다**.
`cfb` 0.14 는 `uuid` 를 re-export 하지 않지만(`cfb/src/lib.rs:64`), inherent method 호출은 타입 이름을
요구하지 않는다. 2.1 프로브와 2.2 계측이 모두 이 호출을 쓰고 빌드에 성공했다.

→ **계획의 대안(`uuid = "1"` 명시 추가)은 불필요하다.**

### 2.4 `cfb` 크레이트 오라클 검증

`CompoundFile::root_entry().clsid().to_bytes_le()` 가 원본 파일의 원시 16바이트와 일치함을 확인했다
(56건 전부 동일 그룹으로 수렴, 그리고 그 값이 #4055 report §3 이 기록한 실측값과 정확히 같다).
→ 테스트의 **독립 오라클**로 쓸 수 있다.

## 3. 계획 갱신 사항

| 계획 기재 | Stage 1 이후 |
|---|---|
| "`hwp_nested_cfb()`(support:337) 사용 검토" | **불필요** — `bin_data_content` 직접 사용 |
| "`known` 목록에 새 스트림 추가 가능성" | **없음** — 3종으로 확정 |
| "56건 테스트 시간 90초 초과 시 완화" | **불필요** — 223 ms |
| "`uuid` 명시 의존성 추가 가능성" | **불필요** |
| "SO-SUEOP 서브 스토리지 CLSID 미확인" | `{00044214-0000-0000-C000-000000000046}`, 비-0 |

## 4. 검증

```
CARGO_INCREMENTAL=0 cargo test --profile release-test --test task4097_stage1_probe -- --nocapture
  running 1 test ... test result: ok. 1 passed; 0 failed  (0.22s)

CARGO_INCREMENTAL=0 cargo test --profile release-test --lib task3363_hwp3_embedded_ole_payload_extraction -- --nocapture
  running 1 test ... test result: ok. 1 passed; 0 failed  (0.01s)
```

측정 종료 후 임시 프로브(`tests/task4097_stage1_probe.rs`)와 임시 계측(`src/parser/hwp3/ole.rs`)을
제거해 `git status` 가 깨끗함을 확인했다.

## 5. 절차 기록

이슈 assignee 지정은 **권한 부족으로 실패**했다.

```text
gh issue edit 4097 --add-assignee johndoekim -R edwardkim/rhwp
  failed: GraphQL: johndoekim does not have the correct permissions to execute
          `ReplaceActorsForAssignable`
```

착수 시점 `gh pr list` 확인 결과 #4097 관련 열린 PR 없음, `4097` 관련 브랜치 없음.
