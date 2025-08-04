import React, { useEffect, useRef, useLayoutEffect } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { emit, listen } from '@tauri-apps/api/event';

interface SimpleTerminalProps {
  terminalId: string;
  isActive: boolean;
}

const SimpleTerminal: React.FC<SimpleTerminalProps> = ({ terminalId, isActive }) => {
  const terminalContainerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);

  useLayoutEffect(() => {
    if (!terminalContainerRef.current || termRef.current) return;

    console.log(`[${terminalId}] Initializing terminal`);
    const term = new Terminal({
      cursorBlink: true,
      fontFamily: '"Cascadia Code", Menlo, Monaco, "Courier New", monospace',
      fontSize: 14,
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#f8f8f8',
        selectionBackground: '#555555',
      },
      allowTransparency: true,
    });
    termRef.current = term;

    const fitAddon = new FitAddon();
    fitAddonRef.current = fitAddon;
    term.loadAddon(fitAddon);

    term.open(terminalContainerRef.current);
    fitAddon.fit();

    term.onData((data) => {
      emit('pty-input', { terminal_id: terminalId, data });
    });

    emit('terminal-ready', { terminal_id: terminalId });

    const resizeObserver = new ResizeObserver(() => {
      fitAddonRef.current?.fit();
    });
    resizeObserver.observe(terminalContainerRef.current);

    return () => {
      resizeObserver.disconnect();
      term.dispose();
      termRef.current = null;
    };
  }, [terminalId]);

  useEffect(() => {
    const unlistenPromise = listen<string>(`pty-data-${terminalId}`, (event) => {
      termRef.current?.write(event.payload);
    });

    return () => {
      unlistenPromise.then(unlisten => unlisten());
    };
  }, [terminalId]);

  useEffect(() => {
    if (isActive) {
      termRef.current?.focus();
      fitAddonRef.current?.fit();
    }
  }, [isActive]);

  return (
    <div
      ref={terminalContainerRef}
      style={{
        width: '100%',
        height: '100%',
        visibility: isActive ? 'visible' : 'hidden',
        position: isActive ? 'relative' : 'absolute',
      }}
    />
  );
};

export default SimpleTerminal;
