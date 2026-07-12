import { useMemo, useState } from "react";
import { helpArticles, type HelpArticle, type HelpLanguage } from "./helpCatalog";

type Props = { onNewProject: (language: HelpLanguage) => void };

export default function Help({ onNewProject }: Props) {
  const [language, setLanguage] = useState<"all" | HelpLanguage>("all");
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState(helpArticles[0].id);
  const [copied, setCopied] = useState("");
  const articles = useMemo(() => {
    const search = query.trim().toLocaleLowerCase("fr");
    return helpArticles.filter(article => {
      const languageMatches = language === "all" || article.language === language;
      const haystack = [article.title, article.category, article.summary, ...article.apis].join(" ").toLocaleLowerCase("fr");
      return languageMatches && (!search || haystack.includes(search));
    });
  }, [language, query]);
  const selected = articles.find(article => article.id === selectedId) ?? articles[0];
  const copy = async (article: HelpArticle) => {
    try {
      await navigator.clipboard.writeText(article.code);
      setCopied(article.id);
      window.setTimeout(() => setCopied(""), 1600);
    } catch {
      setCopied("");
    }
  };

  return <div className="help-page">
    <div className="help-head">
      <div><small>CENTRE D’AIDE</small><h1>Développer sur PSP</h1><p>Exemples vérifiés à partir de PSPDEV et de LuaPlayer Plus r163.</p></div>
      <button className="build" onClick={() => onNewProject(language === "lua" ? "lua" : "cpp")}>＋ Projet d’exemple</button>
    </div>
    <div className="help-tools">
      <input type="search" value={query} onChange={event => setQuery(event.target.value)} placeholder="Rechercher une API, un module ou une fonction…"/>
      <div className="help-filters">
        <button className={language === "all" ? "active" : ""} onClick={() => setLanguage("all")}>Tout</button>
        <button className={language === "cpp" ? "active" : ""} onClick={() => setLanguage("cpp")}>C++ / PSPSDK</button>
        <button className={language === "lua" ? "active" : ""} onClick={() => setLanguage("lua")}>LuaPlayer</button>
      </div>
    </div>
    <div className="help-layout">
      <nav className="help-index">
        <b>{articles.length} exemples</b>
        {articles.map(article => <button className={article.id === selected?.id ? "active" : ""} onClick={() => setSelectedId(article.id)} key={article.id}><span>{article.language === "cpp" ? "C++" : "LUA"}</span><div><strong>{article.title}</strong><small>{article.category}</small></div></button>)}
        {!articles.length && <p>Aucun résultat pour cette recherche.</p>}
      </nav>
      {selected ? <article className="help-article">
        <div className="help-article-title"><div><span>{selected.language === "cpp" ? "C++ / PSPSDK" : "LuaPlayer Plus"}</span><h2>{selected.title}</h2></div><button onClick={() => copy(selected)}>{copied === selected.id ? "✓ Copié" : "Copier l’exemple"}</button></div>
        <p className="help-summary">{selected.summary}</p>
        <h3>API utilisées</h3>
        <div className="api-list">{selected.apis.map(api => <code key={api}>{api}</code>)}</div>
        <h3>Exemple</h3>
        <pre><code>{selected.code}</code></pre>
        {selected.note && <aside>💡 {selected.note}</aside>}
        <footer className="help-source">Source locale : {selected.source}</footer>
      </article> : <div className="help-empty">Modifie les filtres pour retrouver un exemple.</div>}
    </div>
  </div>;
}
