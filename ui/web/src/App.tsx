import { useEffect, useState } from 'react';
import { AnimatePresence } from 'framer-motion';
import { TitleScreen } from './components/TitleScreen';
import { GameScreen } from './components/GameScreen';
import { RulesPanel } from './components/RulesPanel';
import { recordVisit, getStats, type Stats } from './lib/stats';
import type { Color, Difficulty } from './lib/wasmEngine';

type View = { name: 'title' } | { name: 'game'; color: Color; difficulty: Difficulty; key: number };

let nextGameKey = 0;

export default function App() {
  const [view, setView] = useState<View>({ name: 'title' });
  const [showRules, setShowRules] = useState(false);
  const [stats, setStats] = useState<Stats>(getStats());

  useEffect(() => {
    setStats(recordVisit());
  }, []);

  return (
    <div className="min-h-screen w-full flex items-start justify-center">
      {view.name === 'title' ? (
        <TitleScreen
          stats={stats}
          onShowRules={() => setShowRules(true)}
          onPlay={(color, difficulty) =>
            setView({ name: 'game', color, difficulty, key: nextGameKey++ })
          }
        />
      ) : (
        <GameScreen
          key={view.key}
          playerColor={view.color}
          difficulty={view.difficulty}
          onExit={() => {
            setStats(getStats());
            setView({ name: 'title' });
          }}
          onShowRules={() => setShowRules(true)}
        />
      )}

      <AnimatePresence>{showRules && <RulesPanel onClose={() => setShowRules(false)} />}</AnimatePresence>
    </div>
  );
}
