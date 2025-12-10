"use client"

import { useState, useEffect, useRef } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { MetricsChart, ChartDataPoint } from "@/components/metrics-chart"
import { RequestLogs } from "@/components/request-logs"
import { Activity, Clock, TrendingUp, AlertCircle } from "lucide-react"
import { BackendRequestLog, SummaryMetrics, WsMessage } from "@/lib/types"
import { parseDuration } from "@/lib/utils"

interface HistoryPoint {
  time: string
  requests: number // per minute
  latency: number // ms
  errorRate: number // %
  success: number
  client_error: number
  server_error: number
}

export default function ProxyDashboard() {
  const [summary, setSummary] = useState<SummaryMetrics>({
    total_requests: 0,
    total_errors: 0,
    active_connections: 0,
    recent_logs: [],
    route_stats: {},
  })
  const [logs, setLogs] = useState<BackendRequestLog[]>([])
  const [history, setHistory] = useState<HistoryPoint[]>([])
  const [isConnected, setIsConnected] = useState(false)

  const windowLogs = useRef<BackendRequestLog[]>([])

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:3000/ws")

    ws.onopen = () => {
      setIsConnected(true)
      console.log("Connected to WebSocket")
    }

    ws.onclose = () => {
      setIsConnected(false)
      console.log("Disconnected from WebSocket")
    }

    ws.onmessage = (event) => {
      try {
        const message: WsMessage = JSON.parse(event.data)

        if (message.type === "NewLog" && message.log) {
          const log = message.log
          windowLogs.current.push(log)
          setLogs((prev) => [log, ...prev].slice(0, 50))
        } else if (message.type === "MetricsUpdate" && message.metrics) {
          setSummary(message.metrics)
        }
      } catch (error) {
        console.error("Failed to parse WebSocket message:", error)
      }
    }

    return () => {
      ws.close()
    }
  }, [])

  useEffect(() => {
    const interval = setInterval(() => {
      const currentLogs = windowLogs.current
      windowLogs.current = [] // Clear buffer

      const now = new Date()
      const timeLabel = `${now.getHours().toString().padStart(2, "0")}:${now.getMinutes().toString().padStart(2, "0")}:${now.getSeconds().toString().padStart(2, "0")}`

      let avgLatency = 0
      let success = 0
      let client_error = 0
      let server_error = 0

      if (currentLogs.length > 0) {
        const totalLatency = currentLogs.reduce(
          (acc, log) => acc + parseDuration(log.response_time),
          0,
        )
        avgLatency = totalLatency / currentLogs.length

        currentLogs.forEach((log) => {
          if (log.status >= 200 && log.status < 300) success++
          else if (log.status >= 400 && log.status < 500) client_error++
          else if (log.status >= 500) server_error++
        })
      }

      // Extrapolate to requests per minute (interval is 5s)
      const requestsPerMin = currentLogs.length * 12
      const totalErrors = client_error + server_error
      const errorRate = currentLogs.length > 0 ? (totalErrors / currentLogs.length) * 100 : 0

      const newPoint: HistoryPoint = {
        time: timeLabel,
        requests: requestsPerMin,
        latency: Math.round(avgLatency),
        errorRate: Number(errorRate.toFixed(1)),
        success,
        client_error,
        server_error,
      }

      setHistory((prev) => [...prev.slice(-23), newPoint])
    }, 5000)

    return () => clearInterval(interval)
  }, [])

  // Derived metrics for cards (use latest history point or summary)
  const latestStats = history[history.length - 1] || {
    requests: 0,
    latency: 0,
    errorRate: 0,
    success: 0,
    client_error: 0,
    server_error: 0,
  }

  // Calculate success rate from summary if possible, or use latest window
  const globalSuccessRate =
    summary.total_requests > 0
      ? ((summary.total_requests - summary.total_errors) / summary.total_requests) * 100
      : 100

  return (
    <div className="min-h-screen bg-background p-4 md:p-8">
      <div className="mx-auto max-w-7xl space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-balance text-3xl font-semibold tracking-tight">Proxy Monitoring</h1>
            <p className="text-pretty text-muted-foreground mt-1">Real-time metrics and request logs</p>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant={isConnected ? "outline" : "destructive"} className="gap-1.5">
              <div className={`size-2 rounded-full ${isConnected ? "bg-chart-1 animate-pulse" : "bg-red-500"}`} />
              {isConnected ? "Live" : "Disconnected"}
            </Badge>
          </div>
        </div>

        {/* Metrics Cards */}
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Total Requests</CardTitle>
              <Activity className="size-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-semibold">{summary.total_requests.toLocaleString()}</div>
              <p className="text-xs text-muted-foreground mt-1">{(latestStats.requests / 60).toFixed(1)} req/s</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Avg Latency</CardTitle>
              <Clock className="size-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-semibold">{latestStats.latency}ms</div>
              <p className="text-xs text-muted-foreground mt-1">Response time (5s window)</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
              <TrendingUp className="size-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-semibold">{globalSuccessRate.toFixed(1)}%</div>
              <p className="text-xs text-muted-foreground mt-1">
                {(summary.total_requests - summary.total_errors).toLocaleString()} successful
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Error Rate</CardTitle>
              <AlertCircle className="size-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-semibold">{latestStats.errorRate}%</div>
              <p className="text-xs text-muted-foreground mt-1">
                {summary.total_errors.toLocaleString()} errors (total)
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Charts */}
        <div className="grid gap-4 md:grid-cols-2">
          <MetricsChart
            title="Requests"
            description="Requests per minute"
            type="requests"
            data={history.map((h) => ({ time: h.time, value: h.requests }))}
          />
          <MetricsChart
            title="Response Time"
            description="Average latency in milliseconds"
            type="latency"
            data={history.map((h) => ({ time: h.time, value: h.latency }))}
          />
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          <MetricsChart
            title="Status Codes"
            description="HTTP status code distribution (count per window)"
            type="status"
            data={history.map((h) => ({
              time: h.time,
              success: h.success,
              client_error: h.client_error,
              server_error: h.server_error,
            }))}
          />
          <MetricsChart
            title="Error Rate"
            description="Error percentage over time"
            type="errors"
            data={history.map((h) => ({ time: h.time, value: h.errorRate }))}
          />
        </div>

        {/* Request Logs */}
        <RequestLogs logs={logs} />
      </div>
    </div>
  )
}
