using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;
using Windows.Foundation;

namespace Rhwp.WinUI;

public sealed partial class DocumentCanvas : Canvas
{
    internal DocumentAutomationSnapshot? AutomationDocument { get; private set; }
    private ScrollViewer? automationViewport;
    private double zoom = 1;

    internal void UpdateAutomationViewport(ScrollViewer viewport, double pageZoom)
    {
        automationViewport = viewport;
        zoom = pageZoom;
    }

    internal Rect AutomationScreenBounds(DocumentAutomationSnapshot snapshot, DocumentAutomationNode node)
    {
        if (!ReferenceEquals(AutomationDocument, snapshot) ||
            automationViewport is not { ViewportWidth: > 0, ViewportHeight: > 0 } scroll ||
            XamlRoot is not { IsHostVisible: true } root)
            return default;

        var viewport = new CaretBounds(scroll.HorizontalOffset / zoom, scroll.VerticalOffset / zoom,
            scroll.ViewportWidth / zoom, scroll.ViewportHeight / zoom);
        if (node.VisibleBounds(viewport) is not { } bounds)
            return default;

        var local = new Rect(bounds.X * zoom, bounds.Y * zoom, bounds.Width * zoom, bounds.Height * zoom);
        Rect inRoot = TransformToVisual(root.Content).TransformBounds(local);
        var physical = root.CoordinateConverter.ConvertLocalToScreen(inRoot);
        return new Rect(physical.X, physical.Y, physical.Width, physical.Height);
    }

    internal void SetAutomationDocument(DocumentAutomationSnapshot? snapshot)
    {
        if (ReferenceEquals(AutomationDocument, snapshot))
            return;

        AutomationDocument = snapshot;
        if (AutomationPeer.ListenerExists(AutomationEvents.StructureChanged)
            && FrameworkElementAutomationPeer.FromElement(this) is DocumentCanvasAutomationPeer peer)
        {
            peer.InvalidatePeer();
            peer.RaiseStructureChangedEvent(AutomationStructureChangeType.ChildrenInvalidated, null);
        }
    }

    protected override AutomationPeer OnCreateAutomationPeer() => new DocumentCanvasAutomationPeer(this);
}

internal sealed class DocumentCanvasAutomationPeer : FrameworkElementAutomationPeer
{
    private readonly DocumentCanvas canvas;
    private DocumentAutomationSnapshot? cachedSnapshot;
    private IReadOnlyList<AutomationPeer> documentChildren = [];

    internal DocumentCanvasAutomationPeer(DocumentCanvas owner) : base(owner) => canvas = owner;

    protected override string GetClassNameCore() => nameof(DocumentCanvas);
    protected override string GetNameCore() => "문서 내용";
    protected override string GetHelpTextCore() => canvas.AutomationDocument?.Truncated == true
        ? "문서 내용의 일부만 제공됩니다."
        : "";
    protected override AutomationControlType GetAutomationControlTypeCore() => AutomationControlType.Group;
    protected override bool IsControlElementCore() => true;
    protected override bool IsContentElementCore() => true;

    protected override IList<AutomationPeer> GetChildrenCore()
    {
        DocumentAutomationSnapshot? snapshot = canvas.AutomationDocument;
        if (!ReferenceEquals(cachedSnapshot, snapshot))
        {
            cachedSnapshot = snapshot;
            documentChildren = snapshot is null ? [] : DocumentAutomationNode.Build(snapshot).Children
                .Select(node => (AutomationPeer)new DocumentContentAutomationPeer(canvas, snapshot, node)).ToArray();
        }

        // The bitmap is only a visual surface. Keep real controls such as the
        // inline TextBox in the tree so WinUI provides their automation patterns.
        var children = new List<AutomationPeer>();
        if (base.GetChildrenCore() is { } visualPeers)
        {
            foreach (AutomationPeer peer in visualPeers)
            {
                if (peer.GetAutomationControlType() != AutomationControlType.Image)
                    children.Add(peer);
            }
        }
        children.AddRange(documentChildren);
        return children;
    }
}

internal sealed class DocumentContentAutomationPeer(DocumentCanvas canvas,
    DocumentAutomationSnapshot snapshot, DocumentAutomationNode node) : AutomationPeer
{
    private IList<AutomationPeer>? children;

    protected override string GetClassNameCore() => node.Role switch
    {
        DocumentAutomationRole.Table => "DocumentTable",
        DocumentAutomationRole.Cell => "DocumentCell",
        _ => "DocumentParagraph",
    };

    protected override string GetNameCore() => node.Name;
    protected override string GetAutomationIdCore() => node.Key;
    protected override string GetHelpTextCore() => node.HelpText;
    protected override AutomationControlType GetAutomationControlTypeCore() =>
        node.Role == DocumentAutomationRole.Paragraph ? AutomationControlType.Text : AutomationControlType.Group;
    protected override bool IsControlElementCore() => true;
    protected override bool IsContentElementCore() => true;
    protected override bool IsKeyboardFocusableCore() => false;
    protected override bool HasKeyboardFocusCore() => false;
    protected override bool IsEnabledCore() => true;

    protected override bool IsOffscreenCore() => GetBoundingRectangleCore() is not { Width: > 0, Height: > 0 };
    protected override Rect GetBoundingRectangleCore() => canvas.AutomationScreenBounds(snapshot, node);

    protected override IList<AutomationPeer> GetChildrenCore() => children ??= node.Children
        .Select(child => (AutomationPeer)new DocumentContentAutomationPeer(canvas, snapshot, child)).ToList();
}
