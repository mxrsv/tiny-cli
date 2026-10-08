import AppKit

/// What a confirmation names: the target, the consequence and the action.
struct ConfirmationCopy: Equatable {
    let title: String
    let message: String
    let confirmLabel: String
    let isDestructive: Bool
}

extension ActionRequest {
    var copy: ConfirmationCopy {
        ConfirmationCopy(title: title, message: message, confirmLabel: confirmLabel, isDestructive: isDestructive)
    }
}

/// Native confirmation sheet. Cancel comes first, so it is the default button
/// (Return) and also takes Escape; the action has no key equivalent.
@MainActor enum ConfirmationAlert {
    static let escapeKeyCode: UInt16 = 53

    static func make(_ request: ActionRequest) -> NSAlert { make(request.copy) }

    static func make(_ copy: ConfirmationCopy) -> NSAlert {
        let alert = NSAlert()
        alert.messageText = copy.title
        alert.informativeText = copy.message
        alert.alertStyle = copy.isDestructive ? .critical : .warning
        let cancel = alert.addButton(withTitle: "Cancel")
        cancel.keyEquivalent = "\r"
        let action = alert.addButton(withTitle: copy.confirmLabel)
        action.keyEquivalent = ""
        action.hasDestructiveAction = copy.isDestructive
        return alert
    }

    /// Only the action button confirms; Cancel, Escape and a closed window cancel.
    static func confirms(_ response: NSApplication.ModalResponse) -> Bool { response == .alertSecondButtonReturn }

    @discardableResult
    static func finish(_ response: NSApplication.ModalResponse, request: ActionRequest,
                       actions: ActionState) -> Task<Void, Never>? {
        guard confirms(response) else { actions.cancel(); return nil }
        return actions.confirm(request)
    }

    static func isEscape(_ event: NSEvent) -> Bool { event.type == .keyDown && event.keyCode == escapeKeyCode }

    /// Shows the sheet for a captured request; `finish` maps the response exactly once.
    static func present(_ request: ActionRequest, on window: NSWindow, actions: ActionState) {
        present(request.copy, on: window) { response in finish(response, request: request, actions: actions) }
    }

    static func present(_ copy: ConfirmationCopy, on window: NSWindow,
                        completion: @escaping @MainActor (NSApplication.ModalResponse) -> Void) {
        let alert = make(copy)
        let sheet = alert.window
        let monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            guard isEscape(event), event.window === sheet else { return event }
            window.endSheet(sheet, returnCode: .alertFirstButtonReturn)
            return nil
        }
        alert.beginSheetModal(for: window) { response in
            if let monitor { NSEvent.removeMonitor(monitor) }
            completion(response)
        }
    }

    /// The window a confirmation can attach to, or nil when none can show a sheet now.
    static func hostWindow() -> NSWindow? {
        guard let window = NSApp.keyWindow ?? NSApp.mainWindow, window.attachedSheet == nil else { return nil }
        return window
    }
}
