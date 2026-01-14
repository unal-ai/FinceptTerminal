import React, { useState, useEffect, useRef, useCallback } from 'react';
import { LayoutDashboard, Newspaper, LineChart, MessageSquare, ArrowLeft, Play, Pause } from 'lucide-react';
import { useInterfaceMode } from '@/contexts/InterfaceModeContext';

// Import tabs
import DashboardTab from '@/components/tabs/DashboardTab';
import MarketsTab from '@/components/tabs/MarketsTab';
import NewsTab from '@/components/tabs/NewsTab';
import ChatTab from '@/components/tabs/ChatTab';

const VIEWS = ['dashboard', 'markets', 'news', 'chat'] as const;
const DEFAULT_ROTATION_INTERVAL = 15000; // 15 seconds
const IDLE_TIMEOUT = 5000; // 5 seconds of inactivity to resume

export const WarRoomMode: React.FC = () => {
    const { setMode } = useInterfaceMode();
    const [activeView, setActiveView] = useState<'dashboard' | 'markets' | 'news' | 'chat'>('dashboard');
    const [carouselEnabled, setCarouselEnabled] = useState(true); // User preference for carousel
    const [userActive, setUserActive] = useState(false); // Is user actively browsing?
    const [progress, setProgress] = useState(0); // 0-100 progress for current tab
    const intervalRef = useRef<NodeJS.Timeout | null>(null);
    const progressRef = useRef<NodeJS.Timeout | null>(null);
    const idleTimerRef = useRef<NodeJS.Timeout | null>(null);

    // Effective carousel state: enabled by user AND not actively browsing
    const isCarouselActive = carouselEnabled && !userActive;

    // Handle user activity (mouse movement)
    useEffect(() => {
        const handleMouseMove = () => {
            // User is active - pause carousel
            setUserActive(true);

            // Clear existing idle timer
            if (idleTimerRef.current) {
                clearTimeout(idleTimerRef.current);
            }

            // Set new idle timer - resume after IDLE_TIMEOUT
            idleTimerRef.current = setTimeout(() => {
                setUserActive(false);
            }, IDLE_TIMEOUT);
        };

        window.addEventListener('mousemove', handleMouseMove);

        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            if (idleTimerRef.current) clearTimeout(idleTimerRef.current);
        };
    }, []);

    // Rotate to next view
    const rotateToNext = useCallback(() => {
        setActiveView(prev => {
            const currentIndex = VIEWS.indexOf(prev);
            const nextIndex = (currentIndex + 1) % VIEWS.length;
            return VIEWS[nextIndex];
        });
        setProgress(0);
    }, []);

    // Handle carousel rotation
    useEffect(() => {
        if (isCarouselActive) {
            // Reset progress when starting
            setProgress(0);

            // Main rotation interval
            intervalRef.current = setInterval(rotateToNext, DEFAULT_ROTATION_INTERVAL);

            // Progress update interval (update every 100ms for smooth animation)
            progressRef.current = setInterval(() => {
                setProgress(prev => Math.min(prev + (100 / (DEFAULT_ROTATION_INTERVAL / 100)), 100));
            }, 100);
        }

        return () => {
            if (intervalRef.current) clearInterval(intervalRef.current);
            if (progressRef.current) clearInterval(progressRef.current);
        };
    }, [isCarouselActive, rotateToNext]);

    // Reset progress when view changes manually
    useEffect(() => {
        if (isCarouselActive) {
            setProgress(0);
            // Reset the rotation timer when manually switching
            if (intervalRef.current) clearInterval(intervalRef.current);
            intervalRef.current = setInterval(rotateToNext, DEFAULT_ROTATION_INTERVAL);
        }
    }, [activeView, isCarouselActive, rotateToNext]);

    const toggleCarousel = () => {
        setCarouselEnabled(prev => !prev);
        setProgress(0);
    };

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                setMode('terminal');
            }
            // Number keys to switch views
            // We check for modifiers to avoid conflict if user is typing (though in war room, inputs might be focused)
            // Ideally we only trigger if not in an input, but for now let's use Alt+Number or just Number if we assume "monitor mode"
            // The prompt says "use keyboard to switch", usually implies quick keys like 1, 2, 3, 4.
            // But if there is a chat, user types.
            // So we should check if active element is input.

            const target = e.target as HTMLElement;
            const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

            if (!isInput) {
                if (e.key === '1') setActiveView('dashboard');
                if (e.key === '2') setActiveView('markets');
                if (e.key === '3') setActiveView('news');
                if (e.key === '4') setActiveView('chat');
                if (e.key === ' ') {
                    e.preventDefault();
                    toggleCarousel();
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [setMode]);

    // All tabs are kept mounted to preserve their state and data (hotloaded)
    // We use CSS to show/hide instead of conditional rendering

    const navItems = [
        { id: 'dashboard', icon: LayoutDashboard, label: 'Dashboard', key: '1' },
        { id: 'markets', icon: LineChart, label: 'Markets', key: '2' },
        { id: 'news', icon: Newspaper, label: 'News', key: '3' },
        { id: 'chat', icon: MessageSquare, label: 'AI Chat', key: '4' },
    ];

    return (
        <div className="fixed inset-0 bg-black z-[100] flex overflow-hidden">
            {/* Main Content Area - All tabs mounted, CSS controls visibility */}
            <div className="flex-1 overflow-hidden relative border-r border-zinc-800">
                <div className="absolute inset-0" style={{ display: activeView === 'dashboard' ? 'block' : 'none' }}>
                    <DashboardTab />
                </div>
                <div className="absolute inset-0" style={{ display: activeView === 'markets' ? 'block' : 'none' }}>
                    <MarketsTab />
                </div>
                <div className="absolute inset-0" style={{ display: activeView === 'news' ? 'block' : 'none' }}>
                    <NewsTab />
                </div>
                <div className="absolute inset-0" style={{ display: activeView === 'chat' ? 'block' : 'none' }}>
                    <ChatTab />
                </div>
            </div>

            {/* Right Sidebar */}
            <div className="w-20 bg-black border-l border-zinc-800 flex flex-col items-center py-4 gap-3 z-50 flex-shrink-0">
                {/* Branding - Logo on top, WAR ROOM with financial theme */}
                <div className="flex flex-col items-center gap-2 mb-2 px-1">
                    {/* UNAL Logo */}
                    <img
                        src="/unal_logo.png"
                        alt="UNAL"
                        className="w-14 h-auto opacity-90 hover:opacity-100 transition-opacity"
                    />

                    {/* Divider */}
                    <div className="w-10 h-px bg-gradient-to-r from-transparent via-cyan-500/40 to-transparent" />

                    {/* WAR ROOM with financial theme */}
                    <div className="flex flex-col items-center">
                        {/* Mini candlestick chart decoration - bullish uptrend */}
                        <div className="flex gap-[2px] mb-1 items-end">
                            <div className="w-[2px] h-2 bg-red-500 rounded-sm" />
                            <div className="w-[2px] h-3 bg-green-500 rounded-sm" />
                            <div className="w-[2px] h-2 bg-red-500 rounded-sm" />
                            <div className="w-[2px] h-4 bg-green-500 rounded-sm" />
                            <div className="w-[2px] h-5 bg-green-500 rounded-sm" />
                        </div>
                        <div className="text-cyan-400 font-black text-[9px] tracking-[0.12em] leading-tight text-center">
                            WAR
                        </div>
                        <div className="text-cyan-400 font-black text-[9px] tracking-[0.12em] leading-tight text-center">
                            ROOM
                        </div>
                        {/* Up/Down arrows decoration */}
                        <div className="flex gap-1 mt-1 text-[8px]">
                            <span className="text-green-500">▲</span>
                            <span className="text-red-500">▼</span>
                        </div>
                    </div>
                </div>

                {navItems.map(item => (
                    <button
                        key={item.id}
                        onClick={() => setActiveView(item.id as any)}
                        className={`p-3 rounded-lg transition-all relative group ${activeView === item.id ? 'bg-cyan-600/20 text-cyan-400' : 'text-zinc-500 hover:text-cyan-400 hover:bg-zinc-900'}`}
                        title={`${item.label} (${item.key})`}
                    >
                        <item.icon size={20} />
                        {/* Tooltip on left */}
                        <div className="absolute right-full top-1/2 -translate-y-1/2 mr-3 px-2 py-1 bg-zinc-800 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap border border-zinc-700 pointer-events-none">
                            {item.label} <span className="text-zinc-400">({item.key})</span>
                        </div>
                    </button>
                ))}

                <div className="mt-auto flex flex-col gap-4">
                    {/* Carousel Toggle Button with Progress Ring */}
                    <button
                        onClick={toggleCarousel}
                        className={`p-3 rounded-lg transition-all relative group ${!carouselEnabled
                            ? 'text-zinc-500 hover:text-green-500 hover:bg-zinc-900'
                            : isCarouselActive
                                ? 'bg-green-600/20 text-green-500'
                                : 'bg-sky-600/20 text-sky-400' // Paused due to user activity - soft blue
                            }`}
                        title={
                            !carouselEnabled
                                ? '开始轮博 (Space)'
                                : userActive
                                    ? '轮博已暂停 (鼠标活动中)'
                                    : '暂停轮博 (Space)'
                        }
                    >
                        {/* Progress Ring SVG */}
                        {isCarouselActive && (
                            <svg className="absolute inset-0 w-full h-full -rotate-90" viewBox="0 0 44 44">
                                <circle
                                    cx="22" cy="22" r="18"
                                    fill="none"
                                    stroke="currentColor"
                                    strokeWidth="2"
                                    strokeDasharray={`${2 * Math.PI * 18}`}
                                    strokeDashoffset={`${2 * Math.PI * 18 * (1 - progress / 100)}`}
                                    className="text-green-500/50 transition-all duration-100"
                                />
                            </svg>
                        )}
                        {!carouselEnabled ? <Play size={20} /> : userActive ? <Pause size={20} /> : <Play size={20} />}
                        <div className="absolute right-full top-1/2 -translate-y-1/2 mr-3 px-2 py-1 bg-zinc-800 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap border border-zinc-700 pointer-events-none">
                            {!carouselEnabled
                                ? '开始轮博'
                                : userActive
                                    ? '已暂停 (鼠标活动)'
                                    : '轮博中...'
                            } <span className="text-zinc-400">(Space)</span>
                        </div>
                    </button>

                    <button
                        onClick={() => setMode('terminal')}
                        className="p-3 rounded-lg text-zinc-500 hover:text-red-500 hover:bg-red-500/10 transition-all relative group"
                        title="Exit War Room (Esc)"
                    >
                        <ArrowLeft size={20} />
                        <div className="absolute right-full top-1/2 -translate-y-1/2 mr-3 px-2 py-1 bg-zinc-800 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap border border-zinc-700 pointer-events-none">
                            Exit <span className="text-zinc-400">(Esc)</span>
                        </div>
                    </button>
                </div>
            </div>
        </div>
    );
};
