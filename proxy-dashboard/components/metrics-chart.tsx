"use client"

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Area, AreaChart, CartesianGrid, XAxis, YAxis, ResponsiveContainer, Tooltip } from "recharts"

export interface ChartDataPoint {
  time: string
  value?: number
  success?: number
  client_error?: number
  server_error?: number
  [key: string]: any
}

interface MetricsChartProps {
  title: string
  description: string
  type: "requests" | "latency" | "status" | "errors"
  data?: ChartDataPoint[]
}

export function MetricsChart({ title, description, type, data = [] }: MetricsChartProps) {
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="rounded-lg border bg-background p-2 shadow-sm">
          <div className="grid gap-2">
            <div className="text-xs text-muted-foreground">{label}</div>
            {payload.map((entry: any, index: number) => (
              <div key={index} className="flex items-center gap-2">
                <div className="size-2 rounded-full" style={{ backgroundColor: entry.color }} />
                <span className="text-xs font-medium">
                  {entry.name}: {entry.value.toLocaleString()}
                  {type === "latency" ? "ms" : type === "errors" ? "%" : ""}
                </span>
              </div>
            ))}
          </div>
        </div>
      )
    }
    return null
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="h-[200px] w-full">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={data}>
              <defs>
                <linearGradient id="fillValue" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#FFF" stopOpacity={0.3} />
                  <stop offset="95%" stopColor="hsl(var(--chart-1))" stopOpacity={0} />
                </linearGradient>
                {type === "status" && (
                  <>
                    <linearGradient id="fillSuccess" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#FFF" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="hsl(var(--chart-1))" stopOpacity={0} />
                    </linearGradient>
                    <linearGradient id="fillClientError" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#FFF" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="hsl(var(--chart-2))" stopOpacity={0} />
                    </linearGradient>
                    <linearGradient id="fillServerError" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#FFF" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="hsl(var(--chart-3))" stopOpacity={0} />
                    </linearGradient>
                  </>
                )}
              </defs>
              <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" vertical={false} />
              <XAxis
                dataKey="time"
                stroke="#FFF"
                fontSize={12}
                tickLine={false}
                axisLine={false}
              />
              <YAxis
                stroke="#FFF"
                fontSize={12}
                tickLine={false}
                axisLine={false}
                tickFormatter={(value) => `${value}`}
              />
              <Tooltip content={<CustomTooltip />} />
              {type === "status" ? (
                <>
                                      <Area
                                        type="monotone"
                                        dataKey="success"
                                        name="2xx Success"
                                        stackId="1"
                                        stroke="#FFF"
                                        fill="url(#fillSuccess)"                    strokeWidth={2}
                  />
                                      <Area
                                        type="monotone"
                                        dataKey="client_error"
                                        name="4xx Client Error"
                                        stackId="1"
                                        stroke="#FFF"
                                        fill="url(#fillClientError)"                    strokeWidth={2}
                  />
                                      <Area
                                        type="monotone"
                                        dataKey="server_error"
                                        name="5xx Server Error"
                                        stackId="1"
                                        stroke="#FFF"
                                        fill="url(#fillServerError)"                    strokeWidth={2}
                  />
                </>
              ) : (
                <Area
                  type="monotone"
                  dataKey="value"
                  name={title}
                  stroke="#FFF"
                  fill="url(#fillValue)"
                  strokeWidth={2}
                />
              )}
            </AreaChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  )
}
