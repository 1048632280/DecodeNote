import { useEffect, useRef, useState } from "react";
import { EditorView, keymap } from "@codemirror/view";
import { EditorState, type Extension } from "@codemirror/state";
import { history, redo, undo } from "@codemirror/commands";
import { defaultKeymap } from "@codemirror/commands";

interface CodeMirrorEditorProps {
  onContentChange: () => void;
  onCursorChange: (line: number, col: number) => void;
  onSave: () => void;
  editorRef: React.MutableRefObject<EditorView | null>;
}

export default function CodeMirrorEditor({
  onContentChange,
  onCursorChange,
  onSave,
  editorRef,
}: CodeMirrorEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [initialized, setInitialized] = useState(false);
  const onSaveRef = useRef(onSave);
  onSaveRef.current = onSave;

  const extensions: Extension[] = [
    history(),
    keymap.of([
      ...defaultKeymap,
      { key: "Mod-z", run: undo },
      { key: "Mod-y", run: redo },
      { key: "Mod-Shift-z", run: redo },
      {
        key: "Mod-s",
        run: () => {
          onSaveRef.current();
          return true;
        },
        preventDefault: true,
      },
    ]),
    EditorView.lineWrapping,
    EditorState.transactionFilter.of((tr) => {
      if (tr.docChanged) {
        setTimeout(() => onContentChange(), 0);
      }
      return tr;
    }),
    EditorView.updateListener.of((update) => {
      if (update.selectionSet) {
        const pos = update.state.selection.main.head;
        const line = update.state.doc.lineAt(pos);
        setTimeout(() => onCursorChange(line.number, pos - line.from + 1), 0);
      }
    }),
  ];

  useEffect(() => {
    if (!containerRef.current || initialized) return;
    const view = new EditorView({
      state: EditorState.create({
        doc: "",
        extensions,
      }),
      parent: containerRef.current,
    });
    editorRef.current = view;
    setInitialized(true);
    return () => {
      view.destroy();
      editorRef.current = null;
    };
  }, []);

  return (
    <div
      ref={containerRef}
      className="editor-container"
      style={{ flex: 1, overflow: "auto", height: "100%" }}
    />
  );
}
