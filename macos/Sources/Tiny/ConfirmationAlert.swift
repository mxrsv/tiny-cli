import AppKit

/// Native confirmation sheet for one captured request. Cancel comes first, so it is
/// the default button (Return) and also takes Escape; the action has no key equivalent.
@MainActor enum ConfirmationAlert {
    static let escapeKeyCode: UInt16 = 53

    static func make(_ request: ActionRequest) -> NSAlert {
        let alert = NSAlert()
        alert.messageText = request.title
        alert.informativeText = request.message
        alert.alertStyle = request.isDestructive ? .critical : .warning
        let cancel = alert.addButton(withTitle: "Cancel")
        cancel.keyEquivalent = "\r"
        let action = alert.addButton(withTitle: request.confirmLabel)
        action.keyEquivalent = ""
        action.hasDestructiveAction = request.isDestructive
        return alert
    }

    /// Only the action button confirms; Cancel, Escape and a closed window cancel.
    @discardableResult
    static func finish(_ response: NSApplication.ModalResponse, request: ActionRequest,
                       actions: ActionState) -> Task<Void, Never>? {
        guard response == .alertSecondButtonReturn else { actions.cancel(); return nil }
        return actions.confirm(request)
    }

    static func isEscape(_ event: NSEvent) -> Bool { event.type == .keyDown && event.keyCode == escapeKeyCode }

    /// Shows the sheet on `window`; the request is captured here, when the alert opens.
    static func present(_ request: ActionRequest, on window: NSWindow, actions: ActionState) {
        let alert = make(request)
        let sheet = alert.window
        let monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            guard isEscape(event), event.window === sheet else { return event }
            window.endSheet(sheet, returnCode: .alertFirstButtonReturn)
            return nil
        }
        alert.beginSheetModal(for: window) { response in
            if let monitor { NSEvent.removeMonitor(monitor) }
            finish(response, request: request, actions: actions)
        }
    }
}
