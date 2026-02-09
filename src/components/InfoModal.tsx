interface InfoModalProps {
  isOpen: boolean;
  title: string;
  icon?: string;
  sections: {
    title?: string;
    content: string;
    type?: "text" | "code" | "list";
  }[];
  confirmText?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function InfoModal({
  isOpen,
  title,
  icon = "ℹ️",
  sections,
  confirmText = "确定",
  onConfirm,
  onCancel,
}: InfoModalProps) {
  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={onCancel}>
      <div className="info-modal" onClick={(e) => e.stopPropagation()}>
        <div className="info-modal-header">
          <div className="info-modal-icon">{icon}</div>
          <h3 className="info-modal-title">{title}</h3>
          <button className="info-modal-close" onClick={onCancel}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" width="20" height="20">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>

        <div className="info-modal-body">
          {sections.map((section, index) => (
            <div key={index} className="info-section">
              {section.title && <h4 className="info-section-title">{section.title}</h4>}
              {section.type === "code" ? (
                <pre className="info-code-block">
                  <code>{section.content}</code>
                </pre>
              ) : section.type === "list" ? (
                <div className="info-list" dangerouslySetInnerHTML={{ __html: section.content }} />
              ) : (
                <p className="info-text">{section.content}</p>
              )}
            </div>
          ))}
        </div>

        <div className="info-modal-footer">
          <button className="info-btn cancel" onClick={onCancel}>
            取消
          </button>
          <button className="info-btn confirm" onClick={onConfirm}>
            {confirmText}
          </button>
        </div>
      </div>
    </div>
  );
}
