import { useEffect, useRef, useState } from "react";
import { EditorView, keymap, Decoration, DecorationSet } from "@codemirror/view";
import { EditorState, StateField, type Extension, RangeSetBuilder } from "@codemirror/state";
import { history, redo, undo } from "@codemirror/commands";
import { defaultKeymap } from "@codemirror/commands";

interface CodeMirrorEditorProps {
  onContentChange: () => void;
  onCursorChange: (line: number, col: number) => void;
  onSave: () => void;
  editorRef: React.MutableRefObject<EditorView | null>;
}

const garbledMark = Decoration.mark({ class: "cm-garbled-highlight" });

function buildGarbledDecorations(state: EditorState): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  const text = state.doc.toString();
  for (let i = 0; i < text.length; i++) {
    if (text[i] === "\uFFFD") {
      builder.add(i, i + 1, garbledMark);
    }
  }
  return builder.finish();
}

const garbledField = StateField.define<DecorationSet>({
  create(state) {
    return buildGarbledDecorations(state);
  },
  update(decorations, tr) {
    if (tr.docChanged) {
      return buildGarbledDecorations(tr.state);
    }
    return decorations.map(tr.changes);
  },
  provide: (f) => EditorView.decorations.from(f),
});

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
    garbledField,
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
