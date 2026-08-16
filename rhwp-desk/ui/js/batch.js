// batch 러너 — 폴더 드롭/선택 → 문서 N건 일괄 처리.
// 진행률은 작업 큐에, 호출 1건 = 카드 1장(저널 1:1)은 그대로 유지.
// 실패 문서는 격리해 계속 진행하고, 끝에 실패 목록을 요약한다.

import { runTool, listDocuments, basename, dirname } from "./api.js";

export class BatchRunner {
  /**
   * deps: { enginePath(), queue: {start,step,finish,isCancelled,progress},
   *         onEntry(entry), note(title, body), hasLayoutAnomaly() }
   */
  constructor(deps) {
    this.deps = deps;
  }

  /** 폴더 → 모드 선택 모달을 띄울 수 있게 문서 수를 미리 센다. */
  async prepare(dir) {
    const files = await listDocuments(dir);
    return { dir, files };
  }

  async run(dir, files, mode) {
    const d = this.deps;
    const label = { info: "메타 스윕", verify: "검증 스윕", pdf: "PDF 변환" }[mode] || mode;
    const q = d.queue.start(`${label}: ${basename(dir)} (${files.length}건)`, files.length * (mode === "verify" ? 3 : 1));
    const failed = [];
    let done = 0;

    const call = async (file, args, origin) => {
      const entry = await runTool(d.enginePath(), args, origin);
      d.onEntry(entry);
      if (entry.exitCode !== 0 && entry.exitCode !== 3) {
        throw new Error(`exit ${entry.exitCode}`);
      }
      return entry;
    };

    for (const file of files) {
      if (d.queue.isCancelled(q)) break;
      d.queue.step(q, basename(file));
      try {
        if (mode === "info") {
          await call(file, ["info", file, "--json"], "batch");
          d.queue.progress(q, ++done);
        } else if (mode === "verify") {
          for (const axis of ["hidden-text", "injection", "unicode"]) {
            await call(file, ["inspect", axis, file, "--json"], "batch");
            d.queue.progress(q, ++done);
          }
        } else if (mode === "pdf") {
          const out = `${dirname(file)}\\rhwp-pdf\\${basename(file).replace(/\.(hwp|hwpx)$/i, "")}.pdf`;
          await call(file, ["export-pdf", file, "-o", out, "--json"], "batch");
          d.queue.progress(q, ++done);
        }
      } catch (e) {
        failed.push(`${basename(file)} — ${String(e).slice(0, 120)}`);
        // 실패 격리: 다음 문서로 계속
        done = mode === "verify" ? Math.ceil(done / 3) * 3 : done;
        d.queue.progress(q, done, failed.length);
      }
    }

    const cancelled = d.queue.isCancelled(q);
    d.queue.finish(q, failed.length === 0 && !cancelled);
    d.note(
      `${label} ${cancelled ? "중단" : "완료"}`,
      `${files.length}건 중 실패 ${failed.length}건${failed.length ? ":\n" + failed.slice(0, 10).join("\n") : ""}` +
        (failed.length > 10 ? `\n… 외 ${failed.length - 10}건` : ""),
    );
  }
}
