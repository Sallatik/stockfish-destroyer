import { Chessground } from '@lichess-org/chessground';
import type { Api } from '@lichess-org/chessground/api';
import type { Key } from '@lichess-org/chessground/types';
import { Chess, type Move } from 'chess.js';
import '@lichess-org/chessground/assets/chessground.base.css';
import '@lichess-org/chessground/assets/chessground.brown.css';
import '@lichess-org/chessground/assets/chessground.cburnett.css';
import './style.css';

interface GameMeta {
  file: string;
  date: string;
  stockfish_elo: number;
  our_color: 'white' | 'black';
  result: string;
  outcome: 'win' | 'loss' | 'draw';
  plies: number;
}

const $ = <T extends HTMLElement>(sel: string) => document.querySelector(sel) as T;

/** Stockfish's maximum UCI_Elo. At this level the score is the win ratio and the average game length. */
const MAX_ELO = 3190;

interface LevelStats {
  elo: number;
  games: number;
  wins: number;
  draws: number;
  losses: number;
  ratio: number;
  avgMoves: number;
  avgMovesWins: number | null;
}

let allGames: GameMeta[] = [];
let levelFilter: number | null = null;

function levelStats(games: GameMeta[]): LevelStats[] {
  const byElo = new Map<number, GameMeta[]>();
  for (const g of games) byElo.set(g.stockfish_elo, [...(byElo.get(g.stockfish_elo) ?? []), g]);
  const avg = (gs: GameMeta[]) => gs.reduce((a, g) => a + g.plies / 2, 0) / gs.length;
  return [...byElo.entries()]
    .sort((a, b) => b[0] - a[0])
    .map(([elo, gs]) => {
      const wins = gs.filter((g) => g.outcome === 'win');
      return {
        elo,
        games: gs.length,
        wins: wins.length,
        draws: gs.filter((g) => g.outcome === 'draw').length,
        losses: gs.filter((g) => g.outcome === 'loss').length,
        ratio: wins.length / gs.length,
        avgMoves: avg(gs),
        avgMovesWins: wins.length ? avg(wins) : null,
      };
    });
}

const pct = (x: number) => `${Math.round(x * 100)}%`;
const moves = (x: number | null) => (x === null ? '–' : x.toFixed(1));

function renderStats() {
  const box = $('#stats');
  const levels = levelStats(allGames);
  if (!levels.length) { box.innerHTML = ''; return; }
  const max = levels.find((l) => l.elo === MAX_ELO);
  const bestWin = levels.find((l) => l.wins > 0);
  const top = max ?? levels[0];
  const headline = max
    ? `<div class="stat-label">Stockfish max level · Elo ${MAX_ELO}</div>
       <div class="stat-big">${pct(max.ratio)}<small>win ratio</small></div>
       <div class="stat-row"><span>${max.wins} / ${max.games} games won</span><span>${moves(max.avgMoves)} moves avg</span></div>`
    : `<div class="stat-label">Best verified win</div>
       <div class="stat-big">${bestWin ? `SF ${bestWin.elo}` : '–'}<small>${bestWin ? `${bestWin.wins} win${bestWin.wins > 1 ? 's' : ''}` : 'no wins yet'}</small></div>
       <div class="stat-row"><span>now playing SF ${top.elo}</span><span>${pct(top.ratio)} · ${moves(top.avgMoves)} moves avg</span></div>`;
  const rows = levels
    .map((l) => `<tr class="${l.elo === MAX_ELO ? 'max' : ''} ${l.wins ? 'won' : ''} ${levelFilter === l.elo ? 'selected' : ''}" data-elo="${l.elo}" title="click to filter the game list">
        <td>${l.elo}${l.elo === MAX_ELO ? ' ★' : ''}</td>
        <td class="wdl"><b>${l.wins}</b>-${l.draws}-${l.losses}</td>
        <td>${pct(l.ratio)}</td>
        <td>${moves(l.avgMoves)}</td>
      </tr>`)
    .join('');
  box.innerHTML = `<div class="stat-card ${max ? 'max' : ''}">${headline}</div>
    <table class="ladder">
      <thead><tr><th>Elo</th><th>W-D-L</th><th>ratio</th><th>moves</th></tr></thead>
      <tbody>${rows}</tbody>
    </table>
    <div class="stat-foot">${max ? 'At 3190 the win ratio and average moves decide.' : `Up to ${MAX_ELO}: win ratio and average moves decide there.`}</div>`;
  box.querySelectorAll<HTMLTableRowElement>('tr[data-elo]').forEach((tr) => {
    tr.onclick = () => {
      const elo = Number(tr.dataset.elo);
      levelFilter = levelFilter === elo ? null : elo;
      renderStats();
      renderList();
    };
  });
}

