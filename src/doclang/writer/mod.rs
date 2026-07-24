//! DocLang v0.6 XML writer.
//!
//! Consumes a [`SirDocument`] (the rhwp-agnostic Semantic IR) and produces a
//! DocLang XML string.  Output is built by manual string concatenation so that
//! raw-payload elements (`<formula>`, `<custom>`) keep precise control over
//! escaping; see [`escape`] for the rationale.
//!
//! ## Element-head ordering
//!
//! DocLang elements place a fixed-order "element head" before their content:
//! `label → thread → xref/href → layer → location → caption → custom`.  This
//! v1 writer only ever emits the `thread` and `caption` members of that head;
//! the others belong to features not yet implemented (xref/href links,
//! layering, v2 `<location>`).  Where the head is emitted it follows that order.
//!
//! ## Whitespace
//!
//! `xml:space="preserve"` is intentionally not emitted (see [`escape`]): the
//! adapter trim-normalises paragraph whitespace, so the writer treats element
//! content whitespace as insignificant.

pub(crate) mod custom;
pub mod escape;
pub mod inline;
pub(crate) mod otsl;

use crate::doclang::error::ConvertError;
use crate::doclang::ir::block::{Block, HeaderFooterApply, ListItem};
use crate::doclang::ir::formula::Formula;
use crate::doclang::ir::prov::{Location, LocationMap, Prov, LOCATION_RESOLUTION};
use crate::doclang::ir::{Section, SirDocument};
use crate::doclang::loss::{LossEntry, LossKind, LossReport};
use crate::doclang::options::{ConvertOptions, Mode};
use crate::doclang::resources::resolve_resource;

use custom::{record_loss, write_custom_payload, write_lost_as_custom};
use escape::{escape_attr, escape_text};
use inline::write_inlines;

/// Serialise a Semantic IR document to DocLang v0.6 XML.
///
/// The root element is `<doclang version="…">` using
/// [`SirDocument::doclang_version`].  Any information that cannot be represented
/// is appended to `loss`.
pub fn write_doclang(
    doc: &SirDocument,
    opts: &ConvertOptions,
    loss: &mut LossReport,
) -> Result<String, ConvertError> {
    write_doclang_with_locations(doc, opts, &LocationMap::new(), loss)
}

/// Serialise a Semantic IR document to DocLang v0.6 XML, emitting v2
/// `<location>` head elements for every block whose [`Prov`] resolves a box in
/// `locs`.
///
/// When `opts.with_location` is `false` (or `locs` is empty) this is
/// byte-identical to [`write_doclang`]: no `<location>` is ever emitted, so the
/// default-off path is a strict superset-free no-op.
pub fn write_doclang_with_locations(
    doc: &SirDocument,
    opts: &ConvertOptions,
    locs: &LocationMap,
    loss: &mut LossReport,
) -> Result<String, ConvertError> {
    let mut out = String::new();

    out.push_str("<doclang version=\"");
    out.push_str(&escape_attr(doc.doclang_version));
    out.push_str("\">");

    for (si, section) in doc.sections.iter().enumerate() {
        write_section(&mut out, section, si, opts, locs, loss);
    }

    out.push_str("</doclang>");
    Ok(out)
}

fn write_section(
    out: &mut String,
    section: &Section,
    si: usize,
    opts: &ConvertOptions,
    locs: &LocationMap,
    loss: &mut LossReport,
) {
    for (bi, block) in section.blocks.iter().enumerate() {
        let loc = format!("section[{si}]/block[{bi}]");
        write_block(out, block, &loc, opts, locs, loss);
    }
}

/// Resolve a block's location box (only when location output is enabled).
fn resolve_loc<'a>(
    opts: &ConvertOptions,
    locs: &'a LocationMap,
    prov: Option<Prov>,
) -> Option<&'a Location> {
    if !opts.with_location {
        return None;
    }
    prov.and_then(|p| locs.get(&p))
}

