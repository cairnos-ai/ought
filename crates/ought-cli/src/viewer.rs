use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use axum::{
    Router,
    response::{Html, Json},
    routing::get,
};
use serde_json::{Value, json};

use ought_spec::{Clause, Config, Keyword, Section, Spec, SpecGraph};

// ─── JSON serialization ────────────────────────────────────────────────────

fn spec_to_json(spec: &Spec) -> Value {
    json!({
        "name": spec.name,
        "source_path": spec.source_path.display().to_string(),
        "metadata": {
            "context": spec.metadata.context,
            "sources": spec.metadata.sources,
            "schemas": spec.metadata.schemas,
            "requires": spec.metadata.requires.iter().map(|r| json!({
                "label": r.label,
                "path": r.path.display().to_string(),
                "anchor": r.anchor,
            })).collect::<Vec<_>>(),
        },
        "sections": spec.sections.iter().map(section_to_json).collect::<Vec<_>>(),
    })
}

fn section_to_json(section: &Section) -> Value {
    json!({
        "title": section.title,
        "depth": section.depth,
        "prose": section.prose,
        "clauses": section.clauses.iter().map(clause_to_json).collect::<Vec<_>>(),
        "subsections": section.subsections.iter().map(section_to_json).collect::<Vec<_>>(),
    })
}

fn clause_to_json(clause: &Clause) -> Value {
    let temporal = clause.temporal.as_ref().map(|t| match t {
        ought_spec::Temporal::Invariant => json!({ "kind": "invariant" }),
        ought_spec::Temporal::Deadline(dur) => json!({ "kind": "deadline", "duration": format!("{:?}", dur) }),
    });

    json!({
        "id": clause.id.0,
        "keyword": format!("{:?}", clause.keyword),
        "severity": format!("{:?}", clause.severity),
        "text": clause.text,
        "condition": clause.condition,
        "otherwise": clause.otherwise.iter().map(clause_to_json).collect::<Vec<_>>(),
        "temporal": temporal,
        "hints": clause.hints,
    })
}

fn keyword_display(kw: &Keyword) -> &'static str {
    match kw {
        Keyword::Must => "Must",
        Keyword::MustNot => "MustNot",
        Keyword::Should => "Should",
        Keyword::ShouldNot => "ShouldNot",
        Keyword::May => "May",
        Keyword::Wont => "Wont",
        Keyword::Given => "Given",
        Keyword::Otherwise => "Otherwise",
        Keyword::MustAlways => "MustAlways",
        Keyword::MustBy => "MustBy",
    }
}

fn count_clauses(sections: &[Section]) -> usize {
    sections
        .iter()
        .map(|s| {
            s.clauses.len()
                + s.clauses.iter().map(|c| c.otherwise.len()).sum::<usize>()
                + count_clauses(&s.subsections)
        })
        .sum()
}

fn count_sections(sections: &[Section]) -> usize {
    sections
        .iter()
        .map(|s| 1 + count_sections(&s.subsections))
        .sum()
}

fn count_by_keyword(sections: &[Section], counts: &mut HashMap<&'static str, usize>) {
    for section in sections {
        for clause in &section.clauses {
            *counts.entry(keyword_display(&clause.keyword)).or_insert(0) += 1;
            for ow in &clause.otherwise {
                *counts.entry(keyword_display(&ow.keyword)).or_insert(0) += 1;
            }
        }
        count_by_keyword(&section.subsections, counts);
    }
}

fn build_api_response(specs: &[Spec]) -> Value {
    let total_specs = specs.len();
    let total_sections: usize = specs.iter().map(|s| count_sections(&s.sections)).sum();
    let total_clauses: usize = specs.iter().map(|s| count_clauses(&s.sections)).sum();

    let mut by_keyword: HashMap<&str, usize> = HashMap::new();
    for spec in specs {
        count_by_keyword(&spec.sections, &mut by_keyword);
    }

    json!({
        "specs": specs.iter().map(spec_to_json).collect::<Vec<_>>(),
        "stats": {
            "total_specs": total_specs,
            "total_sections": total_sections,
            "total_clauses": total_clauses,
            "by_keyword": by_keyword,
        },
    })
}

// ─── HTML template ─────────────────────────────────────────────────────────

