import { describe, expect, it } from "vitest";
import { helpArticles } from "./helpCatalog";
import { templates } from "./projectCatalog";

describe("documentation et projets d’exemple", () => {
  it("publie vingt articles uniques couvrant C++ et Lua", () => {
    expect(helpArticles).toHaveLength(20);
    expect(new Set(helpArticles.map(article => article.id)).size).toBe(20);
    expect(helpArticles.filter(article => article.language === "cpp")).toHaveLength(8);
    expect(helpArticles.filter(article => article.language === "lua")).toHaveLength(12);
    for (const article of helpArticles) {
      expect(article.code.trim().length).toBeGreaterThan(20);
      expect(article.apis.length).toBeGreaterThan(0);
      expect(article.source.length).toBeGreaterThan(0);
    }
  });

  it("propose six modèles pour chaque langage", () => {
    expect(templates.cpp).toHaveLength(6);
    expect(templates.lua).toHaveLength(6);
    expect(new Set(templates.cpp.map(template => template.id)).size).toBe(6);
    expect(new Set(templates.lua.map(template => template.id)).size).toBe(6);
  });
});
