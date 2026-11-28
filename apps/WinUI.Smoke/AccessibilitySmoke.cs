using System.Text.Json;
using Rhwp.WinUI;

internal static class AccessibilitySmoke
{
    internal static void CheckContract()
    {
        PageInfo page = new(new DocumentTarget(1, 2), 3, 520, 521, 640, 800);
        using var reply = JsonDocument.Parse("""
            {"kind":"accessibility","target":{"documentId":1,"revision":2},"snapshot":{
              "revision":2,"fontGeneration":3,"scope":"bodyAndTableCellText","page":520,
              "complete":true,"truncated":false,"nodes":[{
                "key":"later","role":"tableCellParagraph",
                "start":{"section":0,"paragraph":520,"scalarOffset":0,
                  "cellPath":[{"control":0,"cell":0,"paragraph":0}]},
                "text":"후반 쪽 셀","textComplete":true,
                "cellLayout":{"row":0,"column":0,"rowSpan":10,"columnSpan":1,"rowCount":2,"columnCount":1},
                "pageBounds":[{"x":30,"y":20,"width":20,"height":10},
                  {"x":60,"y":40,"width":10,"height":10}]
              }]
            }}
            """);
        var snapshot = NativeWorkerDispatcher.ReadAutomationDocument(reply.RootElement, page, page.Page);
        var root = DocumentAutomationNode.Build(snapshot);
        var cell = root.Children.Single().Children.Single();
        if (cell.Children.Single().Name != "후반 쪽 셀" || cell.Layout is not null)
            throw new InvalidOperationException("Accessibility text must survive invalid optional grid metadata.");

        if (root.VisibleBounds(new CaretBounds(38, 25, 20, 10)) != new CaretBounds(38, 25, 12, 5) ||
            root.VisibleBounds(new CaretBounds(50, 30, 10, 10)) is not null)
            throw new InvalidOperationException("Accessibility bounds must clip visible text fragments.");
    }
}
