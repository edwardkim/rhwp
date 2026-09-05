# 24 — export-render-tree (후속)

`render-diff` 가 이미 같은 트리로 변위를 잰다. 사람이 전후 JSON 을
직접 diff 하고 싶을 때만 후속으로 친다. 새 명령이 아니다.

```bash
rhwp export-render-tree 전.hwp -p 0 > before.json
rhwp export-render-tree 후.hwp -p 0 > after.json
```

bbox 좌표가 들어 있다. 자동화 게이트의 1차는 여전히 `render-diff`
종료 코드와 TSV 다. 이 덤프는 좁힌 뒤의 정밀 대조다.

`export-svg --debug-overlay -p N` 은 문단/표 경계를 그림으로 겹친다.
숫자 다음의 눈 검증.

이 스킬이 이 명령을 발명한 것이 아니다. 이미 CLI 에 있다. 1차 판정은
항상 `render-diff` 의 status 와 TSV 다.
