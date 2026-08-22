---
kind: guide
status: active
canonical: mydocs/tech/investigations/issue-4963/hyperv_reproduction_guide.md
last_verified: 2026-08-23
---

# Issue #4963 Hyper-V Font Oracle 재현 가이드

## 1. 목적과 재현 수준

이 문서는 다른 개발자가 #4963의 **외부 checkpoint 제어 → Windows font 상태 구성 → interactive HWP
automation → PDF 관측 → checkpoint 복원**을 자신의 환경에서 다시 구현하도록 한다. 기존 보고서의
VM/checkpoint digest는 원 실행의 계보 증거이지 구축 설명서가 아니므로 이 가이드가 실행 정본이다.

재현 수준은 둘로 분리한다.

| 수준 | 요구사항 | 판정 |
| --- | --- | --- |
| 절차 재현 | Windows 11 guest, 지원되는 한컴, 합법적으로 확보한 font, 같은 fixture·state loop | 새 environment identity로 독립 결과 생성 |
| 결과 재현 | 위 조건과 함께 HWP executable·보안 DLL·font·fixture SHA-256과 한컴 build까지 일치 | 기존 selection·projection과 비교 가능 |

PDF에는 실행시각 등 metadata가 들어갈 수 있으므로 raw PDF file hash 동일성은 결과 재현의 필수조건이
아니다. font·glyph·advance·position·line을 canonicalize한 typesetting projection을 비교한다. 반대로
font bytes나 HWP build가 다르면 결과가 비슷해도 기존 acceptance profile을 재생했다고 주장하지 않는다.

## 2. 구성도와 책임 경계

```text
Hyper-V host PowerShell (외부 control plane)
  ├─ Standard checkpoint restore 전/후 강제
  ├─ PowerShell Direct로 guest 준비·manifest 회수
  └─ interactive user의 일회성 Scheduled Task 시작
       ↓
Windows 11 guest
  ├─ 한컴 2020 + 공식 Automation 보안 모듈
  ├─ local-only font source root
  ├─ exact-only / subst-only / none-related 상태
  └─ HWPX open → PDF export → path-free JSON
       ↓
Linux/WSL 또는 host 분석 환경
  ├─ pdf_oracle_observe.py
  └─ oracle_stage4_reproduction_compare.py
```

checkpoint restore는 반드시 guest 밖 Hyper-V host가 수행한다. guest가 자신의 복구 성공을 선언하는
구조는 정전·hang·악성 입력 때 신뢰할 수 없다. Microsoft의 [PowerShell Direct 요구사항][ps-direct]에
따라 VM은 host에 로컬로 실행되고, host 사용자는 Hyper-V 관리자이며, guest에는 암호가 있는 유효한
계정과 사용자 profile이 있어야 한다. 네트워크나 WinRM은 필요하지 않다.

[ps-direct]: https://learn.microsoft.com/en-my/windows-server/virtualization/hyper-v/powershell-direct

## 3. 필요한 자원

### 3.1 Hyper-V host

- Windows 11 Pro/Enterprise 또는 Hyper-V를 제공하는 동등한 Windows host
- 관리자 PowerShell과 `Hyper-V` module
- Windows 11 ISO, 정당한 guest 라이선스, 80 GiB 이상의 VM 저장 공간
- repository checkout과 결과를 보관할 owner-only local output root
- guest admin 계정의 `PSCredential`; 빈 암호는 PowerShell Direct 재현 계약에서 허용하지 않음

### 3.2 guest

- Windows 11, 한글 UI가 필수는 아니지만 culture와 system locale을 결과에 기록
- 한컴 2020 또는 조사할 명시적 세대; 설치·update 뒤 GUI 최초 실행과 정상 종료를 한 번 완료
- 한컴 공식 Automation 보안 승인 모듈
- interactive logon session 1개, 실행 직전 `Hwp.exe` 0개
- local-only font source root; contract에 선언된 상대 경로와 SHA-256을 유지

한컴 공식 [Automation 개발 가이드][hancom-automation]는 Automation용 보안 모듈을 내려받아
레지스트리에 등록하고 `RegisterModule("FilePathCheckDLL", "FilePathCheckerModuleExample")`을 호출하도록
안내한다. HwpObject용 값은 **그 interactive 계정의**
`HKCU\Software\HNC\HwpAutomation\Modules` 아래에 둔다. DLL bitness는 Hwp.exe와 맞추고, 다운로드한
파일의 SHA-256을 environment identity에 기록한다. 이번 기준 결과의 DLL hash는
`9ac5b97c47ac8aed1e8bca27a3eef39411361d8f68c262509f0c40a8f9d21bb6`이지만, 재배포하지 않는다.

