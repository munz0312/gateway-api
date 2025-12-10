"use client"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"
import { BackendRequestLog, Duration } from "@/lib/types"
import { parseDuration } from "@/lib/utils"

interface RequestLogsProps {
  logs: BackendRequestLog[]
}

export function RequestLogs({ logs }: RequestLogsProps) {
  const getStatusColor = (status: number) => {
    if (status >= 200 && status < 300) return "bg-chart-1"
    if (status >= 400 && status < 500) return "bg-chart-2"
    return "bg-chart-3"
  }

  const getMethodColor = (method: string) => {
    switch (method) {
      case "GET":
        return "bg-chart-1/20 text-chart-1"
      case "POST":
        return "bg-chart-2/20 text-chart-2"
      case "PUT":
        return "bg-chart-4/20 text-chart-4"
      case "DELETE":
        return "bg-chart-3/20 text-chart-3"
      default:
        return "bg-chart-5/20 text-chart-5"
    }
  }

  const formatLatency = (duration: Duration) => {
    return parseDuration(duration).toFixed(1)
  }

  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString()
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Request Logs</CardTitle>
        <CardDescription>Real-time request activity from the proxy server</CardDescription>
      </CardHeader>
      <CardContent>
        <ScrollArea className="h-[400px] w-full">
          <div className="space-y-2">
            {logs.map((log, i) => (
              <div
                key={`${log.timestamp}-${i}`}
                className="flex items-center gap-4 rounded-md border border-border bg-card p-3 text-sm font-mono transition-colors hover:bg-accent/50"
              >
                <span className="text-muted-foreground text-xs w-20 shrink-0">{formatTime(log.timestamp)}</span>
                <Badge variant="secondary" className={`w-16 justify-center ${getMethodColor(log.method)}`}>
                  {log.method}
                </Badge>
                <span className="flex-1 truncate text-foreground">{log.path}</span>
                <div className="flex items-center gap-2 shrink-0">
                  <div className={`size-2 rounded-full ${getStatusColor(log.status)}`} />
                  <span className="w-10 text-muted-foreground">{log.status}</span>
                </div>
                <span className="text-muted-foreground w-16 text-right">{formatLatency(log.response_time)}ms</span>
                <span className="text-muted-foreground text-xs w-28 text-right">{log.client_ip}</span>
              </div>
            ))}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  )
}
