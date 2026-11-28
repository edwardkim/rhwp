namespace Rhwp.WinUI;

internal static class TableObjectSmoke
{
    public static async Task RunAsync()
    {
        var dispatcher = new NativeWorkerDispatcher();
        CancellationToken token = CancellationToken.None;
        try
        {
            Assert(await dispatcher.SupportsTableObjectsAsync(token), "table object capability");
            PageInfo current = await dispatcher.NewAsync(token);
            TextPosition body = new(0, 0, 0, []);
            await dispatcher.BeginEditAsync(current.Target, new TextSelection(body, body), token);
            EditResult created = await dispatcher.CreateTableAsync(current.Target, 2, 2, token);
            current = await dispatcher.GetPageAsync(current with { Target = created.Target }, 0, token);
            TextPosition cell = created.Outcome.Selection.Focus;
            NativeTableObject baseline = await dispatcher.GetTableObjectAsync(current, cell, token);
            Assert(baseline is { RowCount: 2, ColumnCount: 2, Editable: true, CanResize: true,
                Bounds: { Width: > 0, Height: > 0 } }, "top-level table shape, properties and bounds");

            try
            {
                await dispatcher.GetTableObjectAsync(current with { FontGeneration = current.FontGeneration + 1 },
                    cell, token);
                throw new InvalidOperationException("stale table geometry was accepted");
            }
            catch (InvalidDataException)
            {
            }

            NativeTableObjectProperties properties = baseline.Properties;
            TableObjectPropertyChanges patch = new(properties.CellSpacing, properties.PaddingLeft,
                properties.PaddingRight, properties.PaddingTop, properties.PaddingBottom, 330,
                properties.OuterRight, properties.OuterTop, properties.OuterBottom, 2, properties.RepeatHeader);
            EditResult styled = await dispatcher.SetTableObjectPropertiesAsync(current.Target, baseline.Address, patch, token);
            current = await dispatcher.GetPageAsync(current with { Target = styled.Target }, 0, token);
            NativeTableObject styledTable = await dispatcher.GetTableObjectAsync(current, cell, token);
            Assert(styledTable.Properties is { OuterLeft: 330, PageBreak: 2 }, "property mutation");

            uint width = properties.TableWidth - 2_000;
            // Exceed the two rows' content-fitting height so the rendered
            // bounds also grow when the stored table height changes.
            uint height = properties.TableHeight + 3_500;
            EditResult resized = await dispatcher.ResizeTableObjectAsync(current.Target, baseline.Address, width, height, token);
            current = await dispatcher.GetPageAsync(current with { Target = resized.Target }, 0, token);
            NativeTableObject result = await dispatcher.GetTableObjectAsync(current, cell, token);
            Assert(result.Properties.TableWidth == width && result.Properties.TableHeight == height &&
                result.Bounds is { } actual && baseline.Bounds is { } original &&
                actual.Width < original.Width && actual.Height > original.Height,
                "resize changes stored dimensions and displayed bounds");

            EditResult undone = await dispatcher.UndoAsync(current.Target, token);
            current = await dispatcher.GetPageAsync(current with { Target = undone.Target }, 0, token);
            NativeTableObject restored = await dispatcher.GetTableObjectAsync(current, cell, token);
            Assert(restored.Properties == styledTable.Properties, "undo restores table properties");
            Console.WriteLine("PASS: WinUI managed table object, stale stamp, properties, geometry and undo");
        }
        finally
        {
            await dispatcher.CloseAsync();
        }
    }

    private static void Assert(bool expected, string check)
    {
        if (!expected)
            throw new InvalidOperationException(check);
    }
}