[hancom-automation]: https://developer.hancom.com/en-us/hwpautomation

## 4. VM 최초 구축

다음은 host 관리자 PowerShell의 예다. 경로와 VM 이름은 local-only 값으로 바꾼다.

```powershell
$VmName = '<oracle-vm>'
$VmRoot = '<host-vm-root>'
$Iso = '<windows-11-iso>'
$Switch = '<hyper-v-switch>'

New-VM -Name $VmName -Generation 2 `
  -MemoryStartupBytes 8GB `
  -NewVHDPath (Join-Path $VmRoot 'oracle.vhdx') `
  -NewVHDSizeBytes 80GB `
  -SwitchName $Switch
Set-VMProcessor -VMName $VmName -Count 4
Set-VM -Name $VmName -AutomaticCheckpointsEnabled $false
Set-VM -Name $VmName -CheckpointType Standard
Set-VMFirmware -VMName $VmName -EnableSecureBoot On `
  -SecureBootTemplate MicrosoftWindows
$dvd = Add-VMDvdDrive -VMName $VmName -Path $Iso -Passthru
Set-VMFirmware -VMName $VmName -FirstBootDevice $dvd
```

CPU·RAM 수치는 최소 실행 recipe이지 Oracle identity를 대신하지 않는다. 실제 VM generation, VMId,
processor count, memory, Windows build를 local attestation에 기록한다. 이 실험은 실행 중인 interactive
desktop 상태까지 되돌려야 하므로 Microsoft가 설명하는 [Standard checkpoint][checkpoints]를 사용한다.
Production checkpoint나 자동 checkpoint로 조용히 대체하지 않는다.

[checkpoints]: https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/checkpoints

## 5. guest 기준선 만들기

순서를 바꾸지 않는다.

1. Windows 설치와 update를 완료한다.
2. 암호가 있는 local admin 계정으로 로그인해 profile을 만든다.
3. 한컴을 설치·update하고 GUI를 한 번 열어 초기화·약관·update migration을 끝낸 뒤 정상 종료한다.
4. 공식 Automation 보안 DLL을 guest local-only 경로에 놓고 같은 계정의 HKCU에 등록한다.
5. `scripts/oracle_stage4_windows_interactive.ps1`가 `RegisterModule=true`, HWPX `Open=true`, 1쪽 이상과
   비어 있지 않은 text를 반환하는지 read-only canary로 확인한다.
6. font source root를 guest에 놓되 Windows font registry나 Fonts folder에는 아직 설치하지 않는다.
7. 아래 repository 파일과 대상 fixture를 guest 작업 root에 복사한다.
8. interactive 계정은 로그인된 상태로 두고 HWP process와 일회성 task가 0개인지 확인한다.
9. ambient manifest를 두 번 실행해 digest가 같은지 확인한다.
10. 그 상태에서 Standard checkpoint를 하나만 만들고 VMId·checkpoint Id를 local-only 원장에 고정한다.

필요한 repository 파일은 다음과 같다.

```text
scripts/oracle_stage4_windows_manifest.ps1
scripts/oracle_stage4_windows_font_state.ps1
scripts/oracle_stage4_windows_interactive.ps1
scripts/oracle_stage4_windows_task.ps1
mydocs/tech/investigations/issue-4963/fixtures/oracle_typesetting_fixture.hwpx
```

보안 모듈 등록 예시는 guest의 interactive 계정 PowerShell에서 실행한다.

```powershell
$ModuleName = 'FilePathCheckerModuleExample'
$ModuleDll = '<guest-local-path>\FilePathCheckerModuleExample.dll'
$Key = 'HKCU:\Software\HNC\HwpAutomation\Modules'
New-Item -Path $Key -Force | Out-Null
New-ItemProperty -Path $Key -Name $ModuleName `
  -Value $ModuleDll -PropertyType String -Force | Out-Null
```

checkpoint 생성과 identity 고정은 host에서 수행한다.

```powershell
$BaselineName = '<oracle-interactive-baseline>'
Set-VM -Name $VmName -CheckpointType Standard
Checkpoint-VM -VMName $VmName -SnapshotName $BaselineName
$vm = Get-VM -Name $VmName
$checkpoint = Get-VMSnapshot -VMName $VmName -Name $BaselineName
if ($vm.CheckpointType -ne 'Standard') { throw 'Standard checkpoint required' }
$vm.Id
$checkpoint.Id
```

