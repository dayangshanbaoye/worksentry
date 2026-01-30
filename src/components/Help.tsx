function Help() {
    return (
        <div className="help-panel">
            <h2 className="help-title">🔍 Search Guide</h2>

            <section className="help-section">
                <h3>Basic Search</h3>
                <p className="help-description">
                    Just type to search! Characters are matched in sequence, like a launcher.
                </p>
                <div className="help-examples">
                    <div className="help-example">
                        <code>7r</code>
                        <span>→ matches "7 Rules of Power.pdf"</span>
                    </div>
                    <div className="help-example">
                        <code>vibe</code>
                        <span>→ matches "vibe_coding.txt", "Good Vibes.mp3"</span>
                    </div>
                </div>
            </section>

            <section className="help-section">
                <h3>🏷️ Filter by Extension</h3>
                <p className="help-description">
                    Use <code>ext:</code> to filter by specific file extensions.
                </p>
                <div className="help-examples">
                    <div className="help-example">
                        <code>report ext:pdf</code>
                        <span>→ only PDF files</span>
                    </div>
                    <div className="help-example">
                        <code>notes ext:md,txt</code>
                        <span>→ markdown or text files</span>
                    </div>
                    <div className="help-example">
                        <code>.pdf</code>
                        <span>→ shorthand for ext:pdf (shows all PDFs)</span>
                    </div>
                </div>
            </section>

            <section className="help-section">
                <h3>📁 Filter by Type Category</h3>
                <p className="help-description">
                    Use <code>type:</code> to filter by category (groups of extensions).
                </p>
                <div className="help-types">
                    <div className="help-type-row">
                        <code>type:doc</code>
                        <span>pdf, docx, txt, md, epub, mobi...</span>
                    </div>
                    <div className="help-type-row">
                        <code>type:app</code>
                        <span>exe, lnk, bat, cmd, msi...</span>
                    </div>
                    <div className="help-type-row">
                        <code>type:image</code>
                        <span>jpg, png, gif, svg, webp...</span>
                    </div>
                    <div className="help-type-row">
                        <code>type:video</code>
                        <span>mp4, mkv, avi, mov, wmv...</span>
                    </div>
                    <div className="help-type-row">
                        <code>type:audio</code>
                        <span>mp3, wav, flac, ogg...</span>
                    </div>
                    <div className="help-type-row">
                        <code>type:code</code>
                        <span>rs, py, js, ts, java, cpp...</span>
                    </div>
                    <div className="help-type-row">
                        <code>type:archive</code>
                        <span>zip, rar, 7z, tar, gz...</span>
                    </div>
                    <div className="help-type-row">
                        <code>type:ppt</code>
                        <span>ppt, pptx, odp</span>
                    </div>
                    <div className="help-type-row">
                        <code>type:excel</code>
                        <span>xls, xlsx, csv, ods</span>
                    </div>
                </div>
            </section>

            <section className="help-section">
                <h3>🌐 Filter by Source</h3>
                <p className="help-description">
                    Use <code>in:</code> to filter by where the result comes from.
                </p>
                <div className="help-examples">
                    <div className="help-example">
                        <code>in:files</code>
                        <span>→ only local files</span>
                    </div>
                    <div className="help-example">
                        <code>in:bookmarks</code>
                        <span>→ only browser bookmarks</span>
                    </div>
                    <div className="help-example">
                        <code>in:history</code>
                        <span>→ only browser history</span>
                    </div>
                    <div className="help-example">
                        <code>in:web</code>
                        <span>→ bookmarks + history</span>
                    </div>
                </div>
            </section>

            <section className="help-section">
                <h3>🚀 Pro Tips</h3>
                <div className="help-tips">
                    <div className="help-tip">
                        <span className="tip-icon">⚡</span>
                        <span>Combine filters: <code>report type:doc in:files</code></span>
                    </div>
                    <div className="help-tip">
                        <span className="tip-icon">⌨️</span>
                        <span>Use <strong>↑ ↓</strong> arrows to navigate, <strong>Enter</strong> to open</span>
                    </div>
                    <div className="help-tip">
                        <span className="tip-icon">🎯</span>
                        <span>Shorter queries rank higher (denser matches)</span>
                    </div>
                    <div className="help-tip">
                        <span className="tip-icon">📱</span>
                        <span>Apps (.exe, .lnk) are boosted in results</span>
                    </div>
                </div>
            </section>
        </div>
    );
}

export default Help;