const VIEWER_HTML: &str = r##"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>ought viewer</title>
<style>
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}

/* Light theme */
:root,[data-theme="light"]{
  --bg:#f5f5f7;--surface:#fff;--surface2:#f8f9fb;--border:#e5e7eb;
  --text:#111827;--text2:#6b7280;--text3:#9ca3af;
  --accent:#2563eb;--accent-subtle:#eff6ff;
  --shadow:0 1px 3px rgba(0,0,0,.06),0 1px 2px rgba(0,0,0,.04);
  --shadow-lg:0 4px 12px rgba(0,0,0,.08);
  --radius:8px;--header-bg:#0f172a;--header-text:#f1f5f9;
  --sidebar-hover:#f1f5f9;--sidebar-active:#e0e7ff;
  --clause-border:#f3f4f6;
  --hint-bg:#f8fafc;--hint-border:#e2e8f0;
  --meta-bg:#f8fafc;
}

/* Dark theme */
[data-theme="dark"]{
  --bg:#0f1117;--surface:#1a1d27;--surface2:#21242f;--border:#2d3140;
  --text:#e5e7eb;--text2:#9ca3af;--text3:#6b7280;
  --accent:#60a5fa;--accent-subtle:#1e293b;
  --shadow:0 1px 3px rgba(0,0,0,.3);
  --shadow-lg:0 4px 12px rgba(0,0,0,.4);
  --header-bg:#090b10;--header-text:#e2e8f0;
  --sidebar-hover:#252836;--sidebar-active:#1e293b;
  --clause-border:#252836;
  --hint-bg:#1e2130;--hint-border:#2d3140;
  --meta-bg:#1e2130;
}