function renderList() {
  const list = $('#game-list');
  list.innerHTML = '';
  const games = levelFilter === null ? allGames : allGames.filter((g) => g.stockfish_elo === levelFilter);
  $('#games-title').textContent = levelFilter === null ? `Games (${games.length})` : `Games vs SF ${levelFilter} (${games.length})`;
  for (const g of [...games].reverse()) {
    const li = document.createElement('li');
    li.innerHTML = `<span class="outcome-${g.outcome}">${g.outcome.toUpperCase()}</span> vs SF ${g.stockfish_elo}
      <small>${g.date.replace('T', ' ')} · ${Math.ceil(g.plies / 2)} moves · we played ${g.our_color}</small>`;
    li.onclick = () => loadGame(g, li);
    list.append(li);
  }
  if (!allGames.length) list.innerHTML = '<li><small>No games yet. Run <code>uv run arena --elo 1320</code></small></li>';
}

let history: Move[] = [];
let fens: string[] = [];
let ply = 0;
let timer: number | undefined;

const cg: Api = Chessground($('#board'), { viewOnly: true, animation: { duration: 250 } });

function show(n: number) {
  ply = Math.max(0, Math.min(n, history.length));
  const last = history[ply - 1];
  cg.set({
    fen: fens[ply],
    lastMove: last ? [last.from as Key, last.to as Key] : undefined,
    check: last?.san.includes('+') || last?.san.includes('#'),
    turnColor: ply % 2 === 0 ? 'white' : 'black',
  });
  document.querySelectorAll('.ply').forEach((el, i) => el.classList.toggle('current', i === ply - 1));
}

function renderMoves() {
  const box = $('#moves');
  box.innerHTML = '';
  history.forEach((m, i) => {
    if (i % 2 === 0) box.insertAdjacentHTML('beforeend', `<span class="num">${i / 2 + 1}.</span>`);
    const el = document.createElement('span');
    el.className = 'ply';
    el.textContent = m.san;
    el.onclick = () => show(i + 1);
    box.append(el);
  });
}

async function loadGame(meta: GameMeta, li: HTMLElement) {
  document.querySelectorAll('#game-list li').forEach((el) => el.classList.remove('active'));
  li.classList.add('active');
  const pgn = await (await fetch(`/${meta.file}`)).text();
  const chess = new Chess();
  chess.loadPgn(pgn);
  history = chess.history({ verbose: true });
  fens = [history[0]?.before ?? chess.fen(), ...history.map((m) => m.after)];
  $('#title').textContent = `vs Stockfish ${meta.stockfish_elo} · ${meta.result} (${meta.outcome}, ${Math.ceil(meta.plies / 2)} moves)`;
  cg.set({ orientation: meta.our_color });
  renderMoves();
  show(0);
}

async function init() {
  const res = await fetch('/index.json');
  allGames = res.ok ? await res.json() : [];
  renderStats();
  renderList();
}

function step(action: string) {
  if (action !== 'play') {
    clearInterval(timer);
    timer = undefined;
  }
  if (action === 'first') show(0);
  if (action === 'prev') show(ply - 1);
  if (action === 'next') show(ply + 1);
  if (action === 'last') show(history.length);
  if (action === 'play') {
    if (timer) { clearInterval(timer); timer = undefined; return; }
    timer = window.setInterval(() => (ply >= history.length ? step('stop') : show(ply + 1)), 700);
  }
}

document.querySelectorAll<HTMLButtonElement>('#controls button').forEach((b) => (b.onclick = () => step(b.dataset.step!)));
document.addEventListener('keydown', (e) => {
  const map: Record<string, string> = { ArrowLeft: 'prev', ArrowRight: 'next', Home: 'first', End: 'last', ' ': 'play' };
  if (map[e.key]) { e.preventDefault(); step(map[e.key]); }
});

init();