raw VM 이름·GUID·path는 공개 profile에 넣지 않고 SHA-256 identity로 투영한다. 로컬 control plane은
restore 직전에 raw Id가 원장과 정확히 일치하는지 확인해야 한다.

## 6. baseline preflight

host에서 guest credential을 받고 PowerShell Direct를 확인한다.

```powershell
$Credential = Get-Credential
$vm = Get-VM -Name $VmName -ErrorAction Stop
$checkpoint = Get-VMSnapshot -VMName $VmName -Name $BaselineName -ErrorAction Stop
if ($vm.Id -ne [guid]'<expected-vm-id>') { throw 'VM identity drift' }
if ($checkpoint.Id -ne [guid]'<expected-checkpoint-id>') {
  throw 'Checkpoint identity drift'
}

Restore-VMCheckpoint -VMName $VmName -Name $BaselineName -Confirm:$false
if ((Get-VM -Name $VmName).State -eq 'Off') { Start-VM -Name $VmName }
$probe = Invoke-Command -VMName $VmName -Credential $Credential -ScriptBlock {
  [ordered]@{
    user = (Get-CimInstance Win32_ComputerSystem).UserName
    hwpProcessCount = @(Get-Process Hwp -ErrorAction SilentlyContinue).Count
    vmSession = (Get-Service vmicvmsession).Status.ToString()
    os = (Get-CimInstance Win32_OperatingSystem).Caption
  }
}
if (-not $probe.user) { throw 'Interactive logon session is required' }
if ($probe.hwpProcessCount -ne 0) { throw 'Hwp.exe baseline is dirty' }
```

PowerShell Direct는 valid guest credential이 필요하며, interactive HWP COM은 별도 desktop session이
필요하다. 그래서 font 상태 구성·manifest는 PowerShell Direct로 실행하고 HWP export만 이미 로그인한
사용자의 일회성 Scheduled Task로 실행한다.

baseline manifest는 guest root와 font root를 매개변수로 전달해 두 번 얻는다.

```powershell
$ManifestScript = '<host-repo>\scripts\oracle_stage4_windows_manifest.ps1'
$FontRoot = '<guest-font-source-root>'
$m1 = Invoke-Command -VMName $VmName -Credential $Credential `
  -FilePath $ManifestScript -ArgumentList $FontRoot,$false | ConvertFrom-Json
$m2 = Invoke-Command -VMName $VmName -Credential $Credential `
  -FilePath $ManifestScript -ArgumentList $FontRoot,$false | ConvertFrom-Json
if ($m1.manifestSha256 -ne $m2.manifestSha256) { throw 'Baseline manifest drift' }
if ($m1.unrelatedProjectionSha256 -ne $m2.unrelatedProjectionSha256) {
  throw 'Baseline unrelated projection drift'
}
if ($m1.managedInstalledByExactBytes.Count -ne 0) {
  throw 'Managed font already exists in baseline'
}
```

## 7. three-state 실행

각 물리 상태는 **서로 이어서 실행하지 않는다**. `exact-only`, `subst-only`, `none-related` 각각에 대해
다음 loop를 독립 실행한다.

1. host가 exact VM/checkpoint Id를 다시 확인한다.
2. baseline checkpoint를 restore한다.
3. PowerShell Direct와 interactive logon, HWP 0개를 확인한다.
4. `font-state-spec.json`을 만들고 guest에서 font-state helper를 실행한다.
5. state manifest의 managed set과 unrelated projection을 확인한다.
6. `task-spec.json`을 만들고 interactive user의 일회성 Scheduled Task를 실행한다.
7. PDF·interactive JSON·ambient manifest를 owner-only output으로 회수한다.
8. task와 HWP가 0개인지 확인한다.
9. `finally`에서 baseline checkpoint를 다시 restore한다.
10. recovered manifest가 baseline과 같은지 확인하고 함께 보관한다.

local-only `font-state-spec.json` 형식은 다음과 같다. none-related에서는 `fonts`가 빈 배열이어야 한다.

```json
{
  "schemaVersion": 1,
  "kind": "font-oracle-hyperv-state-spec",
  "issue": 4963,
  "physicalState": "exact-only",
  "fonts": [
    {
      "source": "<guest-font-source-file>",
      "sha256": "<expected-font-sha256>"
    }
  ]
}
```

guest admin PowerShell Direct session에서 상태를 구성한다.

