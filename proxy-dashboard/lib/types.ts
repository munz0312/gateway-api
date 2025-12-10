export interface DurationObj {
  secs: number;
  nanos: number;
}

export type Duration = DurationObj | string;

export interface BackendRequestLog {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  response_time: Duration;
  client_ip: string;
}

export interface SummaryMetrics {
  total_requests: number;
  total_errors: number;
  active_connections: number;
  recent_logs: BackendRequestLog[];
  route_stats: Record<string, number>;
}

export interface WsMessage {
  type: "NewLog" | "MetricsUpdate";
  log?: BackendRequestLog;
  metrics?: SummaryMetrics;
}