/// Emit the four `<location value="N" resolution="512"/>` head elements for a
/// resolved box, in DocLang order (x_min, y_min, x_max, y_max). No-op for
/// `None`.
fn emit_location(out: &mut String, loc: Option<&Location>) {
    let Some(l) = loc else { return };
    for value in [l.x_min, l.y_min, l.x_max, l.y_max] {
        out.push_str("<location value=\"");
        out.push_str(&value.to_string());
        out.push_str("\" resolution=\"");
        out.push_str(&LOCATION_RESOLUTION.to_string());
        out.push_str("\"/>");
    }
}

fn write_block(
    out: &mut String,
    block: &Block,
    loc: &str,
    opts: &ConvertOptions,
    locs: &LocationMap,
    loss: &mut LossReport,
) {
    match block {
        Block::Paragraph {
            content,
            lost,
            prov,
        } => {
            out.push_str("<text>");
            // Element-head order: (lost→custom is LAST, so) location precedes it.
            emit_location(out, resolve_loc(opts, locs, *prov));
            emit_or_record_lost(out, lost.as_ref(), loc, opts, loss);
            write_inlines(out, content);
            out.push_str("</text>");
        }

        Block::Heading {
            level,
            content,
            lost,
            prov,
        } => {
            let lvl = clamp_level(*level);
            out.push_str("<heading level=\"");
            out.push_str(&lvl.to_string());
            out.push_str("\">");
            emit_location(out, resolve_loc(opts, locs, *prov));
            emit_or_record_lost(out, lost.as_ref(), loc, opts, loss);
            write_inlines(out, content);
            out.push_str("</heading>");
        }

        Block::List {
            ordered,
            items,
            lost,
            prov,
        } => {
            out.push_str("<list class=\"");
            out.push_str(if *ordered { "ordered" } else { "unordered" });
            out.push_str("\">");
            emit_location(out, resolve_loc(opts, locs, *prov));
            emit_or_record_lost(out, lost.as_ref(), loc, opts, loss);
            for (ii, item) in items.iter().enumerate() {
                let item_loc = format!("{loc}/ldiv[{ii}]");
                write_list_item(out, item, &item_loc, opts, locs, loss);
            }
            out.push_str("</list>");
        }

        Block::Table(table) => {
            otsl::write_table(out, table, loc, opts, locs, loss);
        }

        Block::Picture {
            data,
            extension,
            lost,
            prov,
            ..
        } => {
            let resource = resolve_resource(&opts.resource_policy, loc, extension, data);
            out.push_str("<picture>");
            // Element head: location precedes `<custom>` (lost) which precedes `<src>`.
            emit_location(out, resolve_loc(opts, locs, *prov));
            emit_or_record_lost(out, lost.as_ref(), loc, opts, loss);
            out.push_str("<src uri=\"");
            out.push_str(&escape_attr(&resource.uri));
            out.push_str("\"/></picture>");
        }

        Block::Formula(formula) => {
            write_formula(out, formula, loc, opts, locs, loss);
        }

        Block::PageBreak => {
            out.push_str("<page_break/>");
        }

        Block::Footnote {
            number: _,
            content,
            prov,
        } => {
            // DocLang footnotes are numbered implicitly by document order; the
            // schema (`component_with_semantic_seq`) does not permit a `number`
            // attribute, so the IR's number is intentionally dropped here.
            out.push_str("<footnote>");
            emit_location(out, resolve_loc(opts, locs, *prov));
            write_blocks(out, content, loc, opts, locs, loss);
            out.push_str("</footnote>");
        }

        Block::PageHeader {
            content,
            apply,
            prov,
        } => {
            record_apply_loss(loss, apply, loc, "page_header");
            out.push_str("<page_header>");
            emit_location(out, resolve_loc(opts, locs, *prov));
            write_blocks(out, content, loc, opts, locs, loss);
            out.push_str("</page_header>");
        }

        Block::PageFooter {
            content,
            apply,
            prov,
        } => {
            record_apply_loss(loss, apply, loc, "page_footer");
            out.push_str("<page_footer>");
            emit_location(out, resolve_loc(opts, locs, *prov));
            write_blocks(out, content, loc, opts, locs, loss);
            out.push_str("</page_footer>");
        }

        // Threads mark column / text-box flow boundaries. In DocLang v0.6
        // `<thread>` is NOT a standalone block — it is a member of an element
        // *head* and carries a required `thread_id` of type positiveInteger
        // (no `id`, no `continuation` attribute exist in the schema). We
        // therefore anchor each boundary to an empty `<group>` whose head holds
        // the `<thread>`: `<group><thread thread_id="N"/></group>`. Both the
        // start and every continuation of the same flow share the same numeric
        // id, so a validator/consumer can stitch the segments back together.
        Block::ThreadStart { thread_id } => {
            out.push_str("<group><thread thread_id=\"");
            out.push_str(&thread_id_to_int(thread_id).to_string());
            out.push_str("\"/></group>");
        }

        Block::ThreadContinuation { thread_id } => {
            out.push_str("<group><thread thread_id=\"");
            out.push_str(&thread_id_to_int(thread_id).to_string());
            out.push_str("\"/></group>");
            // The start/continuation distinction has no DocLang v0.6
            // representation (a thread head is identical for every segment);
            // record it so nothing is silently dropped.
            loss.push(LossEntry {
                kind: LossKind::SectionSettings,
                location: loc.to_string(),
                detail: format!(
                    "thread '{thread_id}' continuation marker flattened; DocLang \
                     v0.6 <thread> has no continuation attribute"
                ),
            });
        }

        // In Preserve mode: emit the opaque custom block as-is.
        // In Lean mode: drop the block entirely and record a loss entry so
        // callers know something was elided.
        Block::Custom { namespace, payload } => {
            match opts.mode {
                Mode::Preserve => {
                    // A bare `<custom>` is only valid inside an element head, so
                    // wrap it in a `<group>` (whose head may carry `<custom>`).
                    // The opaque `key=value;` payload becomes `<hwp_prop>`
                    // children — `<custom>` is element-only and attribute-free.
                    out.push_str("<group>");
                    write_custom_payload(out, namespace, payload);
                    out.push_str("</group>");
                }
                Mode::Lean => {
                    // Drop; record as FloatingObject loss (the main source of
                    // adapter-generated Custom blocks is shapes/text-boxes).
                    loss.push(LossEntry {
                        kind: LossKind::FloatingObject,
                        location: loc.to_string(),
                        detail: format!("custom block dropped (ns={})", namespace),
                    });
                }
            }
        }
    }
}