```powershell
& C:\rhwp-oracle\oracle_stage4_windows_font_state.ps1 `
  -StateSpec C:\rhwp-oracle\font-state-spec.json `
  -ResultOutput C:\rhwp-oracle\font-state-result.json `
  -CheckpointRestoreAttested
```

helper는 Hyper-V guest, HWP 0개, source hash, 허용 상태와 font 수를 fail-closed로 검사한다. font 제거
명령은 제공하지 않는다. 실험이 중단돼도 외부 checkpoint restore만이 상태 제거 수단이다.

interactive task spec은 다음 형식이다. JSON을 사용하므로 한글 face 이름과 배열이 Task Scheduler의
명령행 tokenization을 통과한다.

```json
{
  "schemaVersion": 1,
  "kind": "font-oracle-hyperv-task-spec",
  "issue": 4963,
  "runner": "C:\\rhwp-oracle\\oracle_stage4_windows_interactive.ps1",
  "source": "C:\\rhwp-oracle\\fixture.hwpx",
  "pdfOutput": "C:\\rhwp-oracle\\state.pdf",
  "resultOutput": "C:\\rhwp-oracle\\state.interactive.json",
  "documentFace": "<document-face>",
  "queueRank": 8,
  "expectedSourceSha256": "<fixture-sha256>",
  "probeFaces": ["<english-sfnt-alias>", "<substitution-face>"],
  "fontResourceFiles": ["<installed-or-source-font-path>"],
  "securityModuleName": "FilePathCheckerModuleExample"
}
```

일회성 task는 로그인된 사용자 token에서 실행한다.

```powershell
$TaskName = 'rhwp-4963-oracle-once'
$InteractiveUser = Invoke-Command -VMName $VmName -Credential $Credential `
  -ScriptBlock { (Get-CimInstance Win32_ComputerSystem).UserName }
Invoke-Command -VMName $VmName -Credential $Credential -ScriptBlock {
  param($TaskName, $InteractiveUser)
  $action = New-ScheduledTaskAction -Execute 'powershell.exe' -Argument (
    '-NoProfile -ExecutionPolicy Bypass -File ' +
    '"C:\rhwp-oracle\oracle_stage4_windows_task.ps1" ' +
    '-StateSpec "C:\rhwp-oracle\task-spec.json"'
  )
  $principal = New-ScheduledTaskPrincipal -UserId $InteractiveUser `
    -LogonType Interactive -RunLevel Highest
  Register-ScheduledTask -TaskName $TaskName -Action $action `
    -Principal $principal -Force | Out-Null
  Start-ScheduledTask -TaskName $TaskName
} -ArgumentList $TaskName,$InteractiveUser
```

result JSON이 생길 때까지 상한 120초로 polling하고, `status=observed`, input SHA, `Open=true`, page/text
non-empty, `securityModuleRegistered=true`, PDF hash를 확인한다. timeout이면 task/HWP를 정리한 뒤 다음
상태로 진행하지 말고 `finally` restore와 recovered manifest 검사부터 수행한다.

host control plane의 각 상태는 최소한 다음 `try/finally` 형태여야 한다. 아래 함수 이름은 이 절 앞의
restore·manifest·task 명령을 함수로 감싼 것이며, 중요한 계약은 성공 여부와 무관하게 `finally`가 같은
raw checkpoint Id를 다시 대사하고 복원한다는 점이다.

