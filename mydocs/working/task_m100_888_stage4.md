# Task m100 #888 Stage 4 - rhwp-studio HWPX 저장 활성화

## 1. 목적

Stage 1~3에서 `hwpx -> IR -> clone -> hwp save` 코어 변환은 `basic-table-01.hwpx`, `expense_report.hwpx` 기준으로 작업지시자 판정을 통과했다.

현재 rhwp-studio UI에서는 HWPX 출처 문서의 `file:save`, `file:save-as`가 비활성화되어 있어 WASM 빌드 후 브라우저에서 저장 결과를 직접 판정할 수 없다.

## 2. 변경 원칙

- HWPX 원본 파일을 직접 덮어쓰지 않는다.
- HWPX 출처 문서는 저장 시 `exportHwp()`로 HWP 바이트를 생성한다.
- 저장 파일명은 `.hwpx`가 아니라 `.hwp`로 제안한다.
- 최초 HWPX 저장 시 원본 `.hwpx` handle은 우회하고 save picker를 띄운다.
- 사용자가 `.hwp`로 저장한 뒤에는 해당 `.hwp` handle에 후속 저장할 수 있다.

## 3. 수정 파일

- `rhwp-studio/src/command/commands/file.ts`
  - `file:save`, `file:save-as`의 HWPX 비활성 조건 제거
  - HWPX/HWP 모두 `exportHwp()` 저장으로 통일
  - HWPX 원본 handle 우회 및 `.hwp` 저장 파일명 정규화
- `rhwp-studio/src/command/file-system-access.ts`
  - 열기 picker는 `.hwp`, `.hwpx` 허용
  - 저장 picker는 `.hwp`만 제안
- `rhwp-studio/src/main.ts`
  - HWPX 저장 비활성 안내를 HWP 변환 저장 안내로 변경
- `rhwp-studio/src/ui/menu-bar.ts`
  - HWPX 저장 비활성 툴팁 제거
- `rhwp-studio/src/command/types.ts`
  - sourceFormat 주석 현행화

## 4. 검증

```text
npm run build
```

결과:

- TypeScript 컴파일 통과
- Vite production build 통과
- 기존 dev server HMR 반영 확인

## 5. 작업지시자 판정 결과

### expense_report.hwpx 편집 후 HWP 저장

작업지시자 환경에서 다음 절차를 확인했다.

1. `expense_report.hwpx`를 rhwp-studio로 열기
2. 빈 셀에 간단한 내용 입력
3. 저장 실행
4. `.hwp`로 저장됨 확인
5. 저장된 HWP를 한컴 에디터로 열기
6. 저장된 HWP를 rhwp-studio로 다시 열기

판정:

- 한컴 에디터에서 파일 손상 없이 열림
- rhwp-studio에서 추가 입력한 내용이 한컴 에디터에서 정상 출력됨
- 저장된 HWP를 rhwp-studio에서 다시 열어도 정상 열림

이 판정은 `hwpx -> IR 편집 -> hwp save -> 한컴/rhwp-studio reload` 왕복 경로의 기본 성공 케이스로 본다.

### basic-table-01.hwpx 왕복 저장

작업지시자 환경에서 다음 절차를 확인했다.

1. rhwp-studio에서 `basic-table-01.hwpx` 열기
2. 빈 셀/기존 셀에 텍스트 입력
3. `.hwp` 저장
4. 한컴 에디터로 재열기
5. rhwp-studio로 재열기

판정:

- 통과

### expense_report.hwpx 엣지 케이스

작업지시자 환경에서 다음 절차를 확인했다.

1. 병합 셀 안 텍스트 입력
2. 표 밖 일반 문단 편집
3. 셀 안 텍스트 스타일 변경
4. 저장 후 한컴 에디터/rhwp-studio로 재열기

판정:

- 통과

### 원본 보호 검증

작업지시자 환경에서 다음 항목을 확인했다.

1. `.hwpx` 원본 파일이 HWP 바이트로 덮어써지지 않는지 확인
2. 저장 picker가 `.hwp`를 제안하는지 확인

판정:

- 통과

## 6. 완료 판단

dev server:

```text
http://localhost:7700/
```

Stage 4 기준으로 다음 경로가 통과했다.

- `basic-table-01.hwpx` 편집 후 HWP 저장/reload
- `expense_report.hwpx` 편집 후 HWP 저장/reload
- 병합 셀, 표 밖 문단, 셀 텍스트 스타일 변경
- 원본 `.hwpx` 보호
- 저장 picker `.hwp` 제안

다음 단계는 변경분 정리, 커밋, PR 또는 `local/devel` push 준비다.