fn write_blocks(
    out: &mut String,
    blocks: &[Block],
    loc: &str,
    opts: &ConvertOptions,
    locs: &LocationMap,
    loss: &mut LossReport,
) {
    for (bi, block) in blocks.iter().enumerate() {
        let child_loc = format!("{loc}/block[{bi}]");
        write_block(out, block, &child_loc, opts, locs, loss);
    }
}

fn write_list_item(
    out: &mut String,
    item: &ListItem,
    loc: &str,
    opts: &ConvertOptions,
    locs: &LocationMap,
    loss: &mut LossReport,
) {
    // Per the DocLang schema, `<ldiv>` is an item-delimiter element whose only
    // permitted child is an optional `<marker>`; the item's actual content
    // (text / nested semantic elements) follows the `<ldiv/>` as siblings
    // within the enclosing `<list>` (schema group `list_item`).
    out.push_str("<ldiv/>");
    write_blocks(out, &item.content, loc, opts, locs, loss);
}

/// In Preserve mode: append `<custom ns="hwp:style">…</custom>` adjacent to
/// the owning element (LAST in element-head order per DocLang spec).
/// In Lean mode: record each non-empty field as a [`LossEntry`].
///
/// Does nothing when `lost` is `None` or all fields are empty.
fn emit_or_record_lost(
    out: &mut String,
    lost: Option<&crate::doclang::ir::style::LostProperties>,
    loc: &str,
    opts: &ConvertOptions,
    loss: &mut LossReport,
) {
    let Some(lost) = lost else { return };
    match opts.mode {
        Mode::Preserve => {
            write_lost_as_custom(out, lost);
        }
        Mode::Lean => {
            record_loss(loss, lost, loc);
        }
    }
}

