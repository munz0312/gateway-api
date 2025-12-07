import { useState, useEffect, useRef } from 'react';
import './App.css';

// --- Types ---

interface ResponseTime extends Array<number> {
  0: number; // secs
  1: number; // nanos
}

interface RequestLog {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  response_time: ResponseTime;
  client_ip: string;
}

interface RouteStats {
  [path: string]: number;
}

interface SummaryMetrics {
  total_requests: number;
  total_errors: number;
  active_connections: number;
  recent_logs: RequestLog[];
  route_stats: RouteStats;
}

type WsMessage =
  | { type: 'NewLog'; log: RequestLog }
  | { type: 'MetricsUpdate'; metrics: SummaryMetrics };

type MessageWithId = WsMessage & { id: number };

type FilterType = 'All' | 'NewLog' | 'MetricsUpdate';

// --- Components ---

function formatDuration(rt: ResponseTime): string {
  const secs = rt[0];
  const nanos = rt[1];
  const ms = (secs * 1000) + (nanos / 1_000_000);
  return `${ms.toFixed(2)}ms`;
}

function LogEntry({ data }: { data: RequestLog }) {
  return (
    <div className="log-entry">
      <span className="timestamp">{new Date(data.timestamp).toLocaleTimeString()}</span>
      <span className={`method ${data.method.toLowerCase()}`}>{data.method}</span>
      <span className="path">{data.path}</span>
      <span className={`status status-${data.status}`}>{data.status}</span>
      <span className="duration">{formatDuration(data.response_time)}</span>
      <span className="ip">{data.client_ip}</span>
    </div>
  );
}

function MetricsEntry({ data, id }: { data: SummaryMetrics; id: number }) {
  return (
    <div className="metrics-entry">
      <h4>Metrics Update #{id}</h4>
      <div className="metrics-grid">
        <div className="metric-item">
          <label>Total Requests</label>
          <span>{data.total_requests}</span>
        </div>
        <div className="metric-item">
          <label>Errors</label>
          <span>{data.total_errors}</span>
        </div>
        <div className="metric-item">
          <label>Active Conn</label>
          <span>{data.active_connections}</span>
        </div>
      </div>
      <div className="route-stats">
        <h5>Route Stats</h5>
        <ul>
          {Object.entries(data.route_stats).map(([route, count]) => (
            <li key={route}>
              <span>{route}</span>: <span>{count}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

function App() {
  const [messages, setMessages] = useState<MessageWithId[]>([]);
  const [status, setStatus] = useState<'Connecting' | 'Connected' | 'Disconnected'>('Connecting');
  const [filter, setFilter] = useState<FilterType>('All');
  const ws = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Connect to WebSocket
    const connect = () => {
      setStatus('Connecting');
      const socket = new WebSocket('ws://127.0.0.1:3000/ws');
      ws.current = socket;

      socket.onopen = () => {
        setStatus('Connected');
        console.log('WebSocket connected');
      };

      socket.onmessage = (event) => {
        try {
          const data: WsMessage = JSON.parse(event.data);
          setMessages((prev) => {
            const nextId = prev.length > 0 ? prev[prev.length - 1].id + 1 : 1;
            return [...prev, { ...data, id: nextId }].slice(-200);
          });
        } catch (err) {
          console.error('Failed to parse WS message', err);
        }
      };

      socket.onclose = () => {
        setStatus('Disconnected');
        console.log('WebSocket disconnected, retrying in 3s...');
        setTimeout(connect, 3000);
      };

      socket.onerror = (error) => {
        console.error('WebSocket error', error);
        socket.close();
      };
    };

    connect();

    return () => {
      if (ws.current) {
        ws.current.close();
      }
    };
  }, []);

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const filteredMessages = messages.filter((msg) => {
    if (filter === 'All') return true;
    return msg.type === filter;
  });

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>Gateway Monitor</h1>
        <div className={`connection-status ${status.toLowerCase()}`}>
          {status}
        </div>
      </header>

      <div className="controls">
        <div className="filters">
          <button 
            className={filter === 'All' ? 'active' : ''} 
            onClick={() => setFilter('All')}
          >
            All
          </button>
          <button 
            className={filter === 'NewLog' ? 'active' : ''} 
            onClick={() => setFilter('NewLog')}
          >
            Logs
          </button>
          <button 
            className={filter === 'MetricsUpdate' ? 'active' : ''} 
            onClick={() => setFilter('MetricsUpdate')}
          >
            Metrics
          </button>
        </div>
        <button onClick={() => setMessages([])} className="clear-btn">
          Clear
        </button>
      </div>

      <div className="message-list">
        {filteredMessages.length === 0 ? (
          <div className="empty-state">No messages yet...</div>
        ) : (
          filteredMessages.map((msg) => (
            <div key={msg.id} className={`message-wrapper type-${msg.type}`}>
              {msg.type === 'NewLog' ? (
                <LogEntry data={msg.log} />
              ) : (
                <MetricsEntry data={msg.metrics} id={msg.id} />
              )}
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>
    </div>
  );
}

export default App;
