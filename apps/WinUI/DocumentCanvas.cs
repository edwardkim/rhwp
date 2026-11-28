using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Controls;
using Windows.Foundation;

namespace Rhwp.WinUI;

public sealed partial class DocumentCanvas : Canvas
{
    internal DocumentAutomationSnapshot? AutomationDocument { get; private set; }

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
                .Select(node => (AutomationPeer)new DocumentContentAutomationPeer(node)).ToArray();
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

internal sealed class DocumentContentAutomationPeer(DocumentAutomationNode node) : AutomationPeer
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

    // Nodes represent document structure. The worker does not provide reliable
    // page positions for these nodes, including paragraphs split between pages.
    protected override bool IsOffscreenCore() => true;
    protected override Rect GetBoundingRectangleCore() => default;

    protected override IList<AutomationPeer> GetChildrenCore() => children ??= node.Children
        .Select(child => (AutomationPeer)new DocumentContentAutomationPeer(child)).ToList();
}