fn write_formula(
    out: &mut String,
    formula: &Formula,
    loc: &str,
    opts: &ConvertOptions,
    locs: &LocationMap,
    loss: &mut LossReport,
) {
    out.push_str("<formula>");
    emit_location(out, resolve_loc(opts, locs, formula.prov));
    match &formula.latex {
        Some(latex) => out.push_str(&escape_text(latex)),
        None => {
            // No LaTeX available yet (eqedit conversion is a later task). Emit
            // the raw EqEdit script verbatim (escaped) so no content is lost,
            // and record a FormulaFallback loss entry.
            out.push_str(&escape_text(&formula.raw_eqedit));
            loss.push(LossEntry {
                kind: LossKind::FormulaFallback,
                location: loc.to_string(),
                detail: format!(
                    "no LaTeX conversion; emitted raw EqEdit script: {}",
                    formula.raw_eqedit
                ),
            });
        }
    }
    out.push_str("</formula>");
}

/// Clamp a heading level to the DocLang-supported 1..=6 range.
fn clamp_level(level: u8) -> u8 {
    level.clamp(1, 6)
}

/// Map an adapter thread id (e.g. `"col-0"`, `"textbox-3"`) to a DocLang
/// `thread_id`, which the schema types as `xs:positiveInteger` (≥ 1).
///
/// The adapter assigns ids as `"<prefix>-<n>"` with a monotonically increasing
/// `n` starting at 0. We extract the trailing integer and shift it to a
/// 1-based value so it satisfies `positiveInteger`. Ids without a numeric
/// suffix fall back to `1`. The mapping is deterministic and order-preserving,
/// so start and continuation segments of the same flow get the same id.
fn thread_id_to_int(thread_id: &str) -> u64 {
    thread_id
        .rsplit('-')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .map(|n| n + 1)
        .unwrap_or(1)
}

