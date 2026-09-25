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
  $('#title').textContent = `vs Stockfish ${meta.stockfish_elo} · ${meta.result} (${meta.outcome})`;
  cg.set({ orientation: meta.our_color });
  renderMoves();
  show(0);
}

async function init() {
  const res = await fetch('/index.json');
  const games: GameMeta[] = res.ok ? await res.json() : [];
  const list = $('#game-list');
  for (const g of games.reverse()) {
    const li = document.createElement('li');
    li.innerHTML = `<span class="outcome-${g.outcome}">${g.outcome.toUpperCase()}</span> vs SF ${g.stockfish_elo}
      <small>${g.date.replace('T', ' ')} · ${g.plies} plies · we played ${g.our_color}</small>`;
    li.onclick = () => loadGame(g, li);
    list.append(li);
  }
  if (!games.length) list.innerHTML = '<li><small>No games yet. Run <code>uv run arena --elo 1320</code></small></li>';
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