```powershell
$stateSucceeded = $false
$session = $null
try {
  Assert-OracleIdentity -VMName $VmName `
    -ExpectedVmId $ExpectedVmId `
    -CheckpointName $BaselineName `
    -ExpectedCheckpointId $ExpectedCheckpointId
  Restore-OracleBaseline -VMName $VmName -CheckpointName $BaselineName
  $session = New-OraclePowerShellDirectSession `
    -VMName $VmName -Credential $Credential -TimeoutSeconds 120
  Assert-OracleGuestBaseline -Session $session

  # font-state-spec.json과 task-spec.json은 상태마다 새로 쓴다.
  Invoke-OracleFontState -Session $session -PhysicalState $PhysicalState
  $stateManifest = Get-OracleManifest -Session $session
  Assert-OracleManagedSet -Manifest $stateManifest `
    -ExpectedSha256 $ExpectedManagedSha256
  Assert-OracleUnrelatedProjection -Manifest $stateManifest `
    -ExpectedSha256 $BaselineUnrelatedProjectionSha256

  Invoke-OracleInteractiveTask -Session $session -TimeoutSeconds 120
  Copy-OracleEvidenceFromGuest -Session $session -Destination $StateOutput
  Assert-OracleRunEnvelope -StateOutput $StateOutput
  $stateSucceeded = $true
}
finally {
  if ($null -ne $session) {
    Invoke-Command -Session $session -ScriptBlock {
      Unregister-ScheduledTask -TaskName 'rhwp-4963-oracle-once' `
        -Confirm:$false -ErrorAction SilentlyContinue
      Get-Process Hwp -ErrorAction SilentlyContinue | Stop-Process -Force
    }
    Remove-PSSession $session
  }

  Assert-OracleIdentity -VMName $VmName `
    -ExpectedVmId $ExpectedVmId `
    -CheckpointName $BaselineName `
    -ExpectedCheckpointId $ExpectedCheckpointId
  Restore-OracleBaseline -VMName $VmName -CheckpointName $BaselineName
  $recoverySession = New-OraclePowerShellDirectSession `
    -VMName $VmName -Credential $Credential -TimeoutSeconds 120
  try {
    $recovered = Get-OracleManifest -Session $recoverySession
    Save-LocalOnlyJson $recovered `
      (Join-Path $StateOutput 'recovered.ambient-manifest.json')
    if ($recovered.manifestSha256 -ne $BaselineManifestSha256) {
      throw 'Final restore did not recover the baseline manifest'
    }
    if ($recovered.unrelatedProjectionSha256 -ne
        $BaselineUnrelatedProjectionSha256) {
      throw 'Final restore changed the unrelated font projection'
    }
    if ($recovered.managedInstalledByExactBytes.Count -ne 0 -or
        $recovered.hwpProcessCount -ne 0) {
      throw 'Final restore left managed fonts or Hwp.exe behind'
    }
  }
  finally {
    Remove-PSSession $recoverySession
  }
}
if (-not $stateSucceeded) { throw 'Oracle state failed after verified recovery' }
```

함수 구현은 이름이 아니라 다음 동작으로 판정한다.

- `Assert-OracleIdentity`: `Get-VM`·`Get-VMSnapshot` raw GUID와 checkpoint type `Standard` 대사
- `Restore-OracleBaseline`: `Restore-VMCheckpoint -Confirm:$false`, 필요하면 `Start-VM`, PowerShell
  Direct가 열릴 때까지 bounded retry
- `Invoke-OracleFontState`: 이 저장소의 `oracle_stage4_windows_font_state.ps1` 실행
- `Get-OracleManifest`: 이 저장소의 `oracle_stage4_windows_manifest.ps1` 결과를 JSON으로 parse
- `Invoke-OracleInteractiveTask`: `oracle_stage4_windows_task.ps1`을 interactive principal로 1회 실행
- `Copy-OracleEvidenceFromGuest`: PDF·run JSON·manifest만 owner-only output으로 회수

이 reference loop에서 복구 실패는 원래 실행 실패보다 우선하는 중단 사유다. `finally` 안의 강제 HWP
종료는 곧바로 baseline restore가 뒤따르는 disposable guest에만 허용하며 현재 host나 다른 VM에
적용하지 않는다.

모든 파일 복제는 persistent PowerShell Direct session의 `Copy-Item -ToSession/-FromSession`을 사용하거나
Hyper-V Guest Service Interface를 명시적으로 활성화한 `Copy-VMFile`을 사용한다. 후자를 쓸 경우
Microsoft의 [integration services 절차][integration-services]와 [Copy-VMFile 명세][copy-vm-file]에
따라 host와 guest 양쪽 service 상태를 확인한다.

[integration-services]: https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/manage/manage-hyper-v-integration-services
[copy-vm-file]: https://learn.microsoft.com/en-us/powershell/module/hyper-v/copy-vmfile

## 8. 상태별 managed set

rank 8 재현 예는 다음과 같다. 다른 target은 수행계획의 exact/substitution SHA를 사용한다.

| 상태 | 설치 font SHA-256 | 기대 PDF 선택 |
| --- | --- | --- |
| exact-only | `e3ee21a8…fb18` | 환경에 따라 관찰, 기존 기준은 `KoPubWorldBatangLight` |
| subst-only | `069494cc…c84f` | 환경에 따라 관찰, 기존 기준은 `HCRBatang-Bold` |
| none-related | 없음 | 환경에 따라 관찰, 기존 기준은 `HCRBatang-Bold` |

기대 PDF 선택은 통과조건이 아니라 기존 결과와의 비교값이다. 새 환경이 다르면 실패로 숨기지 않고
environment identity와 실제 subset을 기록한다. 통과조건은 선언된 managed set, 동일 fixture, restore,
privacy와 관측의 자기일관성이다.

## 9. PDF 분석과 독립 재현 비교

각 상태 PDF를 같은 도구로 분석한다.

```bash
python3 scripts/pdf_oracle_observe.py \
  --pdf-root <state-directory> \
  --pdf <state>.pdf \
  --output-root <state-directory> \
  --output <state>.pdf-observation.json
```

세 상태 디렉터리는 다음 이름을 권장한다.

```text
<evidence-root>/rank8-exact-only/exact-only.*
<evidence-root>/rank8-subst-only/subst-only.*
<evidence-root>/rank8-none-related/none-related.*
<evidence-root>/<state-dir>/recovered.ambient-manifest.json
```

local-only compare config에는 자신의 baseline digest와 managed set을 넣는다. 기존 원 실행의 digest를
복사하면 안 된다.

```json
{
  "schemaVersion": 1,
  "kind": "font-oracle-hyperv-reproduction-config",
  "issue": 4963,
  "queueRank": 8,
  "documentFace": "KoPubWorld바탕체 Light",
  "fixtureSha256": "<fixture-sha256>",
  "baseline": {
    "manifestSha256": "<local-baseline-manifest-sha256>",
    "unrelatedProjectionSha256": "<local-unrelated-projection-sha256>"
  },
  "states": {
    "exact-only": {
      "directory": "rank8-exact-only",
      "stem": "exact-only",
      "managedFontSha256": ["<exact-font-sha256>"]
    },
    "subst-only": {
      "directory": "rank8-subst-only",
      "stem": "subst-only",
      "managedFontSha256": ["<subst-font-sha256>"]
    },
    "none-related": {
      "directory": "rank8-none-related",
      "stem": "none-related",
      "managedFontSha256": []
    }
  }
}
```

비교기는 raw path를 출력하지 않고 각 파일 hash, PDF font, page/line/glyph 수, typesetting projection과
exact/subst 대 none 비교만 남긴다.

```bash
python3 scripts/oracle_stage4_reproduction_compare.py \
  --evidence-root <local-evidence-root> \
  --config <local-compare-config.json> \
  --output-root <local-output-root> \
  --output reproduction-summary.json
```

`oracle_stage4_profile.py`와 `oracle_stage5_rank8_profile.py`는 기존 acceptance evidence의 raw file hash를
고정한 **공개 투영기**다. 새 개발자의 raw PDF를 그 파일에 덮어쓰거나 hash 상수를 고치지 않는다. 새
실행은 위 reproduction summary로 독립 검증한 뒤, 메인테이너가 새 environment identity를 승인할 때만
별도 profile로 승격한다.

## 10. 실패 복구와 합격 판정

다음 중 하나라도 발생하면 현재 상태는 실패이며 다음 상태를 실행하지 않는다.

- VMId/checkpoint Id 불일치 또는 checkpoint type이 Standard가 아님
- PowerShell Direct credential 실패, interactive user 없음, HWP baseline process 존재
- security module 등록 실패, HWPX open false, empty-open guard 실패
- fixture/font SHA 불일치 또는 managed font 수 불일치
- state 간 unrelated font projection drift
- run 뒤 HWP/task 잔존
- final restore 뒤 baseline manifest·unrelated projection 불일치

실패 경로에서도 `finally` restore를 먼저 수행한다. 복구가 확인되지 않으면 VM을 격리하고 checkpoint를
새 기준선으로 덮어쓰지 않는다. 첫 실행 실패 뒤 성공한 상태만 골라 acceptance ladder로 조립하지 않고,
각 상태의 restore 증거를 함께 보존한다.

재현 실행의 합격 조건은 다음과 같다.

1. 세 상태가 같은 fixture SHA를 사용한다.
2. 상태별 managed set이 config와 정확히 같다.
3. unrelated projection이 세 상태와 recovered baseline에서 같다.
4. 각 상태 뒤 baseline manifest, HWP 0개가 회복된다.
5. run JSON·PDF·observation hash가 연결된다.
6. observed PDF font·glyph·advance·line/page를 기대값과 무관하게 보존한다.
7. font bytes, credential, VM/path, private corpus identity를 공개 summary에 넣지 않는다.

이 일곱 조건을 만족하면 결과가 기존 한컴 2020 기준과 달라도 **절차 재현 성공·Oracle 결과 차이**다.
환경 차이를 버전 분기로 숨기지 않고 후속 분석 대상으로 남긴다.