/// Record the header/footer page-applicability as a loss.
///
/// DocLang v0.6's `page_header`/`page_footer` (`component_with_semantic_seq`)
/// carry **no `apply` attribute** — the schema cannot express "odd/even/first
/// page only". The IR distinguishes these (HWP stores separate odd/even/first
/// headers), so emitting the distinction would produce schema-invalid XML.
/// We therefore drop it from the output and record a [`LossKind::SectionSettings`]
/// entry. `All` is the representable default and produces no loss.
fn record_apply_loss(loss: &mut LossReport, apply: &HeaderFooterApply, loc: &str, element: &str) {
    let scope = match apply {
        HeaderFooterApply::All => return,
        HeaderFooterApply::Even => "even pages",
        HeaderFooterApply::Odd => "odd pages",
        HeaderFooterApply::First => "first page",
    };
    loss.push(LossEntry {
        kind: LossKind::SectionSettings,
        location: loc.to_string(),
        detail: format!(
            "{element} applies to {scope} only; DocLang v0.6 has no page-scope \
             attribute, so it is rendered for all pages"
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doclang::ir::block::Block;
    use crate::doclang::ir::inline::Inline;
    use crate::doclang::ir::style::StyleFlags;
    use crate::doclang::ir::{Section, SirDocument};
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine as _;

    fn convert(blocks: Vec<Block>) -> (String, LossReport) {
        let doc = SirDocument {
            sections: vec![Section { blocks }],
            doclang_version: "0.6",
        };
        let opts = ConvertOptions::default(); // Mode::Lean
        let mut loss = LossReport::new();
        let xml = write_doclang(&doc, &opts, &mut loss).expect("write_doclang");
        (xml, loss)
    }

    fn convert_preserve(blocks: Vec<Block>) -> (String, LossReport) {
        let doc = SirDocument {
            sections: vec![Section { blocks }],
            doclang_version: "0.6",
        };
        let opts = ConvertOptions {
            mode: Mode::Preserve,
            ..ConvertOptions::default()
        };
        let mut loss = LossReport::new();
        let xml = write_doclang(&doc, &opts, &mut loss).expect("write_doclang");
        (xml, loss)
    }

    #[test]
    fn empty_document() {
        let doc = SirDocument {
            sections: vec![],
            doclang_version: "0.6",
        };
        let mut loss = LossReport::new();
        let xml = write_doclang(&doc, &ConvertOptions::default(), &mut loss).unwrap();
        assert_eq!(xml, "<doclang version=\"0.6\"></doclang>");
        assert!(loss.is_empty());
    }

    #[test]
    fn paragraph_with_mixed_styled_runs() {
        let bold = StyleFlags {
            bold: true,
            ..Default::default()
        };
        let (xml, _) = convert(vec![Block::Paragraph {
            content: vec![
                Inline::Text("plain ".into()),
                Inline::Styled(bold, vec![Inline::Text("bold".into())]),
                Inline::Text(" end".into()),
            ],
            lost: None,
            prov: None,
        }]);
        assert!(xml.contains("<text>plain <bold>bold</bold> end</text>"));
    }

    #[test]
    fn nested_styles_emit_in_canonical_order() {
        let flags = StyleFlags {
            bold: true,
            italic: true,
            underline: true,
            ..Default::default()
        };
        let (xml, _) = convert(vec![Block::Paragraph {
            content: vec![Inline::Styled(flags, vec![Inline::Text("x".into())])],
            lost: None,
            prov: None,
        }]);
        assert!(xml.contains("<text><bold><italic><underline>x</underline></italic></bold></text>"));
    }

    #[test]
    fn heading_level_is_clamped() {
        let (xml, _) = convert(vec![
            Block::Heading {
                level: 0,
                content: vec![Inline::Text("a".into())],
                lost: None,
                prov: None,
            },
            Block::Heading {
                level: 9,
                content: vec![Inline::Text("b".into())],
                lost: None,
                prov: None,
            },
            Block::Heading {
                level: 3,
                content: vec![Inline::Text("c".into())],
                lost: None,
                prov: None,
            },
        ]);
        assert!(xml.contains("<heading level=\"1\">a</heading>"));
        assert!(xml.contains("<heading level=\"6\">b</heading>"));
        assert!(xml.contains("<heading level=\"3\">c</heading>"));
    }

    #[test]
    fn ordered_and_unordered_lists() {
        let item = |t: &str| ListItem {
            content: vec![Block::Paragraph {
                content: vec![Inline::Text(t.into())],
                lost: None,
                prov: None,
            }],
        };
        let (xml, _) = convert(vec![
            Block::List {
                ordered: true,
                items: vec![item("one")],
                lost: None,
                prov: None,
            },
            Block::List {
                ordered: false,
                items: vec![item("two")],
                lost: None,
                prov: None,
            },
        ]);
        assert!(xml.contains("<list class=\"ordered\"><ldiv/><text>one</text></list>"));
        assert!(xml.contains("<list class=\"unordered\"><ldiv/><text>two</text></list>"));
    }

    #[test]
    fn picture_base64_round_trip() {
        let bytes = vec![0x89u8, 0x50, 0x4e, 0x47, 0x00, 0xff];
        let (xml, _) = convert(vec![Block::Picture {
            data: bytes.clone(),
            extension: "PNG".into(),
            geometry: None,
            lost: None,
            prov: None,
        }]);
        let expected = BASE64.encode(&bytes);
        assert!(xml.contains("data:image/png;base64,"));
        assert!(xml.contains(&expected));
        // Round-trip the embedded payload.
        let start = xml.find("base64,").unwrap() + "base64,".len();
        let end = xml[start..].find('"').unwrap() + start;
        let decoded = BASE64.decode(&xml[start..end]).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn picture_unknown_extension_is_octet_stream() {
        let (xml, _) = convert(vec![Block::Picture {
            data: vec![1, 2, 3],
            extension: "xyz".into(),
            geometry: None,
            lost: None,
            prov: None,
        }]);
        assert!(xml.contains("data:application/octet-stream;base64,"));
    }

    #[test]
    fn formula_with_latex_and_raw_fallback() {
        // With LaTeX present: no loss.
        let (xml, loss) = convert(vec![Block::Formula(Formula {
            raw_eqedit: "1 over 2".into(),
            latex: Some("\\frac{1}{2}".into()),
            prov: None,
        })]);
        assert!(xml.contains("<formula>\\frac{1}{2}</formula>"));
        assert!(loss.is_empty());

        // Without LaTeX: raw script emitted + FormulaFallback recorded.
        let (xml2, loss2) = convert(vec![Block::Formula(Formula {
            raw_eqedit: "a < b".into(),
            latex: None,
            prov: None,
        })]);
        assert!(xml2.contains("<formula>a &lt; b</formula>"));
        assert_eq!(loss2.len(), 1);
        assert_eq!(loss2.iter().next().unwrap().kind, LossKind::FormulaFallback);
    }

    #[test]
    fn page_elements() {
        let (xml, loss) = convert(vec![
            Block::PageBreak,
            Block::PageHeader {
                content: vec![Block::Paragraph {
                    content: vec![Inline::Text("hdr".into())],
                    lost: None,
                    prov: None,
                }],
                apply: HeaderFooterApply::Odd,
                prov: None,
            },
            Block::PageFooter {
                content: vec![Block::Paragraph {
                    content: vec![Inline::Text("ftr".into())],
                    lost: None,
                    prov: None,
                }],
                apply: HeaderFooterApply::All,
                prov: None,
            },
        ]);
        assert!(xml.contains("<page_break/>"));
        // DocLang v0.6 has no `apply` attribute on page_header/page_footer:
        // an odd-only header is emitted plainly and the page-scope is a loss.
        assert!(xml.contains("<page_header><text>hdr</text></page_header>"));
        assert!(!xml.contains("apply="), "no apply attribute may be emitted");
        assert!(
            loss.iter().any(|e| e.kind == LossKind::SectionSettings
                && e.detail.contains("page_header")
                && e.detail.contains("odd")),
            "odd-page header scope must be recorded as a SectionSettings loss"
        );
        // `All` (the default) is representable, so it produces no loss.
        assert!(xml.contains("<page_footer><text>ftr</text></page_footer>"));
    }

    #[test]
    fn footnote_block() {
        let (xml, _) = convert(vec![Block::Footnote {
            number: 2,
            content: vec![Block::Paragraph {
                content: vec![Inline::Text("note".into())],
                lost: None,
                prov: None,
            }],
            prov: None,
        }]);
        // The `number` attribute is intentionally not emitted (schema disallows it).
        assert!(xml.contains("<footnote><text>note</text></footnote>"));
    }

    #[test]
    fn xml_escaping_in_text_and_attrs() {
        // Special chars in text.
        let (xml, _) = convert(vec![Block::Paragraph {
            content: vec![Inline::Text("a & b < c > d".into())],
            lost: None,
            prov: None,
        }]);
        assert!(xml.contains("<text>a &amp; b &lt; c &gt; d</text>"));

        // A thread id is normalised to a positiveInteger, so no raw adapter
        // string (and thus nothing requiring attribute escaping) reaches the
        // output. A non-numeric id falls back to the minimum valid id.
        let (xml2, _) = convert(vec![Block::ThreadStart {
            thread_id: "id\"&<>".into(),
        }]);
        assert!(xml2.contains("<group><thread thread_id=\"1\"/></group>"));
        assert!(!xml2.contains("id&quot;"), "raw thread id must not leak");
    }

    #[test]
    fn custom_block_preserve_mode_emits_schema_valid_group() {
        // In Preserve mode, Block::Custom is wrapped in a `<group>` whose head
        // carries a `<custom>` element (element-only, no attributes). The
        // namespace becomes a leading `<hwp_prop name="ns" …/>` child and the
        // opaque `key=value;` payload is split into one `<hwp_prop>` per pair.
        let (xml, loss) = convert_preserve(vec![Block::Custom {
            namespace: "hwp:geometry".into(),
            payload: "shape=box;width=51024".into(),
        }]);
        assert!(
            xml.contains("<group><custom>"),
            "must wrap custom in a group"
        );
        assert!(
            !xml.contains("ns=\""),
            "custom must not carry an ns attribute"
        );
        assert!(xml.contains("<hwp_prop name=\"ns\" value=\"hwp:geometry\"/>"));
        assert!(xml.contains("<hwp_prop name=\"shape\" value=\"box\"/>"));
        assert!(xml.contains("<hwp_prop name=\"width\" value=\"51024\"/>"));
        assert!(xml.contains("</custom></group>"));
        assert!(
            loss.is_empty(),
            "preserve mode should not record loss for Custom blocks"
        );
    }

    #[test]
    fn custom_block_lean_mode_drops_and_records_loss() {
        // In Lean mode, Block::Custom is dropped and a loss entry is recorded.
        let (xml, loss) = convert(vec![Block::Custom {
            namespace: "hwp:floating".into(),
            payload: "some payload".into(),
        }]);
        assert!(!xml.contains("<custom"), "lean mode must not emit <custom");
        assert_eq!(loss.len(), 1);
        assert_eq!(loss.iter().next().unwrap().kind, LossKind::FloatingObject);
    }

    #[test]
    fn thread_continuation_marker() {
        let (xml, loss) = convert(vec![
            Block::ThreadStart {
                thread_id: "col-0".into(),
            },
            Block::ThreadContinuation {
                thread_id: "col-0".into(),
            },
        ]);
        // DocLang v0.6: <thread> lives in an element head with a positiveInteger
        // `thread_id`; we anchor it to an empty <group>. "col-0" → 1, and both
        // segments share the same id.
        assert_eq!(
            xml,
            "<doclang version=\"0.6\">\
             <group><thread thread_id=\"1\"/></group>\
             <group><thread thread_id=\"1\"/></group>\
             </doclang>"
        );
        // The continuation distinction is unrepresentable → recorded as a loss.
        assert!(
            loss.iter()
                .any(|e| e.kind == LossKind::SectionSettings && e.detail.contains("continuation")),
            "continuation flattening must be recorded as a loss"
        );
    }

    #[test]
    fn thread_id_suffix_maps_to_positive_integer() {
        assert_eq!(thread_id_to_int("col-0"), 1);
        assert_eq!(thread_id_to_int("col-7"), 8);
        assert_eq!(thread_id_to_int("textbox-3"), 4);
        // No numeric suffix → fall back to the minimum positiveInteger.
        assert_eq!(thread_id_to_int("weird"), 1);
    }

    #[test]
    fn table_serialises_otsl_through_block_writer() {
        use crate::doclang::ir::table::{Table, TableCell};
        let (xml, loss) = convert(vec![Block::Table(Table {
            rows: 1,
            cols: 1,
            cells: vec![TableCell {
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                is_header: false,
                content: vec![Block::Paragraph {
                    content: vec![Inline::Text("hi".into())],
                    lost: None,
                    prov: None,
                }],
            }],
            caption: None,
            prov: None,
        })]);
        assert!(xml.contains("<table><fcel/><text>hi</text><nl/></table>"));
        assert!(loss.is_empty());
    }
}
