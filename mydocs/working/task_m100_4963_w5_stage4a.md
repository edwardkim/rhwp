---
kind: working-note
status: completed
issue: 4963
stage: W5-4A
last_verified: 2026-08-22
---

# Task M100 #4963 W5 Stage 4A — disposable 계약·preflight

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../plans/task_m100_4963.md)
- **브랜치**: `task_m100_4963`
- **단계 상태**: W5-4A 완료, W5-4B 실제 VM/checkpoint 준비 대기

## 1. 결론

W5-4 승인에 따라 실제 font 상태를 바꾸기 전에 현재 Windows가 복구 가능한 disposable 환경인지
read-only로 검사했다. Hyper-V 명령은 설치되어 있지만 현재 세션은 VM inventory를 열람할 권한이 없고,
식별 가능한 checkpoint와 restore 검증도 없다. WSL2는 Windows 호스트 font·registry·한컴 cache를
되돌리는 snapshot이 아니므로 현재 호스트는 `qualified=false`, `mutationAllowed=false`다.

현재 호스트를 억지로 사용하지 않고 W5-4A에서 세 canary의 동일 입력, 5-state disposition, 외부
snapshot attestation과 ambient font delta validator를 먼저 완성했다. font 설치·제거, registry 변경,
한컴 bundled font 변경, private corpus 접근과 remote upload는 모두 0건이다.

## 2. 세 canary와 document substitution

| 순서 | exact face | fixture-declared substFont | exact source |
| ---: | --- | --- | --- |
| 1 | 문체부 바탕체 | KoPubWorld바탕체 Light | local-only SFNT |
| 13 | 휴먼명조 | KoPubWorld바탕체 Light | local-only SFNT + 기존 HFT evidence |
| 7 | KoPubWorld돋움체 Light | KoPubWorld바탕체 Light | 공식 public source local-only |

W5-2 generator에 선택적 `--subst-face`를 추가했다. exact face의 7개 언어군 font entry에 같은
`<hh:substFont>`를 명시하며 기본 옵션이 없으면 기존 rank 1 fixture bytes와 manifest가 변하지 않는다.
세 W5-4 fixture는 각각 두 번 생성했을 때 byte exact였고 font bytes는 포함하지 않는다.

| rank | HWPX SHA-256 | manifest SHA-256 | semantic SHA-256 |
| ---: | --- | --- | --- |
| 1 | `deb4566e…ca511` | `60a5a31c…36dea3` | `fbd8ae46…848b0ff` |
| 13 | `a6dbe726…3f124` | `1e10ab6c…89b515` | `8148150c…77f39f` |
| 7 | `1cc8062c…36f9e` | `38778e8e…fd9df` | `e19b5842…6b61970` |

`substFont`는 이 synthetic fixture가 직접 선언한 document-substitution 관계일 뿐 identity, alias 또는
official successor가 아니다. 세 exact face 모두 직접 publisher/byte lineage가 있는 successor를 아직
찾지 못했으므로 successor-only 질문은 `not-provided`다.

## 3. 5개 질문과 3개 고유 실행

| 질문 | 물리 상태 | managed font |
| --- | --- | --- |
| exact-installed | exact-only | exact만 있음 |
| exact-removed | none-related | exact·subst 모두 없음 |
| document-subst-font-only | subst-only | fixture-declared subst만 있음 |
| curated-official-successor-only | 실행하지 않음 | direct anchor 없음 |
| all-related-fonts-missing | none-related | exact·subst 모두 없음 |

`exact-removed`와 `all-related-fonts-missing`은 현재 증명된 managed related set에서 같은 물리 상태다.
같은 PDF를 두 번 새 증거인 것처럼 만들지 않고 하나의 execution id가 두 질문을 충족한다고 명시한다.
향후 alias 또는 successor가 직접 증명되어 related set이 늘어나면 두 상태는 자동으로 분리해야 한다.

## 4. snapshot·manifest 보호 불변식

실제 W5-4B 실행은 다음을 모두 만족해야 한다.

1. Hyper-V/VMware/VirtualBox 중 하나의 VM identity와 baseline snapshot identity를 hash로 고정한다.
2. snapshot restore는 되돌려지는 guest 내부가 아니라 외부 control plane이 수행한다.
3. 각 고유 상태 실행 전에 baseline을 복구하고 실행 후에도 다시 복구한다.
4. baseline font manifest와 복구 후 manifest가 byte-equivalent digest여야 한다.
5. target exact와 fixture-declared subst 항목을 제외한 unrelated font projection은 모든 상태에서 같다.
6. 각 상태는 새 HWP process와 reboot 또는 명시적 font-cache refresh를 기록한다.
7. managed set 밖의 font가 변하거나 input HWPX hash가 달라지면 해당 run을 거부한다.
8. managed TTF 제거 뒤에도 exact face가 readback되면 bundled HFT 또는 unmanaged source로 판정하고
   missing state 성공을 주장하지 않는다.

마지막 규칙은 특히 `휴먼명조`에서 중요하다. 한컴 2020 bundle이 같은 이름의 HFT를 제공한다면 Windows
TTF를 제거해도 exact face가 남을 수 있다. 이 경우 font bundle을 손상시키지 않고
`blocked-immutable-or-unmanaged-font`로 기록한다.

## 5. 검증 결과

```text
python3 -m unittest -v scripts.tests.test_oracle_stage2
tests 7, pass 7, fail 0

python3 -m unittest -v scripts.tests.test_oracle_stage4
tests 8, pass 8, fail 0

python3 scripts/oracle_stage4_contract.py check
ok true, targets 3, uniquePhysicalStatesPerTarget 3,
currentHostQualified false
```

negative control은 unqualified host의 mutation 허용, snapshot 외부 제어 누락, restore manifest 불일치,
입력 hash drift, unrelated ambient font drift, managed state 오염과 근거 없는 successor 실행을 모두
거부했다.

## 6. W5-4B 시작 조건

다음 절편은 외부 control plane에서 접근 가능한 disposable Windows VM과 baseline checkpoint가 있어야
한다. VM 이름·host 이름·절대 path는 공개하지 않고 identity digest만 기록한다. 실제 restore probe로
baseline manifest가 되돌아오는 것을 먼저 확인한 다음에만 target exact/subst font 설치·제거 runner를
작성·실행한다. 현재 호스트에서는 그 절차를 수행하지 않는다.