body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI","Inter",Roboto,sans-serif;
background:var(--bg);color:var(--text);line-height:1.65;display:flex;flex-direction:column;height:100vh;
-webkit-font-smoothing:antialiased;-moz-osx-font-smoothing:grayscale}
a{color:var(--accent);text-decoration:none}
::selection{background:var(--accent);color:#fff}

/* Header */
.header{background:var(--header-bg);color:var(--header-text);padding:0 24px;
display:flex;align-items:center;gap:20px;flex-shrink:0;height:48px}
.header h1{font-size:17px;font-weight:800;letter-spacing:-.3px;opacity:.95}
.header h1 span{opacity:.4;font-weight:400;margin-left:2px}
.header .stats{font-size:12px;color:var(--text3);margin-left:auto;display:flex;gap:14px;align-items:center}
.header .stats span{white-space:nowrap}
.theme-toggle{background:none;border:1px solid rgba(255,255,255,.15);color:var(--header-text);
cursor:pointer;padding:4px 10px;border-radius:6px;font-size:13px;transition:border-color .15s}
.theme-toggle:hover{border-color:rgba(255,255,255,.35)}

/* Search */
.search-bar{padding:8px 24px;background:var(--surface);border-bottom:1px solid var(--border);
flex-shrink:0;display:flex;gap:10px;align-items:center;flex-wrap:wrap}
.search-bar input{flex:1;min-width:200px;padding:7px 14px;border:1px solid var(--border);
border-radius:var(--radius);font-size:13px;outline:none;background:var(--surface2);color:var(--text);
transition:border-color .15s,box-shadow .15s}
.search-bar input:focus{border-color:var(--accent);box-shadow:0 0 0 3px rgba(37,99,235,.1)}
.search-bar input::placeholder{color:var(--text3)}
.filter-pills{display:flex;gap:4px;flex-wrap:wrap}
.filter-pill{padding:2px 10px;border-radius:12px;font-size:11px;font-weight:700;cursor:pointer;
border:1.5px solid transparent;opacity:.5;transition:all .15s}
.filter-pill:hover{opacity:.8}.filter-pill.active{opacity:1;border-color:currentColor}

/* Layout */
.layout{display:flex;flex:1;overflow:hidden}
.sidebar{width:272px;min-width:220px;background:var(--surface);border-right:1px solid var(--border);
overflow-y:auto;flex-shrink:0;padding:12px 0}
.sidebar::-webkit-scrollbar{width:6px}.sidebar::-webkit-scrollbar-thumb{background:var(--border);border-radius:3px}
.main{flex:1;overflow-y:auto;padding:28px 36px;max-width:960px}
.main::-webkit-scrollbar{width:8px}.main::-webkit-scrollbar-thumb{background:var(--border);border-radius:4px}

/* Sidebar tree */
.tree-item{padding:5px 16px;cursor:pointer;font-size:13px;display:flex;align-items:center;gap:6px;
color:var(--text);transition:background .1s;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;
border-radius:4px;margin:0 6px}
.tree-item:hover{background:var(--sidebar-hover)}
.tree-item.active{background:var(--sidebar-active);font-weight:600;color:var(--accent)}
.tree-item .arrow{font-size:9px;width:14px;text-align:center;transition:transform .2s;flex-shrink:0;opacity:.5}
.tree-item .arrow.open{transform:rotate(90deg);opacity:.8}
.tree-section{padding-left:28px;font-size:12px;color:var(--text2);margin:0 6px}
.tree-children{display:none}.tree-children.open{display:block}
.tree-count{font-size:11px;color:var(--text3);margin-left:auto;padding-right:4px;flex-shrink:0}

/* Spec header */
.spec-header h2{font-size:22px;font-weight:800;margin-bottom:4px;letter-spacing:-.3px}
.spec-path{font-size:12px;color:var(--text3);margin-bottom:16px;font-family:"SF Mono",Menlo,Consolas,monospace}
.meta-block{background:var(--meta-bg);border:1px solid var(--border);border-radius:var(--radius);
padding:14px 18px;margin-bottom:24px;font-size:13px;color:var(--text2)}
.meta-block .meta-row{margin-bottom:4px}.meta-block .meta-label{font-weight:600;color:var(--text)}

/* Section card */
.section-card{background:var(--surface);border:1px solid var(--border);border-radius:var(--radius);
box-shadow:var(--shadow);margin-bottom:14px;overflow:hidden;transition:box-shadow .15s}
.section-card:hover{box-shadow:var(--shadow-lg)}
.section-head{padding:12px 18px;cursor:pointer;display:flex;align-items:center;gap:8px;
font-weight:700;font-size:14px;user-select:none;color:var(--text);letter-spacing:-.2px}
.section-head:hover{background:var(--surface2)}
.section-head .arrow{font-size:10px;transition:transform .2s;opacity:.4}
.section-head .arrow.open{transform:rotate(90deg);opacity:.7}
.section-head .clause-count{font-size:11px;color:var(--text3);font-weight:400;margin-left:auto}
.section-body{padding:4px 18px 14px;display:none}
.section-body.open{display:block}
.section-prose{font-size:13px;color:var(--text2);margin-bottom:10px;padding:8px 12px;
background:var(--surface2);border-radius:6px;border-left:3px solid var(--border)}
.subsection{margin-left:12px;border-left:2px solid var(--border);padding-left:14px;margin-top:12px}
.subsection-title{font-weight:700;font-size:13px;margin-bottom:6px;color:var(--text);letter-spacing:-.1px}

/* Clause */
.clause{display:flex;gap:10px;padding:9px 4px;border-bottom:1px solid var(--clause-border);align-items:baseline}
.clause:last-child{border-bottom:none}
.clause-condition{font-size:12px;color:var(--accent);padding:6px 0 2px;font-weight:500;opacity:.8}
.clause-text{flex:1;font-size:13.5px;line-height:1.55}
.clause-hints{margin-top:6px}
.clause-hint{background:var(--hint-bg);border:1px solid var(--hint-border);border-radius:6px;
padding:8px 12px;font-family:"SF Mono",Menlo,Consolas,monospace;font-size:12px;margin-top:4px;
white-space:pre-wrap;word-break:break-all;line-height:1.5}

/* Otherwise chain */
.otherwise-chain{margin-left:32px;border-left:2px solid var(--border);padding-left:12px;margin-top:2px}
.otherwise-chain .clause{opacity:.85}

/* Keyword badges */
.kw{display:inline-block;padding:2px 9px;border-radius:10px;font-size:10px;font-weight:800;
white-space:nowrap;flex-shrink:0;text-transform:uppercase;letter-spacing:.5px;min-width:52px;text-align:center}
.kw-Must,.kw-MustNot,.kw-MustAlways,.kw-MustBy{background:#fef2f2;color:#dc2626}
.kw-Should,.kw-ShouldNot{background:#fffbeb;color:#d97706}
.kw-May{background:var(--surface2);color:var(--text2)}
.kw-Wont{background:#f5f3ff;color:#7c3aed}
.kw-Given{background:#eff6ff;color:#2563eb}
.kw-Otherwise{background:#fff7ed;color:#ea580c}
[data-theme="dark"] .kw-Must,[data-theme="dark"] .kw-MustNot,
[data-theme="dark"] .kw-MustAlways,[data-theme="dark"] .kw-MustBy{background:#450a0a;color:#fca5a5}
[data-theme="dark"] .kw-Should,[data-theme="dark"] .kw-ShouldNot{background:#451a03;color:#fcd34d}
[data-theme="dark"] .kw-Wont{background:#2e1065;color:#c4b5fd}
[data-theme="dark"] .kw-Given{background:#172554;color:#93c5fd}
[data-theme="dark"] .kw-Otherwise{background:#431407;color:#fdba74}

/* Temporal badges */
.temporal{font-size:10px;padding:2px 7px;border-radius:8px;background:#ecfdf5;color:#059669;
margin-left:6px;font-weight:700;letter-spacing:.3px}
[data-theme="dark"] .temporal{background:#064e3b;color:#6ee7b7}

/* Responsive */
@media(max-width:768px){.sidebar{display:none}.main{padding:16px}}

/* Transitions */
.section-card,.tree-item,.clause{transition:background .1s}
</style>
</head>
<body>
<div class="header">
  <h1>ought<span>viewer</span></h1>
  <div class="stats" id="stats"></div>
  <button class="theme-toggle" onclick="toggleTheme()" id="theme-btn" title="Toggle dark/light mode">Light</button>
</div>
<div class="search-bar">
  <input type="text" id="search" placeholder="Search clauses...">
  <div class="filter-pills" id="filters"></div>
</div>
<div class="layout">
  <div class="sidebar" id="sidebar"></div>
  <div class="main" id="main"><p style="color:#999;padding:40px">Loading specs...</p></div>
</div>
<script>
let DATA=null,activeSpec=null,activeFilter=null;

function toggleTheme(){
  const html=document.documentElement;
  const current=html.getAttribute("data-theme");
  const next=current==="dark"?"light":"dark";
  html.setAttribute("data-theme",next);
  document.getElementById("theme-btn").textContent=next==="dark"?"Dark":"Light";
  localStorage.setItem("ought-theme",next);
}
// Restore saved theme
(function(){
  const saved=localStorage.getItem("ought-theme");
  if(saved){document.documentElement.setAttribute("data-theme",saved);
    document.addEventListener("DOMContentLoaded",()=>{
      const btn=document.getElementById("theme-btn");if(btn)btn.textContent=saved==="dark"?"Dark":"Light"})};
})();
const KW_LABELS={Must:"MUST",MustNot:"MUST NOT",Should:"SHOULD",ShouldNot:"SHOULD NOT",
  May:"MAY",Wont:"WONT",Given:"GIVEN",Otherwise:"OTHERWISE",MustAlways:"MUST ALWAYS",MustBy:"MUST BY"};

async function init(){
  const r=await fetch("/api/specs");DATA=await r.json();
  renderStats();renderFilters();renderSidebar();
  if(DATA.specs.length)selectSpec(0);
}

function renderStats(){
  const s=DATA.stats,el=document.getElementById("stats");
  el.innerHTML=`<span>${s.total_specs} specs</span><span>${s.total_sections} sections</span><span>${s.total_clauses} clauses</span>`;
  const bk=s.by_keyword;
  for(const[k,v]of Object.entries(bk)){el.innerHTML+=`<span class="kw kw-${k}" style="font-size:11px">${KW_LABELS[k]||k} ${v}</span>`}
}

function renderFilters(){
  const el=document.getElementById("filters");
  const kws=["Must","MustNot","Should","ShouldNot","May","Wont","Given","Otherwise","MustAlways","MustBy"];
  kws.forEach(k=>{
    if(!DATA.stats.by_keyword[k])return;
    const pill=document.createElement("span");
    pill.className=`filter-pill kw kw-${k}`;pill.textContent=KW_LABELS[k]||k;
    pill.onclick=()=>{
      if(activeFilter===k){activeFilter=null;pill.classList.remove("active")}
      else{document.querySelectorAll(".filter-pill").forEach(p=>p.classList.remove("active"));activeFilter=k;pill.classList.add("active")}
      renderMain();
    };el.appendChild(pill);
  });
}

function renderSidebar(){
  const el=document.getElementById("sidebar");el.innerHTML="";
  DATA.specs.forEach((spec,si)=>{
    const item=document.createElement("div");
    const cc=countSpecClauses(spec);
    item.className="tree-item";item.dataset.idx=si;
    item.innerHTML=`<span class="arrow">&#9654;</span>${esc(spec.name)}<span class="tree-count">${cc}</span>`;
    item.onclick=e=>{e.stopPropagation();selectSpec(si);toggleTree(item)};
    el.appendChild(item);
    const children=document.createElement("div");children.className="tree-children";
    renderTreeSections(spec.sections,children,si);
    el.appendChild(children);
  });
}

function renderTreeSections(sections,parent,si){
  sections.forEach(sec=>{
    const item=document.createElement("div");item.className="tree-item tree-section";
    item.textContent=sec.title;item.onclick=e=>{e.stopPropagation();selectSpec(si);
      setTimeout(()=>{const t=document.getElementById("sec-"+sec.title.replace(/\s+/g,"-"));if(t)t.scrollIntoView({behavior:"smooth"})},50)};
    parent.appendChild(item);
    if(sec.subsections.length){const ch=document.createElement("div");ch.className="tree-children open";
      renderTreeSections(sec.subsections,ch,si);parent.appendChild(ch)}
  });
}

function toggleTree(item){
  const next=item.nextElementSibling;
  if(next&&next.classList.contains("tree-children")){
    next.classList.toggle("open");item.querySelector(".arrow").classList.toggle("open")}
}

function selectSpec(idx){
  activeSpec=idx;
  document.querySelectorAll(".tree-item[data-idx]").forEach(el=>{
    el.classList.toggle("active",parseInt(el.dataset.idx)===idx);
    if(parseInt(el.dataset.idx)===idx){const a=el.querySelector(".arrow");if(a)a.classList.add("open");
      const n=el.nextElementSibling;if(n&&n.classList.contains("tree-children"))n.classList.add("open")}
  });
  renderMain();
}

function renderMain(){
  const el=document.getElementById("main");
  if(activeSpec===null){el.innerHTML="<p>Select a spec</p>";return}
  const spec=DATA.specs[activeSpec],q=document.getElementById("search").value.toLowerCase();
  let h=`<div class="spec-header"><h2>${esc(spec.name)}</h2></div>`;
  h+=`<div class="spec-path">${esc(spec.source_path)}</div>`;
  // metadata
  const m=spec.metadata;
  if(m.context||m.sources.length||m.schemas.length||m.requires.length){
    h+=`<div class="meta-block">`;
    if(m.context)h+=`<div class="meta-row"><span class="meta-label">Context:</span> ${esc(m.context)}</div>`;
    if(m.sources.length)h+=`<div class="meta-row"><span class="meta-label">Sources:</span> ${m.sources.map(esc).join(", ")}</div>`;
    if(m.schemas.length)h+=`<div class="meta-row"><span class="meta-label">Schemas:</span> ${m.schemas.map(esc).join(", ")}</div>`;
    if(m.requires.length)h+=`<div class="meta-row"><span class="meta-label">Requires:</span> ${m.requires.map(r=>esc(r.label||r.path)).join(", ")}</div>`;
    h+=`</div>`;
  }
  h+=renderSections(spec.sections,q);
  el.innerHTML=h;
  // wire section toggles
  el.querySelectorAll(".section-head").forEach(sh=>{
    sh.onclick=()=>{sh.querySelector(".arrow").classList.toggle("open");
      sh.nextElementSibling.classList.toggle("open")}
  });
}

function renderSections(sections,q,depth){
  depth=depth||0;let h="";
  sections.forEach(sec=>{
    const clauses=filterClauses(sec.clauses,q);
    const subHtml=renderSections(sec.subsections,q,depth+1);
    if(!clauses.length&&!subHtml&&q)return;
    const id="sec-"+sec.title.replace(/\s+/g,"-");
    if(depth>0){
      h+=`<div class="subsection" id="${id}"><div class="subsection-title">${esc(sec.title)}</div>`;
      if(sec.prose)h+=`<div class="section-prose">${esc(sec.prose)}</div>`;
      h+=renderClauseList(clauses)+subHtml+`</div>`;
    }else{
      const cc=sec.clauses.length;
      h+=`<div class="section-card" id="${id}"><div class="section-head"><span class="arrow open">&#9654;</span>${esc(sec.title)}<span class="clause-count">${cc} clause${cc!==1?"s":""}</span></div>`;
      h+=`<div class="section-body open">`;
      if(sec.prose)h+=`<div class="section-prose">${esc(sec.prose)}</div>`;
      h+=renderClauseList(clauses)+subHtml+`</div></div>`;
    }
  });
  return h;
}

function filterClauses(clauses,q){
  return clauses.filter(c=>{
    if(activeFilter&&c.keyword!==activeFilter)return false;
    if(q&&!c.text.toLowerCase().includes(q)&&!(c.condition||"").toLowerCase().includes(q)&&!c.id.toLowerCase().includes(q))return false;
    return true;
  });
}

function renderClauseList(clauses){
  if(!clauses.length)return"";
  let h="";
  clauses.forEach(c=>{
    if(c.condition)h+=`<div class="clause-condition">GIVEN: ${esc(c.condition)}</div>`;
    h+=`<div class="clause"><span class="kw kw-${c.keyword}">${KW_LABELS[c.keyword]||c.keyword}</span>`;
    h+=`<div class="clause-text">${esc(c.text)}`;
    if(c.temporal){
      if(c.temporal.kind==="invariant")h+=` <span class="temporal">INVARIANT</span>`;
      else if(c.temporal.kind==="deadline")h+=` <span class="temporal">${esc(c.temporal.duration)}</span>`;
    }
    if(c.hints&&c.hints.length){h+=`<div class="clause-hints">`;c.hints.forEach(hint=>{h+=`<div class="clause-hint">${esc(hint)}</div>`});h+=`</div>`}
    h+=`</div></div>`;
    if(c.otherwise&&c.otherwise.length){h+=`<div class="otherwise-chain">`;c.otherwise.forEach(ow=>{
      h+=`<div class="clause"><span class="kw kw-${ow.keyword}">${KW_LABELS[ow.keyword]||ow.keyword}</span>`;
      h+=`<div class="clause-text">${esc(ow.text)}</div></div>`;
    });h+=`</div>`}
  });
  return h;
}

function countSpecClauses(spec){let n=0;function cs(secs){secs.forEach(s=>{n+=s.clauses.length;s.clauses.forEach(c=>n+=c.otherwise.length);cs(s.subsections)})}cs(spec.sections);return n}
function esc(s){if(!s)return"";return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}

document.getElementById("search").addEventListener("input",()=>renderMain());
init();
</script>
</body>
</html>
"##;

// ─── Server ────────────────────────────────────────────────────────────────

pub fn cmd_view(
    config_path: &Option<PathBuf>,
    port: u16,
    no_open: bool,
) -> anyhow::Result<()> {
    let (cfg_path, config) = match config_path {
        Some(path) => {
            let config = Config::load(path)?;
            (path.clone(), config)
        }
        None => Config::discover()?,
    };

    let config_dir = cfg_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    let roots: Vec<PathBuf> = config
        .specs
        .roots
        .iter()
        .map(|r| config_dir.join(r))
        .collect();

    let graph = SpecGraph::from_roots(&roots).map_err(|errors| {
        let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        anyhow::anyhow!("spec parse errors:\n  {}", messages.join("\n  "))
    })?;

    let api_json = build_api_response(graph.specs());

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let json_data = api_json.clone();
        let app = Router::new()
            .route("/", get(|| async { Html(VIEWER_HTML) }))
            .route(
                "/api/specs",
                get(move || {
                    let data = json_data.clone();
                    async move { Json(data) }
                }),
            );

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        eprintln!("Serving ought viewer at http://localhost:{}", port);

        if !no_open {
            let url = format!("http://localhost:{}", port);
            let _ = std::process::Command::new("open").arg(&url).spawn();
        }

        axum::serve(listener, app).await?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
