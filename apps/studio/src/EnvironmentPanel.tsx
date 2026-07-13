import type { EnvironmentStatus } from "./backend";

type Props = {
  environment?: EnvironmentStatus;
  refreshing: boolean;
  onRefresh: () => void;
  onUsePsp: (path: string) => void;
};

export default function EnvironmentPanel({ environment, refreshing, onRefresh, onUsePsp }: Props) {
  const requiredReady = environment?.checks.filter(check => check.required).every(check => check.ready) ?? false;
  return <div className="environment-page">
    <div className="environment-head">
      <div><small>CONFIGURATION CONTRÔLÉE</small><h1>Environnement PSP</h1><p>Diagnostic en lecture seule des outils nécessaires. Aucun terminal libre n’est exposé.</p></div>
      <button className="build" disabled={refreshing} onClick={onRefresh}>{refreshing ? "Analyse…" : "↻ Relancer le diagnostic"}</button>
    </div>
    <section className={requiredReady ? "environment-summary ready" : "environment-summary blocked"}>
      <b>{requiredReady ? "Prêt à compiler" : "Configuration incomplète"}</b>
      <span>{requiredReady ? "Les composants C++ obligatoires sont disponibles." : "Un composant obligatoire doit être installé ou réparé."}</span>
    </section>
    <div className="environment-cards">{environment?.checks.map(check => <article key={check.id} className={check.ready ? "ready" : "missing"}>
      <div><span>{check.ready ? "✓" : "!"}</span><div><h2>{check.label}</h2><small>{check.required ? "Obligatoire" : "Optionnel"}</small></div></div>
      <p>{check.detail}</p>
      {check.path && <code>{check.path}</code>}
      {check.id === "psp" && check.path && <button onClick={() => onUsePsp(check.path!)}>Utiliser cette PSP</button>}
    </article>)}</div>
    <aside className="security-note"><b>🔒 Mode sécurisé</b><p>La compilation utilise une configuration générée par l’IDE. Les chemins sont confinés au projet et les liens symboliques sont refusés.</p></aside>
  </div>;
}
