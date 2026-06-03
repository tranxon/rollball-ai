import { useEffect, useRef, useState } from "react";
import mermaid from "mermaid";

/** Initialize mermaid once with light theme. */
let initialized = false;
function ensureInit() {
  if (initialized) return;
  initialized = true;
  mermaid.initialize({
    startOnLoad: false,
    theme: "base",
    themeVariables: {
      background: "#ffffff",
      primaryColor: "#f0f4ff",
      primaryBorderColor: "#93c5fd",
      primaryTextColor: "#1e293b",
      lineColor: "#94a3b8",
      secondaryColor: "#f8fafc",
      tertiaryColor: "#f1f5f9",
      clusterBkg: "#f8fafc",
      clusterBorder: "#e2e8f0",
      edgeLabelBackground: "#ffffff",
      nodeBorder: "#cbd5e1",
      nodeTextColor: "#1e293b",
      fontSize: "12px",
      fontFamily: "system-ui, -apple-system, sans-serif",
    },
    flowchart: {
      useMaxWidth: true,
      htmlLabels: true,
      curve: "basis",
      padding: 6,
      nodeSpacing: 35,
      rankSpacing: 35,
    },
    sequence: {
      useMaxWidth: true,
      showSequenceNumbers: false,
    },
  });
}

interface MermaidBlockProps {
  chart: string;
}

export function MermaidBlock({ chart }: MermaidBlockProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const idRef = useRef(0);
  const [error, setError] = useState(false);

  useEffect(() => {
    ensureInit();

    if (!containerRef.current) return;

    let cancelled = false;

    const render = async () => {
      try {
        const id = `rollball-mermaid-${idRef.current++}`;
        const { svg } = await mermaid.render(id, chart);
        if (!cancelled && containerRef.current) {
          containerRef.current.innerHTML = svg;
        }
      } catch {
        if (!cancelled) setError(true);
      }
    };

    setError(false);
    render();

    return () => {
      cancelled = true;
    };
  }, [chart]);

  if (error) {
    return (
      <div className="overflow-x-auto whitespace-pre-wrap rounded-lg border border-zinc-200 bg-zinc-100 p-3 font-mono text-xs leading-relaxed text-zinc-500 dark:border-zinc-700 dark:bg-zinc-900/60 dark:text-zinc-400">
        {chart}
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="my-2 flex justify-start overflow-x-auto rounded-lg border border-zinc-200 bg-white p-3 dark:border-zinc-700 dark:bg-zinc-900/40 [&_.label]:text-zinc-600"
    />
  );
}