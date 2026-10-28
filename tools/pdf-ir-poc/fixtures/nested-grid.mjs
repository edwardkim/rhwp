import fs from "node:fs/promises";

// Independent three-level geometry; byte offsets and stream length are computed, not hardcoded.
export async function nestedPdf() {
  const content = await fs.readFile(new URL("./nested-grid.content", import.meta.url));
  const objects = [
    Buffer.from("<< /Type /Catalog /Pages 2 0 R >>"),
    Buffer.from("<< /Type /Pages /Kids [3 0 R] /Count 1 >>"),
    Buffer.from("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>"),
    Buffer.from("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"),
    Buffer.concat([Buffer.from(`<< /Length ${content.length} >>\nstream\n`), content, Buffer.from("\nendstream")])
  ];
  let pdf = Buffer.from("%PDF-1.4\n");
  const offsets = [0];
  for (const [i, object] of objects.entries()) {
    offsets.push(pdf.length);
    pdf = Buffer.concat([pdf, Buffer.from(`${i + 1} 0 obj\n`), object, Buffer.from("\nendobj\n")]);
  }
  const xref = pdf.length;
  const entries = offsets.slice(1).map(offset => `${String(offset).padStart(10,"0")} 00000 n \n`).join("");
  return Buffer.concat([pdf, Buffer.from(`xref\n0 6\n0000000000 65535 f \n${entries}trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`)]);
}
