import type { ReactNode } from "react";

interface PanelProps {
  title: string;
  icon: ReactNode;
  children: ReactNode;
}

export function Panel({ title, icon, children }: PanelProps) {
  return (
    <section className="panel">
      <h2 className="panel-title"><span>{icon}</span>{title}</h2>
      <div className="panel-body">{children}</div>
    </section>
  );
}
