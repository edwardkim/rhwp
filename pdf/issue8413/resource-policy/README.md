# PDF 자원 한도 제어군

외부 문서나 글꼴을 포함하지 않는 공개 합성 입력이다. 실제 사용자 원본 네 파일을
대신하는 시각 완료 증거가 아니다. [공통 작업 기록](../../../mydocs/working/task_m100_8413_implementation.md).

- `resource-objects-nonempty.pdf`: catalog의 private 배열에 60,000개 정수 값을 둔다.
  한 페이지의 `q Q`는 paint하지 않으며 객체 값 계수와 요청 경계를 확인한다.
- `resource-dense.pdf`: 한 페이지에 `q Q`를 320,000번 반복해 640,000개 명령을 만든다.
  화면 결과를 고정하지 않는 파싱 작업량 제어군이다.
- `resource-8k.pdf`: 7680×4320의 8비트 회색 이미지 하나를 두 페이지에서 재사용한다.
  모든 source sample은 137이며 쪽당 300×200pt, 이미지 CTM은 `[240 0 0 135 30 30]`이다.
  실제 해상도·pixel 저장과 가져오기/재열기를 검증하며 자동 축소하지 않는다.

입력 식별은 `manifest.json`을 따른다. 아래 generator는 동일 의미의 입력을 재생성한다.
PDF의 생성 ID는 달라질 수 있으므로 새 실행은 새 입력 SHA로 기록한다.

```bash
venv/bin/python pdf/issue8413/resource-policy/generate.py output/pdf-resource-controls
```

정식 Rust 검사는 `issue_8174_pdf_resource_policy`와 `issue_8174_pdf_operator_limits`의
합성 생성자를 사용한다. 위 파일은 실제 CLI·WASM·Studio 실행 제어군이다.
